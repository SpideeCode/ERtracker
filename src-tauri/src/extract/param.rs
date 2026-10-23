//! PARAM tables (game data rows) as found in regulation.bin.
//!
//! Elden Ring params use 64-bit data offsets: a 0x40-byte header, then one
//! 0x18-byte row header per row (id, padding, data offset, name offset).

use super::{ExtractError, Result};

pub struct Param<'a> {
    data: &'a [u8],
    pub rows: Vec<(i32, usize)>,
}

pub struct Row<'a> {
    pub id: i32,
    data: &'a [u8],
}

impl<'a> Param<'a> {
    pub fn read(data: &'a [u8]) -> Result<Self> {
        let count = data
            .get(0x0A..0x0C)
            .map(|b| u16::from_le_bytes([b[0], b[1]]) as usize)
            .ok_or_else(|| ExtractError::new("truncated PARAM"))?;
        let rows = (0..count)
            .map(|i| {
                let h = data
                    .get(0x40 + i * 0x18..0x40 + (i + 1) * 0x18)
                    .ok_or_else(|| ExtractError::new("PARAM row header out of bounds"))?;
                let id = i32::from_le_bytes(h[0..4].try_into().unwrap());
                let offset = u64::from_le_bytes(h[8..16].try_into().unwrap()) as usize;
                Ok((id, offset))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { data, rows })
    }

    /// Size of one row, deduced from the gap between the first two rows.
    pub fn row_size(&self) -> usize {
        match self.rows.as_slice() {
            [(_, a), (_, b), ..] => b - a,
            _ => 0,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Row<'a>> + '_ {
        let size = self.row_size();
        self.rows
            .iter()
            .filter_map(move |&(id, offset)| Some(Row { id, data: self.data.get(offset..offset + size)? }))
    }
}

impl Row<'_> {
    pub fn u8(&self, at: usize) -> u8 {
        self.data[at]
    }

    pub fn u32(&self, at: usize) -> u32 {
        u32::from_le_bytes(self.data[at..at + 4].try_into().unwrap())
    }

    pub fn i32(&self, at: usize) -> i32 {
        self.u32(at) as i32
    }

    pub fn f32(&self, at: usize) -> f32 {
        f32::from_le_bytes(self.data[at..at + 4].try_into().unwrap())
    }
}
