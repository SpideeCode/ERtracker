//! Play history. The save only holds the current state, so every save the game
//! writes is summarised into a JSON-lines file per character: deaths, where the
//! last death happened, where the player stands, what was just unlocked.

use crate::extract::world::MapPos;
use crate::save::Character;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    /// Milliseconds since the Unix epoch.
    pub t: u64,
    pub deaths: u32,
    pub level: u32,
    pub runes: u32,
    /// In-game play time, seconds.
    pub played: u32,
    pub pos: Option<MapPos>,
    /// Where the latest death happened, when deaths went up since the previous record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_pos: Option<MapPos>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gained: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lost: Vec<String>,
}

/// One file per save folder, slot and character name.
pub fn file(dir: &Path, save: &Path, character: &Character) -> PathBuf {
    let account = save.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("save");
    let name: String = character.name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
    dir.join(format!("{account}-slot{}-{name}.jsonl", character.slot + 1))
}

pub fn append(path: &Path, record: &Record) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{}", serde_json::to_string(record).map_err(std::io::Error::other)?)
}

pub fn read(path: &Path) -> Vec<Record> {
    let Ok(f) = std::fs::File::open(path) else { return Vec::new() };
    std::io::BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect()
}

/// The last record of a file, to compare the next save with.
pub fn last(path: &Path) -> Option<Record> {
    read(path).pop()
}

/// Whether a new save is worth a line: something happened or the player moved.
pub fn changed(prev: &Record, next: &Record) -> bool {
    let moved = match (&prev.pos, &next.pos) {
        (Some(a), Some(b)) => a.map != b.map || (a.x - b.x).hypot(a.y - b.y) > 5.0 || (a.height - b.height).abs() > 5.0,
        (a, b) => a.is_some() != b.is_some(),
    };
    moved || prev.deaths != next.deaths || prev.level != next.level || !next.gained.is_empty() || !next.lost.is_empty()
}
