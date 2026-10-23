//! Finds the save files and the game install without any user input.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const SAVE_NAMES: [&str; 2] = ["ER0000.sl2", "ER0000.co2"]; // vanilla, Seamless Coop
const STEAM_APP_DIR: &str = r"steamapps\common\ELDEN RING";

#[derive(Clone, Debug, Serialize)]
pub struct SaveLocation {
    pub path: PathBuf,
    pub steam_id: String,
    pub seamless_coop: bool,
    /// Last write, in seconds since the Unix epoch.
    pub modified: u64,
}

/// Every save under %APPDATA%\EldenRing\<SteamID64>\, most recently written first.
pub fn find_saves() -> Vec<SaveLocation> {
    let Some(root) = std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("EldenRing")) else {
        return Vec::new();
    };
    let Ok(dirs) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut saves: Vec<SaveLocation> = dirs
        .flatten()
        .filter(|d| {
            // Skip copies such as "7656119... - Copie": only real SteamID64 folders.
            let name = d.file_name();
            let name = name.to_string_lossy();
            name.len() == 17 && name.bytes().all(|b| b.is_ascii_digit())
        })
        .flat_map(|dir| {
            let steam_id = dir.file_name().to_string_lossy().into_owned();
            SAVE_NAMES.iter().filter_map(move |file| {
                let path = dir.path().join(file);
                let modified = path.metadata().ok()?.modified().ok()?;
                Some(SaveLocation {
                    seamless_coop: file.ends_with(".co2"),
                    steam_id: steam_id.clone(),
                    modified: modified.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()),
                    path,
                })
            })
        })
        .collect();
    saves.sort_by(|a, b| b.modified.cmp(&a.modified));
    saves
}

pub fn is_game_dir(dir: &Path) -> bool {
    dir.join("Game").join("eldenring.exe").is_file()
}

/// The "ELDEN RING" folder (the one containing `Game\eldenring.exe`).
pub fn find_game_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    // Steam: registry -> every library listed in libraryfolders.vdf.
    for steam in steam_roots() {
        candidates.push(steam.join(STEAM_APP_DIR));
        let vdf = steam.join(r"steamapps\libraryfolders.vdf");
        if let Ok(text) = std::fs::read_to_string(vdf) {
            candidates.extend(library_paths(&text).map(|lib| lib.join(STEAM_APP_DIR)));
        }
    }

    // Usual locations on every drive, Steam first, then standalone installs.
    let drives: Vec<PathBuf> = (b'C'..=b'Z')
        .map(|d| PathBuf::from(format!("{}:\\", d as char)))
        .filter(|d| d.exists())
        .collect();
    for drive in &drives {
        for sub in [r"Program Files (x86)\Steam", r"Program Files\Steam", "SteamLibrary", r"Steam", r"Games\Steam"] {
            candidates.push(drive.join(sub).join(STEAM_APP_DIR));
        }
    }
    for drive in &drives {
        for sub in [r"Games\ELDEN RING", "ELDEN RING", r"Jeux\ELDEN RING"] {
            candidates.push(drive.join(sub));
        }
    }

    candidates.into_iter().find(|dir| is_game_dir(dir))
}

fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        use winreg::RegKey;
        let keys = [
            (HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Valve\Steam", "InstallPath"),
        ];
        for (hive, key, value) in keys {
            if let Ok(path) = RegKey::predef(hive).open_subkey(key).and_then(|k| k.get_value::<String, _>(value)) {
                roots.push(PathBuf::from(path.replace('/', "\\")));
            }
        }
    }
    roots
}

/// `"path"		"D:\\SteamLibrary"` lines of libraryfolders.vdf.
fn library_paths(vdf: &str) -> impl Iterator<Item = PathBuf> + '_ {
    vdf.lines().filter_map(|line| {
        let mut parts = line.split('"').filter(|s| !s.trim().is_empty());
        (parts.next()? == "path").then(|| PathBuf::from(parts.next().unwrap_or_default().replace("\\\\", "\\")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_library_folders() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}";
        let libs: Vec<_> = library_paths(vdf).collect();
        assert_eq!(libs, [PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]);
    }
}
