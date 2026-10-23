//! Game positions -> pixel positions on the world map images.
//!
//! Overworld tiles (m60 = Lands Between, m61 = Realm of Shadow) are 256 m wide
//! and their local origin is the tile centre. The level-0 map image is 1 px per
//! metre and is offset from the world origin by a fixed amount. Dungeons and the
//! underground have their own frames, which WorldMapLegacyConvParam translates
//! onto the overworld (sometimes through another dungeon first).

use super::{param, Result, TilePos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const TILE_METRES: f32 = 256.0;
const OFFSET_X: f32 = -7168.0;
const OFFSET_Y: f32 = 16640.0;

/// Underground blocks (m12_XX) drawn on the underground map rather than the surface.
const UNDERGROUND_BLOCKS: [u8; 6] = [1, 2, 3, 4, 5, 7];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MapId {
    /// Lands Between
    M00,
    /// Underground (Siofra, Ainsel, Deeproot, Lake of Rot, Mohgwyn)
    M01,
    /// Realm of Shadow
    M10,
    /// Realm of Shadow underground
    M11,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct MapPos {
    pub map: MapId,
    /// Pixels on the level-0 image, from the top-left corner.
    pub x: f32,
    pub y: f32,
    /// Height in metres in the destination frame.
    pub height: f32,
}

struct Conv {
    src: TilePos,
    dst: TilePos,
    rank: u8,
}

pub struct Projector {
    by_block: HashMap<(u8, u8, u8), Vec<Conv>>,
}

impl Projector {
    pub fn new(legacy_conv_param: &[u8]) -> Result<Self> {
        let mut by_block: HashMap<(u8, u8, u8), Vec<Conv>> = HashMap::new();
        for r in param::Param::read(legacy_conv_param)?.iter() {
            let src = TilePos::read(&r, 4);
            let dst = TilePos::read(&r, 20);
            let base = r.u8(36) & 1 != 0;
            // Prefer the game's own base point, then rows landing directly on the
            // overworld, then rows that spell out their anchors.
            let overworld = dst.area == 60 || dst.area == 61;
            let rank = u8::from(!base) * 8
                + u8::from(!overworld) * 4
                + u8::from(src.x == 0.0 && src.z == 0.0) * 2
                + u8::from(dst.x == 0.0 && dst.z == 0.0);
            by_block.entry((src.area, src.grid_x, src.grid_z)).or_default().push(Conv { src, dst, rank });
        }
        for rows in by_block.values_mut() {
            rows.sort_by_key(|c| c.rank);
        }
        Ok(Self { by_block })
    }

    pub fn project(&self, pos: TilePos) -> Option<MapPos> {
        let map = match pos.area {
            61 => MapId::M10,
            12 if UNDERGROUND_BLOCKS.contains(&pos.grid_x) => MapId::M01,
            _ => MapId::M00,
        };
        let (world, height, area) = self.to_overworld(pos, 0)?;
        let map = if area == 61 && map == MapId::M00 { MapId::M10 } else { map };
        Some(MapPos { map, x: world.0 + OFFSET_X, y: OFFSET_Y - world.1, height })
    }

    /// -> ((world x, world z), height, overworld area)
    fn to_overworld(&self, p: TilePos, depth: u8) -> Option<((f32, f32), f32, u8)> {
        if p.area == 60 || p.area == 61 {
            let x = p.grid_x as f32 * TILE_METRES + TILE_METRES / 2.0 + p.x;
            let z = p.grid_z as f32 * TILE_METRES + TILE_METRES / 2.0 + p.z;
            return Some(((x, z), p.y, p.area));
        }
        if depth > 4 {
            return None;
        }
        let rows = self
            .by_block
            .get(&(p.area, p.grid_x, p.grid_z))
            .or_else(|| self.by_block.get(&(p.area, p.grid_x, 0)))?;
        // Some rows lead to a dungeon with no way onward: try the next one.
        rows.iter().find_map(|c| {
            let next = TilePos {
                area: c.dst.area,
                grid_x: c.dst.grid_x,
                grid_z: c.dst.grid_z,
                x: p.x - c.src.x + c.dst.x,
                y: p.y - c.src.y + c.dst.y,
                z: p.z - c.src.z + c.dst.z,
            };
            self.to_overworld(next, depth + 1)
        })
    }
}
