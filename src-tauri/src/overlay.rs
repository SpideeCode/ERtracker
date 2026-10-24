//! In-game overlay: a separate transparent, borderless, always-on-top window
//! covering the screen, that lets clicks through to the game. Nothing is
//! injected into the game; it only works with the game in borderless windowed
//! mode (exclusive fullscreen hides every other window).

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub const LABEL: &str = "overlay";
pub const INTERACTIVE_EVENT: &str = "overlay://interactive";

static INTERACTIVE: AtomicBool = AtomicBool::new(false);

fn toggle_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyO)
}

fn interact_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyI)
}

/// Shows the overlay (creating it on first use) or hides it. Returns whether it is now visible.
pub fn toggle(app: &AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window(LABEL) {
        let visible = win.is_visible().map_err(|e| e.to_string())?;
        if visible {
            win.hide().map_err(|e| e.to_string())?;
        } else {
            win.show().map_err(|e| e.to_string())?;
        }
        return Ok(!visible);
    }
    create(app)?;
    Ok(true)
}

fn create(app: &AppHandle) -> Result<(), String> {
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("overlay".into()))
        .title("ERtracker overlay")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;
    // Cover the monitor the game is most likely on: the primary one.
    if let Some(monitor) = win.primary_monitor().map_err(|e| e.to_string())? {
        let (pos, size) = (monitor.position(), monitor.size());
        win.set_position(PhysicalPosition::new(pos.x, pos.y)).map_err(|e| e.to_string())?;
        win.set_size(PhysicalSize::new(size.width, size.height)).map_err(|e| e.to_string())?;
    }
    win.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
    INTERACTIVE.store(false, Ordering::SeqCst);
    win.show().map_err(|e| e.to_string())
}

/// Click-through (false) or clickable (true). Interactive mode takes the focus.
pub fn set_interactive(app: &AppHandle, on: bool) -> Result<(), String> {
    let Some(win) = app.get_webview_window(LABEL) else { return Ok(()) };
    win.set_ignore_cursor_events(!on).map_err(|e| e.to_string())?;
    if on {
        let _ = win.show();
        let _ = win.set_focus();
    }
    INTERACTIVE.store(on, Ordering::SeqCst);
    let _ = app.emit(INTERACTIVE_EVENT, on);
    Ok(())
}

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            // The handler runs on the event loop: creating a window here would deadlock.
            let app = app.clone();
            if *shortcut == toggle_shortcut() {
                tauri::async_runtime::spawn(async move {
                    let _ = toggle(&app);
                });
            } else if *shortcut == interact_shortcut() {
                tauri::async_runtime::spawn(async move {
                    let _ = set_interactive(&app, !INTERACTIVE.load(Ordering::SeqCst));
                });
            }
        })
        .build()
}

/// Registers Ctrl+Shift+O (show/hide) and Ctrl+Shift+I (interact). Failures (shortcut
/// taken by another app) are not fatal.
pub fn register_shortcuts(app: &AppHandle) {
    for s in [toggle_shortcut(), interact_shortcut()] {
        if let Err(e) = app.global_shortcut().register(s) {
            eprintln!("overlay shortcut: {e}");
        }
    }
}
