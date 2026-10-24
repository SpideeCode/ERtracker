pub mod bnd4;
pub mod event_flags;
mod reader;
pub mod slot;

use md5::{Digest, Md5};
use reader::Reader;
use serde::Serialize;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Read;
use std::ops::Range;
use std::path::Path;

pub use slot::Position;

#[derive(Debug)]
pub struct SaveError(String);

impl SaveError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SaveError {}

pub type Result<T> = std::result::Result<T, SaveError>;

#[derive(Clone, Debug, Serialize)]
pub struct Character {
    pub slot: usize,
    pub name: String,
    pub level: u32,
    pub seconds_played: u32,
    pub version: u32,
    /// False while the game is in the middle of writing this slot.
    pub checksum_ok: bool,
    pub position: Option<Position>,
    pub stats: [u32; 8],
    pub inventory: Vec<u32>,
    pub deaths: u32,
    pub bloodstain: Option<Position>,
    pub runes: u32,
    /// Absolute offset of the held runes (u32, little-endian) in the save file.
    pub runes_offset: usize,
    #[serde(skip)]
    flags: Range<usize>,
    /// Absolute range of the slot data covered by the MD5 stored just before it.
    #[serde(skip)]
    body: Range<usize>,
}

pub struct SaveFile {
    data: Vec<u8>,
    pub characters: Vec<Character>,
}

const MD5_SIZE: usize = 0x10;

impl SaveFile {
    /// Reads without locking the file, so the game can keep writing to it.
    pub fn load(path: &Path) -> Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
            options.share_mode(0x1 | 0x2 | 0x4);
        }
        let mut data = Vec::new();
        options
            .open(path)
            .and_then(|mut f| f.read_to_end(&mut data))
            .map_err(|e| SaveError::new(format!("cannot read {}: {e}", path.display())))?;
        Self::parse(data)
    }

    pub fn parse(data: Vec<u8>) -> Result<Self> {
        let entries = bnd4::entries(&data)?;
        let summary = entries
            .iter()
            .find(|e| e.name.ends_with("10"))
            .map(|e| profile_summary(&data[e.offset..e.offset + e.size]))
            .transpose()?;

        let mut characters = Vec::new();
        for (slot, entry) in entries.iter().take(10).enumerate() {
            if entry.size < MD5_SIZE + slot::SLOT_SIZE {
                continue;
            }
            let (active, seconds_played) = summary.as_ref().map_or((true, 0), |s| s[slot]);
            if !active {
                continue;
            }
            let body_start = entry.offset + MD5_SIZE;
            let body = &data[body_start..body_start + slot::SLOT_SIZE];
            let Some(info) = slot::parse(body)
                .map_err(|e| SaveError::new(format!("slot {slot}: {e}")))?
            else {
                continue;
            };
            let checksum_ok = Md5::digest(body).as_slice() == &data[entry.offset..body_start];
            let flags_start = body_start + info.flags_offset;
            characters.push(Character {
                slot,
                name: info.name,
                level: info.level,
                seconds_played,
                version: info.version,
                checksum_ok,
                position: info.position,
                stats: info.stats,
                deaths: info.deaths,
                bloodstain: info.bloodstain,
                runes: info.runes,
                runes_offset: body_start + info.runes_offset,
                body: body_start..body_start + slot::SLOT_SIZE,
                inventory: info.inventory,
                flags: flags_start..flags_start + event_flags::EVENT_FLAGS_SIZE,
            });
        }
        Ok(Self { data, characters })
    }

    pub fn character(&self, slot: usize) -> Option<&Character> {
        self.characters.iter().find(|c| c.slot == slot)
    }

    pub fn flags(&self, character: &Character) -> &[u8] {
        &self.data[character.flags.clone()]
    }

    pub fn flag(&self, character: &Character, id: u32) -> Option<bool> {
        event_flags::get(self.flags(character), id)
    }
}

/// USER_DATA010 → per slot: (active, seconds played).
fn profile_summary(entry: &[u8]) -> Result<[(bool, u32); 10]> {
    // MD5 (0x10), version (4), Steam id (8), settings (0x140), menu system (0x1808)
    const ACTIVE: usize = 0x10 + 4 + 8 + 0x140 + 0x1808;
    const PROFILES: usize = ACTIVE + 10;
    const PROFILE_SIZE: usize = 0x24C;
    let mut out = [(false, 0); 10];
    for (i, slot) in out.iter_mut().enumerate() {
        let active = Reader::at(entry, ACTIVE + i).u8()? != 0;
        // name (0x20) + terminator (2) + level (4), then seconds played
        let seconds = Reader::at(entry, PROFILES + i * PROFILE_SIZE + 0x26).u32()?;
        *slot = (active, seconds);
    }
    Ok(out)
}

/// Highest rune count the game displays.
pub const MAX_RUNES: u32 = 999_999_999;

#[derive(Serialize)]
pub struct RuneEdit {
    pub backup: std::path::PathBuf,
    pub offset: usize,
    pub before: u32,
    pub after: u32,
}

/// Sets the held runes of a character in a save file, after copying the file to
/// a timestamped backup. The slot's MD5 is recomputed so the game accepts it.
/// The game must not be running: it would overwrite the file at its next save.
pub fn write_runes(path: &Path, slot: usize, value: u32) -> Result<RuneEdit> {
    let value = value.min(MAX_RUNES);
    let save = SaveFile::load(path)?;
    let character = save.character(slot).ok_or_else(|| SaveError::new(format!("no character in slot {slot}")))?;
    if !character.checksum_ok {
        return Err(SaveError::new("the save is being written by the game: close it first"));
    }
    let (offset, body, before) = (character.runes_offset, character.body.clone(), character.runes);

    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let backup = path.with_file_name(format!(
        "{}.ertracker-{stamp}.bak",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("ER0000.sl2")
    ));
    std::fs::copy(path, &backup).map_err(|e| SaveError::new(format!("backup failed, nothing was changed: {e}")))?;

    let mut data = save.data;
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    let digest = Md5::digest(&data[body.clone()]);
    data[body.start - MD5_SIZE..body.start].copy_from_slice(&digest);

    // Write next to the save, then swap, so a crash never leaves a half-written file.
    let tmp = path.with_extension("ertracker-tmp");
    std::fs::write(&tmp, &data).map_err(|e| SaveError::new(format!("cannot write: {e}")))?;
    std::fs::rename(&tmp, path).map_err(|e| SaveError::new(format!("cannot replace the save: {e}")))?;

    let check = SaveFile::load(path)?;
    let after = check.character(slot).map(|c| (c.runes, c.checksum_ok));
    if after != Some((value, true)) {
        std::fs::copy(&backup, path).map_err(|e| SaveError::new(format!("verification failed and restore failed: {e}")))?;
        return Err(SaveError::new("verification failed: the backup was restored"));
    }
    Ok(RuneEdit { backup, offset, before, after: value })
}
