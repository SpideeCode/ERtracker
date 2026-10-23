//! MSB map layouts (map/mapstudio/*.msb.dcx): only the enemy parts are read.

use super::{ExtractError, Result};

const PART_ENEMY: i32 = 2;

pub struct Enemy {
    pub entity_id: u32,
    pub npc_param: i32,
    /// Local position in the map tile.
    pub pos: [f32; 3],
}

fn i32_at(d: &[u8], at: usize) -> Result<i32> {
    d.get(at..at + 4)
        .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| ExtractError::new("MSB read out of bounds"))
}

fn i64_at(d: &[u8], at: usize) -> Result<i64> {
    d.get(at..at + 8)
        .map(|b| i64::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| ExtractError::new("MSB read out of bounds"))
}

fn f32_at(d: &[u8], at: usize) -> Result<f32> {
    Ok(f32::from_bits(i32_at(d, at)? as u32))
}

/// Param lists follow each other: models, events, points, routes, layers, parts.
/// Each is: version, count (+1), name offset, `count - 1` entry offsets, next list offset.
fn param_lists(d: &[u8]) -> Result<Vec<Vec<usize>>> {
    let mut lists = Vec::new();
    let mut at = 0x10;
    for _ in 0..6 {
        let count = i32_at(d, at + 4)?.max(1) as usize;
        let entries = (0..count - 1).map(|i| Ok(i64_at(d, at + 16 + i * 8)? as usize)).collect::<Result<Vec<_>>>()?;
        lists.push(entries);
        at = i64_at(d, at + 16 + (count - 1) * 8)? as usize;
        if at == 0 {
            break;
        }
    }
    Ok(lists)
}

pub fn enemies(d: &[u8]) -> Result<Vec<Enemy>> {
    if !d.starts_with(b"MSB ") {
        return Err(ExtractError::new("not an MSB"));
    }
    let lists = param_lists(d)?;
    let Some(parts) = lists.get(5) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for &p in parts {
        if i32_at(d, p + 0x0C)? != PART_ENEMY {
            continue;
        }
        let entity = p + i64_at(d, p + 0x60)? as usize;
        let type_data = p + i64_at(d, p + 0x68)? as usize;
        out.push(Enemy {
            entity_id: i32_at(d, entity)? as u32,
            npc_param: i32_at(d, type_data + 0x0C)?,
            pos: [f32_at(d, p + 0x20)?, f32_at(d, p + 0x24)?, f32_at(d, p + 0x28)?],
        });
    }
    Ok(out)
}
