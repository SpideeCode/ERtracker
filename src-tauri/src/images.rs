//! Illustrations read from the player's own game on demand, then cached next to
//! the extracted data: item artwork (MENU_Knowledge_*, by icon id) and loading
//! screen scenes (MENU_Load_*). Nothing is shipped with the app.
//!
//! Served as `img/icon/<icon id>.png` and `img/load/<n>.jpg` by the map scheme.

use crate::extract::{archive::Archive, binder, dcx, oodle, tiles};
use crate::paths;
use image::imageops::FilterType;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const BHD: &str = "/menu/hi/00_solo.tpfbhd";
const BDT: &str = "/menu/hi/00_solo.tpfbdt";
const ICON_SIZE: u32 = 256;
const LOAD_WIDTH: u32 = 1600;

struct Source {
    archive: Archive,
    entries: HashMap<String, binder::Entry>,
}

/// Archives holding the menu textures (the DLC adds its own), opened on first use.
static SOURCES: Mutex<Option<Arc<Vec<Source>>>> = Mutex::new(None);

fn sources(data_dir: &Path) -> Option<Arc<Vec<Source>>> {
    let mut cached = SOURCES.lock().unwrap();
    if cached.is_none() {
        let root = std::fs::read_to_string(data_dir.join("game_dir.txt"))
            .ok()
            .map(PathBuf::from)
            .filter(|d| paths::is_game_dir(d))
            .or_else(paths::find_game_dir)?;
        let game = root.join("Game");
        oodle::init(&game).ok()?;
        let list: Vec<Source> = ["DLC", "Data0"]
            .iter()
            .filter_map(|name| {
                let archive = Archive::open(&game, name).ok()?;
                let bhd = dcx::decompress(&archive.read(BHD).ok()?).ok()?;
                let entries = binder::entries(&bhd)
                    .ok()?
                    .into_iter()
                    .map(|e| {
                        let stem = e.name.rsplit(['\\', '/']).next().unwrap_or(&e.name).trim_end_matches(".tpf.dcx").to_string();
                        (stem, e)
                    })
                    .collect();
                Some(Source { archive, entries })
            })
            .collect();
        *cached = Some(Arc::new(list));
    }
    cached.clone()
}

fn texture(data_dir: &Path, stem: &str) -> Option<image::RgbaImage> {
    let list = sources(data_dir)?;
    let (source, entry) = list.iter().find_map(|s| s.entries.get(stem).map(|e| (s, e)))?;
    let packed = source.archive.read_range(BDT, entry.offset, entry.size).ok()?;
    let tpf = dcx::decompress(&packed).ok()?;
    tiles::decode_tpf(&tpf).ok()
}

/// Bytes of `img/...`, made and cached on first request.
pub fn get(data_dir: &Path, rel: &Path) -> Option<Vec<u8>> {
    let file = data_dir.join(rel);
    if let Ok(bytes) = std::fs::read(&file) {
        return Some(bytes);
    }
    let mut parts = rel.iter().map(|p| p.to_string_lossy().to_string());
    let (Some(root), Some(kind), Some(name)) = (parts.next(), parts.next(), parts.next()) else { return None };
    if root != "img" {
        return None;
    }
    let id: u32 = name.split('.').next()?.parse().ok()?;
    let mut out = Vec::new();
    match kind.as_str() {
        "icon" => {
            let img = texture(data_dir, &format!("MENU_Knowledge_{id:05}"))?;
            let img = image::imageops::resize(&img, ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
            img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).ok()?;
        }
        "load" => {
            let img = texture(data_dir, &format!("MENU_Load_{id:05}"))?;
            let height = LOAD_WIDTH * img.height() / img.width().max(1);
            let img = image::DynamicImage::ImageRgba8(img).resize_exact(LOAD_WIDTH, height, FilterType::Triangle).to_rgb8();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 82);
            encoder.encode_image(&img).ok()?;
        }
        _ => return None,
    }
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&file, &out);
    Some(out)
}
