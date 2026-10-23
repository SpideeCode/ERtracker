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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = app.path().app_config_dir()?.join("config.json");
            app.manage::<TrackerState>(std::sync::Mutex::new(Tracker::new(config)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_saves, find_game_dir, load_state, select])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
