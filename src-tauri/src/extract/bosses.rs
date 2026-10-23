//! Boss sheets: where each boss stands (MSB enemy whose entity id is the defeat
//! flag), what it resists (NpcParam), what it gives (GameAreaParam runes,
//! remembrances), and French names from the game's own text.

use super::archive::Archive;
use super::fmg::Fmg;
use super::param::Param;
use super::{dcx, msb, ExtractError, Result, TilePos};
use crate::progress;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Serialize)]
pub struct Boss {
    pub flag: u32,
    pub npc_param: Option<i32>,
    /// Map the boss was found in, e.g. "m10_00_00_00".
    pub msb: Option<String>,
    pub runes: Option<u32>,
    /// % of damage absorbed: standard, slash, strike, pierce, magic, fire, lightning, holy.
    pub absorption: Option<[i32; 8]>,
    /// Build-up thresholds (999 = immune): poison, scarlet rot, bleed, death, frost, sleep, madness.
    pub resistance: Option<[u16; 7]>,
    pub legendary: bool,
    pub remembrance: Option<Text>,
}

#[derive(Serialize, Clone)]
pub struct Text {
    pub en: String,
    pub fr: Option<String>,
}

/// Where a boss can be placed on the map, before projection.
pub struct BossPlacement {
    pub flag: u32,
    pub pos: TilePos,
    /// Map block the boss lives in ("m40_00_00"), before any tile-size conversion.
    pub block: String,
}

const ABSORPTION_OFFSETS: [usize; 8] = [0x1a4, 0x1a8, 0x1ac, 0x1b0, 0x1b4, 0x1b8, 0x1bc, 0x1c0];
const RESIST_OFFSETS: [usize; 7] = [0x108, 0x10a, 0x10c, 0x10e, 0x1e0, 0x126, 0x128];

/// "m60_42_36_01" -> tile position in the 256 m grid; larger tiers cover 2^tier cells.
pub fn tile_pos(map: &str, local: [f32; 3]) -> Option<TilePos> {
    let n: Vec<u32> = map.trim_start_matches('m').split('_').map(|p| p.parse().ok()).collect::<Option<_>>()?;
    let [area, gx, gz, last] = n.as_slice() else { return None };
    let (area, gx, gz, last) = (*area as u8, *gx as u8, *gz as u8, *last);
    if (area == 60 || area == 61) && (1..=2).contains(&last) {
        let size = 256.0 * (1 << last) as f32;
        let wx = gx as f32 * size + size / 2.0 + local[0];
        let wz = gz as f32 * size + size / 2.0 + local[2];
        let (cx, cz) = ((wx / 256.0).floor(), (wz / 256.0).floor());
        return Some(TilePos {
            area,
            grid_x: cx as u8,
            grid_z: cz as u8,
            x: wx - cx * 256.0 - 128.0,
            y: local[1],
            z: wz - cz * 256.0 - 128.0,
        });
    }
    Some(TilePos { area, grid_x: gx, grid_z: gz, x: local[0], y: local[1], z: local[2] })
}

pub struct Extracted {
    pub bosses: Vec<Boss>,
    pub placements: Vec<BossPlacement>,
    /// Every treasure event, as (map name, treasure): read in the same pass over the maps.
    pub treasures: Vec<(String, msb::Treasure)>,
    /// Positions of enemies and assets, as (map name, position): ground the player can stand on.
    pub ground: Vec<(String, [f32; 3])>,
}

