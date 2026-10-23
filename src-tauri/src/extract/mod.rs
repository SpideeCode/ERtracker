//! Reads game data from the player's own Elden Ring install.
//! Nothing extracted here is shipped with ERtracker.

pub mod archive;
pub mod binder;
pub mod bosses;
pub mod catalog;
pub mod dcx;
pub mod fmg;
pub mod items;
pub mod msb;
pub mod oodle;
pub mod param;
pub mod regulation;
pub mod tiles;
pub mod world;
pub mod build;

use std::fmt;

#[derive(Debug)]
pub struct ExtractError(String);

impl ExtractError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ExtractError {}

impl From<std::io::Error> for ExtractError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ExtractError>;

/// A position in a map tile: `m{area}_{grid_x}_{grid_z}_00` plus local coordinates.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct TilePos {
    pub area: u8,
    pub grid_x: u8,
    pub grid_z: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl TilePos {
    /// Map block the position belongs to, e.g. "m30_04_00" (a catacomb) or "m60_42_36".
    pub fn block(&self) -> String {
        format!("m{:02}_{:02}_{:02}", self.area, self.grid_x, self.grid_z)
    }

    pub(crate) fn read(row: &param::Row, at: usize) -> Self {
        Self {
            area: row.u8(at),
            grid_x: row.u8(at + 1),
            grid_z: row.u8(at + 2),
            x: row.f32(at + 4),
            y: row.f32(at + 8),
            z: row.f32(at + 12),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Grace {
    pub row: i32,
    pub flag: u32,
    pub text_id: i32,
    pub pos: TilePos,
}

/// BonfireWarpParam (layout for regulation version >= 1.12.1).
pub fn graces(bonfire_warp: &[u8]) -> Result<Vec<Grace>> {
    Ok(param::Param::read(bonfire_warp)?
        .iter()
        .filter(|r| r.u32(4) != 0)
        .map(|r| Grace { row: r.id, flag: r.u32(4), text_id: r.i32(48), pos: TilePos::read(&r, 32) })
        .collect())
}

#[derive(Debug, serde::Serialize)]
pub struct BossArena {
    pub row: i32,
    pub flag: u32,
    pub pos: TilePos,
}

/// GameAreaParam: one row per boss arena, with its defeat flag and position.
pub fn bosses(game_area: &[u8]) -> Result<Vec<BossArena>> {
    Ok(param::Param::read(game_area)?
        .iter()
        .filter(|r| r.u32(68) != 0)
        .map(|r| BossArena {
            row: r.id,
            flag: r.u32(68),
            pos: TilePos {
                area: r.u8(84),
                grid_x: r.u8(85),
                grid_z: r.u8(86),
                x: r.f32(72),
                y: r.f32(76),
                z: r.f32(80),
            },
        })
        .collect())
}
