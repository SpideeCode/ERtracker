//! Extracted map data on disk: status, extraction runs, tile serving and
//! placing the player on the map.

use crate::extract::build::{self, Manifest, FORMAT, LEGACY_CONV_FILE};
use crate::extract::world::{MapPos, Projector};
use crate::extract::TilePos;
use crate::paths;
use crate::save::Position;
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::http::{Request, Response};
use tauri::{AppHandle, Emitter, Manager};

pub const SCHEME: &str = "ertiles";
pub const PROGRESS_EVENT: &str = "maps://progress";

pub fn dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from(".")).join("maps")
}

fn read_manifest(dir: &Path) -> Option<Manifest> {
    serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).ok()?).ok()
}

#[derive(Serialize)]
pub struct Status {
    pub manifest: Option<Manifest>,
    pub game_dir: Option<PathBuf>,
    /// The game was updated (or the extraction format changed) since the maps were built.
    pub outdated: bool,
    pub running: bool,
}

static RUNNING: Mutex<bool> = Mutex::new(false);

pub fn status(app: &AppHandle, game_dir: Option<PathBuf>) -> Status {
    let manifest = read_manifest(&dir(app));
    let game_dir = game_dir.filter(|d| paths::is_game_dir(d)).or_else(paths::find_game_dir);
    let outdated = match (&manifest, &game_dir) {
        (Some(m), Some(g)) => m.format != FORMAT || m.game_build != build::game_build(&g.join("Game")),
        (Some(m), None) => m.format != FORMAT,
        _ => false,
    };
    Status { manifest, game_dir, outdated, running: *RUNNING.lock().unwrap() }
}

#[derive(Clone, Serialize)]
struct ProgressEvent {
    step: String,
    done: usize,
    total: usize,
    error: Option<String>,
    finished: bool,
}

/// Runs the extraction on a background thread; progress arrives as events.
pub fn start_extraction(app: &AppHandle, game_dir: PathBuf) -> Result<(), String> {
    if !paths::is_game_dir(&game_dir) {
        return Err(format!("{} ne contient pas Game\\eldenring.exe", game_dir.display()));
    }
    {
        let mut running = RUNNING.lock().unwrap();
        if *running {
            return Err("extraction déjà en cours".into());
        }
        *running = true;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let emit = |e: ProgressEvent| {
            let _ = app.emit(PROGRESS_EVENT, e);
        };
        // Build into a fresh folder, then swap it in, so a failed run keeps the old maps.
        let target = dir(&app);
        let staging = target.with_extension("new");
        let _ = std::fs::remove_dir_all(&staging);
        let mut last = 0;
        let result = build::build(&game_dir, &staging, |p| {
            if p.done == p.total || p.done >= last + 25 || p.done < last {
                last = p.done;
                emit(ProgressEvent { step: p.step.into(), done: p.done, total: p.total, error: None, finished: false });
            }
        })
        .map_err(|e| e.to_string())
        .and_then(|_| {
            let _ = std::fs::remove_dir_all(&target);
            std::fs::rename(&staging, &target).map_err(|e| e.to_string())
        });
        *PROJECTOR.lock().unwrap() = None;
        *RUNNING.lock().unwrap() = false;
        emit(ProgressEvent { step: "done".into(), done: 1, total: 1, error: result.err(), finished: true });
    });
    Ok(())
}

/// Serves files of the maps folder (tiles, markers.json) to the web view.
pub fn serve(app: &AppHandle, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path = request.uri().path().trim_start_matches('/');
    let rel = Path::new(path);
    let not_found = || Response::builder().status(404).body(Vec::new()).unwrap();
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return not_found();
    }
    let Ok(body) = std::fs::read(dir(app).join(rel)) else {
        return not_found();
    };
    let mime = match rel.extension().and_then(|e| e.to_str()) {
        Some("jpg") => "image/jpeg",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };
    Response::builder()
        .header("Content-Type", mime)
        .header("Cache-Control", "max-age=86400")
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        .unwrap()
}

static PROJECTOR: Mutex<Option<std::sync::Arc<Projector>>> = Mutex::new(None);
static DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init(app: &AppHandle) {
    let _ = DIR.set(dir(app));
}

/// The player's saved position on the extracted maps, when they exist.
pub fn player_position(pos: &Position) -> Option<MapPos> {
    let projector = {
        let mut cached = PROJECTOR.lock().unwrap();
        if cached.is_none() {
            let data = std::fs::read(DIR.get()?.join(LEGACY_CONV_FILE)).ok()?;
            *cached = Some(std::sync::Arc::new(Projector::new(&data).ok()?));
        }
        cached.clone()?
    };
    // "m61_46_45_00" -> area 61, grid 46/45
    let mut parts = pos.map.trim_start_matches('m').split('_').map(|p| p.parse::<u8>().ok());
    let (Some(area), Some(grid_x), Some(grid_z)) = (parts.next()?, parts.next()?, parts.next()?) else {
        return None;
    };
    projector.project(TilePos { area, grid_x, grid_z, x: pos.x, y: pos.y, z: pos.z })
}