pub fn extract(
    game: &Path,
    params: &HashMap<String, Vec<u8>>,
    en: &HashMap<String, Fmg>,
    fr: &HashMap<String, Fmg>,
    mut progress: impl FnMut(usize, usize),
) -> Result<Extracted> {
    let get = |name: &str| params.get(name).ok_or_else(|| ExtractError::new(format!("{name} missing")));
    let db = progress::db();
    let db_flags: HashSet<u32> = db.entries.iter().filter(|e| e.category == "boss").map(|e| e.flag).collect();
    let arenas = super::bosses(get("GameAreaParam")?)?;
    let runes: HashMap<u32, u32> = Param::read(get("GameAreaParam")?)?
        .iter()
        .filter(|r| r.u32(68) != 0)
        .map(|r| (r.u32(68), r.u32(4)))
        .collect();
    let mut flags = db_flags.clone();
    flags.extend(arenas.iter().map(|a| a.flag));

    // MSB enemies whose entity id is a defeat flag. The first map found wins
    // (base maps are listed before their _99 variants).
    let mut found: HashMap<u32, (String, i32, [f32; 3])> = HashMap::new();
    let mut treasures = Vec::new();
    let mut ground = Vec::new();
    let archives: Vec<(Archive, Vec<(String, String)>)> = ["Data2", "DLC"]
        .iter()
        .map(|name| {
            let a = Archive::open(game, name)?;
            let paths = a.msb_paths();
            Ok((a, paths))
        })
        .collect::<Result<_>>()?;
    let total: usize = archives.iter().map(|(_, p)| p.len()).sum();
    let mut done = 0;
    for (archive, paths) in &archives {
        for (map, path) in paths {
            let raw = dcx::decompress(&archive.read(path)?)?;
            treasures.extend(msb::treasures(&raw)?.into_iter().map(|t| (map.clone(), t)));
            // Enemies, player spawns and assets sit on walkable ground; "_99" maps are world-state variants.
            if !map.ends_with("_99") {
                ground.extend(
                    msb::part_positions(&raw)?
                        .into_iter()
                        .filter(|(kind, _)| matches!(kind, 2 | 4 | 9 | 10 | 13))
                        .map(|(_, p)| (map.clone(), p)),
                );
            }
            for e in msb::enemies(&raw)? {
                if flags.contains(&e.entity_id) && !found.contains_key(&e.entity_id) {
                    found.insert(e.entity_id, (map.clone(), e.npc_param, e.pos));
                }
            }
            done += 1;
            if done % 20 == 0 || done == total {
                progress(done, total);
            }
        }
    }

    let npc: HashMap<i32, Vec<u8>> = {
        let p = Param::read(get("NpcParam")?)?;
        let size = p.row_size();
        let data = get("NpcParam")?;
        p.rows.iter().filter_map(|&(id, off)| Some((id, data.get(off..off + size)?.to_vec()))).collect()
    };
    let f32_at = |d: &[u8], at: usize| f32::from_le_bytes(d[at..at + 4].try_into().unwrap());
    let u16_at = |d: &[u8], at: usize| u16::from_le_bytes(d[at..at + 2].try_into().unwrap());

    let meta: serde_json::Value = serde_json::from_str(include_str!("../../../data/boss_meta.json")).unwrap_or_default();
    let goods_en2fr = reverse(en.get("GoodsName"), fr.get("GoodsName"));

    let mut bosses = Vec::new();
    for entry in db.entries.iter().filter(|e| e.category == "boss") {
        let hit = found.get(&entry.flag);
        let row = hit.and_then(|(_, id, _)| npc.get(id));
        let remembrance = meta["legendary"][&entry.name].as_str().map(|en| Text {
            en: en.to_string(),
            fr: goods_en2fr.get(&en.to_lowercase()).cloned(),
        });
        bosses.push(Boss {
            flag: entry.flag,
            npc_param: hit.map(|(_, id, _)| *id),
            msb: hit.map(|(m, _, _)| m.clone()),
            runes: runes.get(&entry.flag).copied().filter(|&r| r > 0),
            absorption: row.map(|d| ABSORPTION_OFFSETS.map(|o| ((1.0 - f32_at(d, o)) * 100.0).round() as i32)),
            resistance: row.map(|d| RESIST_OFFSETS.map(|o| u16_at(d, o))),
            legendary: remembrance.is_some(),
            remembrance,
        });
    }

    // Exact enemy positions first, arena positions for the rest.
    let mut placements: Vec<BossPlacement> = found
        .iter()
        .filter_map(|(flag, (map, _, pos))| {
            Some(BossPlacement { flag: *flag, pos: tile_pos(map, *pos)?, block: map.get(..9)?.to_string() })
        })
        .collect();
    let placed: HashSet<u32> = placements.iter().map(|p| p.flag).collect();
    placements.extend(
        arenas
            .into_iter()
            .filter(|a| !placed.contains(&a.flag))
            .map(|a| BossPlacement { flag: a.flag, block: a.pos.block(), pos: a.pos }),
    );
    Ok(Extracted { bosses, placements, treasures, ground })
}

/// Lower-cased English text -> French text, for tables sharing ids.
pub fn reverse(en: Option<&Fmg>, fr: Option<&Fmg>) -> HashMap<String, String> {
    let (Some(en), Some(fr)) = (en, fr) else { return HashMap::new() };
    en.iter().filter_map(|(id, e)| Some((e.to_lowercase(), fr.get(id)?.clone()))).collect()
}
