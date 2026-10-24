pub mod extract;
mod images;
pub mod journal;
mod maps;
mod overlay;
pub mod pathfind;
pub mod paths;
pub mod progress;
pub mod save;
mod tracker;

use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tracker::{Tracker, TrackerState, Update};

#[tauri::command]
fn list_saves() -> Vec<paths::SaveLocation> {
    paths::find_saves()
}

#[tauri::command]
fn find_game_dir() -> Option<PathBuf> {
    paths::find_game_dir()
}

/// Current state; also starts watching the save file.
#[tauri::command]
fn load_state(app: AppHandle, state: State<TrackerState>) -> Result<Option<Update>, String> {
    let mut tracker = state.lock().unwrap();
    let update = tracker.refresh()?;
    tracker.watch(&app)?;
    Ok(update)
}

#[tauri::command]
fn select(
    app: AppHandle,
    state: State<TrackerState>,
    save_path: Option<PathBuf>,
    slot: Option<usize>,
) -> Result<Option<Update>, String> {
    let mut tracker = state.lock().unwrap();
    tracker.select(save_path, slot);
    let update = tracker.refresh()?;
    tracker.watch(&app)?;
    Ok(update)
}

#[tauri::command]
fn map_status(app: AppHandle, game_dir: Option<PathBuf>) -> maps::Status {
    maps::status(&app, game_dir)
}

/// Starts extracting maps from the game install; progress comes as `maps://progress` events.
#[tauri::command]
fn extract_maps(app: AppHandle, game_dir: PathBuf) -> Result<(), String> {
    maps::start_extraction(&app, game_dir)
}

#[tauri::command]
fn game_running() -> bool {
    paths::game_running()
}

/// Sets the held runes of a character in the selected save (game must be closed).
#[tauri::command]
fn set_runes(state: State<TrackerState>, slot: usize, value: u32) -> Result<save::RuneEdit, String> {
    if paths::game_running() {
        return Err("Elden Ring est lancé : ferme complètement le jeu avant de modifier la sauvegarde.".into());
    }
    let path = state.lock().unwrap().save_path().ok_or("aucune sauvegarde sélectionnée")?;
    save::write_runes(&path, slot, value).map_err(|e| e.to_string())
}

/// Walking route between two points of a map, over the extracted ground samples.
#[tauri::command]
async fn find_path(map: String, from: pathfind::Point, to: pathfind::Point) -> Result<pathfind::PathResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let graph = maps::graph().ok_or("données de chemins absentes : mets à jour les cartes")?;
        Ok(graph.find(&map, from, to))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Recorded history of the current character (one entry per save that changed something).
#[tauri::command]
fn get_journal(state: State<TrackerState>) -> Vec<journal::Record> {
    state.lock().unwrap().journal()
}

// Async: on Windows, creating a window from a synchronous command deadlocks the event loop.
#[tauri::command]
async fn toggle_overlay(app: AppHandle) -> Result<bool, String> {
    overlay::toggle(&app)
}

#[tauri::command]
async fn set_overlay_interactive(app: AppHandle, on: bool) -> Result<(), String> {
    overlay::set_interactive(&app, on)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(overlay::plugin())
        .register_uri_scheme_protocol(maps::SCHEME, |ctx, request| maps::serve(ctx.app_handle(), &request))
        .setup(|app| {
            maps::init(app.handle());
            overlay::register_shortcuts(app.handle());
            let config = app.path().app_config_dir()?.join("config.json");
            let journal = app.path().app_data_dir()?.join("journal");
            app.manage::<TrackerState>(std::sync::Mutex::new(Tracker::new(config, journal)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_saves, find_game_dir, load_state, select, map_status, extract_maps, game_running, set_runes, find_path, get_journal, toggle_overlay, set_overlay_interactive])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
