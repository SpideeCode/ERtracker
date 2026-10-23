//! Sequential parser for a character slot (USER_DATA000..009, after the MD5).
//!
//! Most sections have a variable length, so the event flags and the player
//! position can only be found by walking every section before them. Layout
//! from ER-Save-Editor (MIT/Apache-2.0) and er-save-manager (MIT, commit
//! bde1359), checked against a v260 save (game 1.17).

use super::event_flags::EVENT_FLAGS_SIZE;
use super::reader::Reader;
use super::{Result, SaveError};
use serde::Serialize;

pub const SLOT_SIZE: usize = 0x280000;

#[derive(Clone, Debug, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Game map name, e.g. "m60_42_36_00" or "m10_00_00_00".
    pub map: String,
}

#[derive(Clone, Debug)]
pub struct SlotInfo {
    pub version: u32,
    pub name: String,
    pub level: u32,
    /// Offset of the event flag bitfield inside the slot data.
    pub flags_offset: usize,
    pub position: Option<Position>,
    /// Vigor, mind, endurance, strength, dexterity, intelligence, faith, arcane.
    pub stats: [u32; 8],
    /// Runes held, and where that u32 lives in the slot data.
    pub runes: u32,
    pub runes_offset: usize,
    /// Item ids held or in the storage chest, with the category in the top nibble
    /// (0 weapon, 1 armour, 2 talisman, 4 goods, 8 ash of war). Weapon ids carry
    /// their affinity and upgrade level in the low digits.
    pub inventory: Vec<u32>,
}

fn map_name(id: &[u8]) -> String {
    format!("m{:02}_{:02}_{:02}_{:02}", id[3], id[2], id[1], id[0])
}

fn counted<'a>(r: &mut Reader<'a>, what: &str, max: u32) -> Result<usize> {
    let n = r.u32()?;
    if n > max {
        return Err(SaveError::new(format!("{what} count {n} at {:#x} is implausible", r.pos - 4)));
    }
    Ok(n as usize)
}

/// Inventory: count, `common` (handle, quantity, index) entries, count, `key` entries, two counters.
fn read_inventory(
    r: &mut Reader,
    common: usize,
    key: usize,
    handles: &std::collections::HashMap<u32, u32>,
    out: &mut Vec<u32>,
) -> Result<()> {
    for slots in [common, key] {
        r.skip(4)?;
        for _ in 0..slots {
            let handle = r.u32()?;
            let quantity = r.u32()?;
            r.skip(4)?;
            if handle == 0 || handle == u32::MAX || quantity == 0 {
                continue;
            }
            // Equipment resolves through the GaItem map; goods and talismans encode their id in the handle.
            let id = handles.get(&handle).copied().or_else(|| match handle & 0xF000_0000 {
                0xB000_0000 => Some(0x4000_0000 | (handle & 0x0FFF_FFFF)),
                0xA000_0000 => Some(0x2000_0000 | (handle & 0x0FFF_FFFF)),
                _ => None,
            });
            if let Some(id) = id.filter(|&id| id != 0 && id != u32::MAX) {
                out.push(id);
            }
        }
    }
    r.skip(8)
}

/// Returns `None` for an empty slot.
pub fn parse(data: &[u8]) -> Result<Option<SlotInfo>> {
    let mut r = Reader::new(data);
    let version = r.u32()?;
    if version == 0 {
        return Ok(None);
    }
    r.skip(4 + 0x18)?; // map id + unknown

    // GaItem map: record size depends on the handle type.
    let gaitems = if version <= 81 { 0x13FE } else { 0x1400 };
    let mut handles = std::collections::HashMap::new();
    for _ in 0..gaitems {
        let handle = r.u32()?;
        let item_id = r.u32()?;
        if handle != 0 {
            handles.insert(handle, item_id);
        }
        let kind = handle & 0xF000_0000;
        if handle != 0 && kind != 0xC000_0000 {
            r.skip(8)?;
            if kind == 0x8000_0000 {
                r.skip(5)?; // weapon: ash of war handle + 1 byte
            }
        }
    }

    // PlayerGameData (0x1B0)
    let pgd = r.pos;
    let level = Reader::at(data, pgd + 0x60).u32()?;
    let name = Reader::at(data, pgd + 0x94).wstr(16)?;
    let runes_offset = pgd + 0x64;
    let runes = Reader::at(data, runes_offset).u32()?;
    let mut stats = [0u32; 8];
    for (i, s) in stats.iter_mut().enumerate() {
        *s = Reader::at(data, pgd + 0x34 + i * 4).u32()?;
    }
    r.skip(0x1B0)?;

    r.skip(13 * 0x10)?; // SP effects
    r.skip(0x58 + 0x1C + 0x58 + 0x58)?; // equip index, arm style, item ids, gaitem handles
    let mut inventory = Vec::new();
    read_inventory(&mut r, 0xA80, 0x180, &handles, &mut inventory)?; // held
    r.skip(14 * 8 + 4)?; // equipped spells
    r.skip(10 * 8 + 4 + 6 * 8 + 8)?; // quick items + pouch
    r.skip(6 * 4)?; // equipped gestures
    let projectiles = counted(&mut r, "projectile", 0x1000)?;
    r.skip(projectiles * 8)?;
    r.skip(0x9C)?; // equipped armaments and items
    r.skip(12)?; // equipped physics
    r.skip(0x12F)?; // face data
    read_inventory(&mut r, 0x780, 0x80, &handles, &mut inventory)?; // storage chest
    inventory.sort_unstable();
    inventory.dedup();
    r.skip(64 * 4)?; // gestures
    let regions = counted(&mut r, "region", 0x1000)?;
    r.skip(regions * 4)?;
    r.skip(0x28)?; // torrent
    r.skip(1)?; // control byte
    r.skip(0x44)?; // blood stain
    r.skip(8)?;
    r.skip(4)?; // menu profile save/load
    let menu = counted(&mut r, "menu save data", 0x10000)?;
    r.skip(menu)?;
    r.skip(0x34)?; // trophy equip data
    r.skip(8 + 7000 * 16)?; // gaitem game data
    r.skip(4)?; // tutorial data
    let tutorial = counted(&mut r, "tutorial data", 0x10000)?;
    r.skip(tutorial)?;
    r.skip(3 + 26)?; // GameMan bytes, deaths, last grace, timers...

    let flags_offset = r.pos;
    r.skip(EVENT_FLAGS_SIZE + 1)?;

    // FieldArea, WorldArea, WorldGeomMan x2, RendMan: size-prefixed blobs.
    let position = (|| -> Result<Position> {
        for _ in 0..5 {
            let size = r.i32()?;
            if !(0..0x100000).contains(&size) {
                return Err(SaveError::new("bad world section size"));
            }
            r.skip(size as usize)?;
        }
        let (x, y, z) = (r.f32()?, r.f32()?, r.f32()?);
        let map = map_name(r.bytes(4)?);
        Ok(Position { x, y, z, map })
    })()
    .ok();

    Ok(Some(SlotInfo { version, name, level, flags_offset, position, stats, runes, runes_offset, inventory }))
}
