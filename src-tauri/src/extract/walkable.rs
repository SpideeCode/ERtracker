//! Walkable ground samples for route finding: positions of enemies and assets
//! from the map layouts, projected onto the world maps and thinned out.
//!
//! walkable.bin: "ERWK", u32 version, u32 count, then per node
//! map u8, area u8, grid x u8, grid z u8, x f32, y f32, z f32 (16 bytes).

use super::bosses::tile_pos;
use super::world::{MapId, Projector};
use super::Result;
use std::collections::HashSet;

/// Grid used to thin samples out: metres horizontally, metres vertically.
const CELL: f32 = 4.0;
const LAYER: f32 = 3.0;

pub fn build(ground: &[(String, [f32; 3])], projector: &Projector) -> Result<Vec<u8>> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(12 + ground.len() * 4);
    out.extend_from_slice(b"ERWK");
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    let mut count: u32 = 0;
    for (map, local) in ground {
        let Some(tile) = tile_pos(map, *local) else { continue };
        let Some(pos) = projector.project(tile) else { continue };
        let map_index: u8 = match pos.map {
            MapId::M00 => 0,
            MapId::M01 => 1,
            MapId::M10 => 2,
            MapId::M11 => 3,
        };
        // Areas from the map file name: a dungeon's interior keeps its own block.
        let parts: Vec<u8> = map.trim_start_matches('m').split('_').filter_map(|p| p.parse().ok()).collect();
        let (area, gx, gz) = match parts.as_slice() {
            [a, x, z, ..] if *a == 60 || *a == 61 => (*a, tile.grid_x, tile.grid_z),
            [a, x, z, ..] => (*a, *x, *z),
            _ => continue,
        };
        let key = (
            map_index,
            area,
            if area == 60 || area == 61 { 0 } else { gx },
            (pos.x / CELL).floor() as i32,
            (pos.y / CELL).floor() as i32,
            (pos.height / LAYER).floor() as i32,
        );
        if !seen.insert(key) {
            continue;
        }
        out.extend_from_slice(&[map_index, area, gx, gz]);
        out.extend_from_slice(&pos.x.to_le_bytes());
        out.extend_from_slice(&pos.y.to_le_bytes());
        out.extend_from_slice(&pos.height.to_le_bytes());
        count += 1;
    }
    out[8..12].copy_from_slice(&count.to_le_bytes());
    Ok(out)
}
