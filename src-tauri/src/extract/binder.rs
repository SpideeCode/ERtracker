//! BND4 binders (a container of named files) and BHF4/BDT split binders, where
//! the headers live in one file (.xxxbhd) and the data in another (.xxxbdt).

use super::{dcx, ExtractError, Result};

pub struct Entry {
    pub id: i32,
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

// Format flags, after normalising the bit order.
const IDS: u8 = 0x02;
const NAMES: u8 = 0x04 | 0x08;
const LONG_OFFSETS: u8 = 0x10;
const COMPRESSION: u8 = 0x20;

fn le_u32(d: &[u8], at: usize) -> Result<u32> {
    d.get(at..at + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| ExtractError::new("binder read out of bounds"))
}

fn le_u64(d: &[u8], at: usize) -> Result<u64> {
    d.get(at..at + 8)
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| ExtractError::new("binder read out of bounds"))
}

fn name_at(d: &[u8], at: usize, unicode: bool) -> String {
    let Some(tail) = d.get(at..) else { return String::new() };
    if unicode {
        let units: Vec<u16> = tail
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        let end = tail.iter().position(|&b| b == 0).unwrap_or(tail.len());
        // Shift-JIS in practice, but the names we look for are ASCII.
        String::from_utf8_lossy(&tail[..end]).into_owned()
    }
}

/// File headers of a BND4 or BHF4. Offsets point into the binder itself (BND4)
/// or into the companion .bdt (BHF4).
pub fn entries(data: &[u8]) -> Result<Vec<Entry>> {
    if !(data.starts_with(b"BND4") || data.starts_with(b"BHF4")) {
        return Err(ExtractError::new("not a BND4/BHF4 binder"));
    }
    if data[0x09] != 0 {
        return Err(ExtractError::new("big-endian binders are not supported"));
    }
    let bit_big_endian = data[0x0A] == 0;
    let count = le_u32(data, 0x0C)? as usize;
    let header_size = le_u64(data, 0x20)? as usize;
    let unicode = data[0x30] != 0;
    let raw = data[0x31];
    let reverse = bit_big_endian || (raw & 1 != 0 && raw & 0x80 == 0);
    let format = if reverse { raw } else { raw.reverse_bits() };

    (0..count)
        .map(|i| {
            let mut at = 0x40 + i * header_size + 8; // flags, padding, -1
            let size = le_u64(data, at)? as usize;
            at += 8;
            if format & COMPRESSION != 0 {
                at += 8; // uncompressed size
            }
            let offset = if format & LONG_OFFSETS != 0 {
                at += 8;
                le_u64(data, at - 8)? as usize
            } else {
                at += 4;
                le_u32(data, at - 4)? as usize
            };
            let id = if format & IDS != 0 {
                at += 4;
                le_u32(data, at - 4)? as i32
            } else {
                -1
            };
            let name = if format & NAMES != 0 { name_at(data, le_u32(data, at)? as usize, unicode) } else { String::new() };
            Ok(Entry { id, name, offset, size })
        })
        .collect()
}

pub fn slice<'a>(data: &'a [u8], entry: &Entry) -> Result<&'a [u8]> {
    data.get(entry.offset..entry.offset + entry.size)
        .ok_or_else(|| ExtractError::new(format!("binder file {} out of bounds", entry.name)))
}

/// Reads a BND4 that may be wrapped in DCX: (id, name, data) per file.
pub fn read_owned(data: &[u8]) -> Result<Vec<(i32, String, Vec<u8>)>> {
    let raw = dcx::decompress(data)?;
    entries(&raw)?
        .into_iter()
        .map(|e| Ok((e.id, e.name.clone(), slice(&raw, &e)?.to_vec())))
        .collect()
}
