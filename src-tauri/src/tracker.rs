//! Current selection (save file + character), file watching and change events.

use crate::extract::world::MapPos;
use crate::{journal, maps, paths};
use crate::progress::{self, Diff};
use crate::save::{Character, SaveFile};
use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

pub const UPDATE_EVENT: &str = "tracker://update";

#[derive(Default, Serialize, Deserialize)]
struct Config {
    save_path: Option<PathBuf>,
    slot: Option<usize>,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub save_path: PathBuf,
    pub characters: Vec<Character>,
    pub slot: usize,
    pub completed: Vec<String>,
    /// Selected character's saved position on the extracted maps.
    pub map_position: Option<MapPos>,
    /// Milliseconds since the Unix epoch.
    pub loaded_at: u64,
    /// Playthrough of the selected character: 0 for the first, 1 for NG+...
    pub cycle: u8,
}

#[derive(Clone, Serialize)]
pub struct Update {
    pub snapshot: Snapshot,
    pub diff: Diff,
}

#[derive(Default)]
pub struct Tracker {
    config: Config,
    config_path: Option<PathBuf>,
    last: Option<(PathBuf, usize, BTreeSet<String>)>,
    journal_dir: Option<PathBuf>,
    /// Journal file of the current character and its last written record.
    journal_last: Option<(PathBuf, journal::Record)>,
    watcher: Option<(PathBuf, Debouncer<RecommendedWatcher>)>,
}

pub type TrackerState = Mutex<Tracker>;

impl Tracker {
    pub fn new(config_path: PathBuf, journal_dir: PathBuf) -> Self {
        let config = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { config, config_path: Some(config_path), journal_dir: Some(journal_dir), ..Default::default() }
    }

    fn save_config(&self) {
        if let Some(path) = &self.config_path {
            let _ = path.parent().map(std::fs::create_dir_all);
            let _ = std::fs::write(path, serde_json::to_string_pretty(&self.config).unwrap_or_default());
        }
    }

    pub fn select(&mut self, save_path: Option<PathBuf>, slot: Option<usize>) {
        if save_path.is_some() && save_path != self.config.save_path {
            self.config.slot = None;
            self.config.save_path = save_path;
        }
        if slot.is_some() {
            self.config.slot = slot;
        }
        self.save_config();
    }

    pub fn save_path(&self) -> Option<PathBuf> {
        self.config
            .save_path
            .clone()
            .filter(|p| p.is_file())
            .or_else(|| paths::find_saves().into_iter().next().map(|s| s.path))
    }

    /// Reloads the save and returns the new state with what changed since the previous call.
    pub fn refresh(&mut self) -> Result<Option<Update>, String> {
        let Some(path) = self.save_path() else {
            return Ok(None);
        };
        let save = load_consistent(&path)?;
        let Some(character) = self
            .config
            .slot
            .and_then(|s| save.character(s))
            .or_else(|| save.characters.iter().max_by_key(|c| c.seconds_played))
        else {
            return Ok(None);
        };

        let mut completed = progress::completed(&save, character);
        for &flag in maps::item_flags().iter() {
            if save.flag(character, flag) == Some(true) {
                completed.insert(format!("item:{flag}"));
            }
        }
        let diff = match &self.last {
            Some((p, s, before)) if *p == path && *s == character.slot => progress::diff(before, &completed),
            _ => progress::diff(&completed, &completed),
        };
        let snapshot = Snapshot {
            save_path: path.clone(),
            characters: save.characters.clone(),
            slot: character.slot,
            completed: completed.iter().cloned().collect(),
            map_position: character.position.as_ref().and_then(maps::player_position),
            loaded_at: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64),
            // Event flags 50-57 mark the playthrough lap (NG, NG+1... NG+7).
            cycle: (0..8u8).rev().find(|&i| save.flag(character, 50 + i as u32) == Some(true)).unwrap_or(0),
        };
        self.record(&path, character, &snapshot, &diff);
        self.last = Some((path, character.slot, completed));
        Ok(Some(Update { snapshot, diff }))
    }

    /// Adds a journal line for this save when something changed since the last one.
    fn record(&mut self, path: &std::path::Path, character: &Character, snapshot: &Snapshot, diff: &Diff) {
        let Some(dir) = &self.journal_dir else { return };
        let file = journal::file(dir, path, character);
        let prev = match &self.journal_last {
            Some((f, r)) if *f == file => Some(r.clone()),
            _ => journal::last(&file),
        };
        let died = prev.as_ref().is_some_and(|p| character.deaths > p.deaths);
        let record = journal::Record {
            t: snapshot.loaded_at,
            deaths: character.deaths,
            level: character.level,
            runes: character.runes,
            played: character.seconds_played,
            pos: snapshot.map_position,
            death_pos: if died { character.bloodstain.as_ref().and_then(maps::player_position) } else { None },
            gained: diff.gained.clone(),
            lost: diff.lost.clone(),
        };
        let kept = if prev.as_ref().is_none_or(|p| journal::changed(p, &record)) {
            if let Err(e) = journal::append(&file, &record) {
                eprintln!("journal: {e}");
            }
            record
        } else {
            prev.unwrap_or(record)
        };
        self.journal_last = Some((file, kept));
    }

    /// History of the current character.
    pub fn journal(&self) -> Vec<journal::Record> {
        self.journal_last.as_ref().map(|(f, _)| journal::read(f)).unwrap_or_default()
    }

    /// Starts watching the selected save's folder, unless it is already watched.
    pub fn watch(&mut self, app: &AppHandle) -> Result<(), String> {
        let Some(path) = self.save_path() else {
            return Ok(());
        };
        if self.watcher.as_ref().is_some_and(|(p, _)| *p == path) {
            return Ok(());
        }
        let handle = app.clone();
        let target = path.clone();
        // The game writes the file in several passes: wait until it has been quiet for 400 ms.
        let mut debouncer = new_debouncer(Duration::from_millis(400), move |res: DebounceEventResult| {
            if res.is_ok_and(|events| events.iter().any(|e| e.path == target)) {
                on_save_written(&handle);
            }
        })
        .map_err(|e| e.to_string())?;
        let dir = path.parent().ok_or("save path has no parent folder")?;
        debouncer.watcher().watch(dir, RecursiveMode::NonRecursive).map_err(|e| e.to_string())?;
        self.watcher = Some((path, debouncer));
        Ok(())
    }
}

fn on_save_written(app: &AppHandle) {
    let state = app.state::<TrackerState>();
    let update = state.lock().unwrap().refresh();
    match update {
        Ok(Some(update)) => {
            let _ = app.emit(UPDATE_EVENT, update);
        }
        Ok(None) => {}
        Err(e) => eprintln!("reload failed: {e}"),
    }
}

/// Retries while a slot checksum does not match, which means the game is still writing.
fn load_consistent(path: &Path) -> Result<SaveFile, String> {
    let mut last = Err(String::new());
    for attempt in 0..6 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(250));
        }
        last = SaveFile::load(path).map_err(|e| e.to_string());
        if last.as_ref().is_ok_and(|s| s.characters.iter().all(|c| c.checksum_ok)) {
            break;
        }
    }
    last
}
