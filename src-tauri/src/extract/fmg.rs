//! FMG text tables (item names, NPC names, place names...) from msg/<lang>/*.msgbnd.dcx.

use super::archive::Archive;
use super::{binder, ExtractError, Result};
use std::collections::HashMap;

/// Text id -> string.
pub type Fmg = HashMap<i32, String>;

/// Parses a 64-bit (Elden Ring) FMG.
pub fn parse(data: &[u8]) -> Result<Fmg> {
    let rd = |at: usize, n: usize| data.get(at..at + n).ok_or_else(|| ExtractError::new("truncated FMG"));
    let i32_at = |at: usize| -> Result<i32> { Ok(i32::from_le_bytes(rd(at, 4)?.try_into().unwrap())) };
    let i64_at = |at: usize| -> Result<i64> { Ok(i64::from_le_bytes(rd(at, 8)?.try_into().unwrap())) };
    if data.get(1) != Some(&0) || data.get(2) != Some(&2) {
        return Err(ExtractError::new("unsupported FMG version"));
    }
    let groups = i32_at(0x0C)? as usize;
    let offsets = i64_at(0x18)? as usize;
    let mut out = HashMap::new();
    for g in 0..groups {
        let at = 0x28 + g * 16;
        let (index, first, last) = (i32_at(at)? as usize, i32_at(at + 4)?, i32_at(at + 8)?);
        for (k, id) in (first..=last).enumerate() {
            let s = i64_at(offsets + (index + k) * 8)? as usize;
            if s == 0 {
                continue;
            }
            let units: Vec<u16> = data[s..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&u| u != 0)
                .collect();
            let text = String::from_utf16_lossy(&units);
            if !text.is_empty() && text != "[ERROR]" {
                out.insert(id, text);
            }
        }
    }
    Ok(out)
}

/// All tables of one language, merged across the base game and DLC message
/// binders (later binders override earlier ones). Keyed by table name, e.g. "NpcName".
pub fn load_language(data0: &Archive, lang: &str) -> Result<HashMap<String, Fmg>> {
    let mut tables: HashMap<String, Fmg> = HashMap::new();
    for bnd in ["item", "item_dlc01", "item_dlc02", "menu", "menu_dlc01", "menu_dlc02"] {
        let path = format!("/msg/{lang}/{bnd}.msgbnd.dcx");
        if !data0.contains(&path) {
            continue;
        }
        for (_, name, data) in binder::read_owned(&data0.read(&path)?)? {
            let file = name.rsplit(['\\', '/']).next().unwrap_or(&name);
            let table = file.trim_end_matches(".fmg");
            // DLC tables are named e.g. "NpcName_dlc01": merge them into the base table.
            let base = table.split("_dlc").next().unwrap_or(table).to_string();
            tables.entry(base).or_default().extend(parse(&data)?);
        }
    }
    Ok(tables)
}
