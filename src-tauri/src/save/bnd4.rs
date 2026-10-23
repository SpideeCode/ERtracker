use super::reader::Reader;
use super::{Result, SaveError};

pub struct Entry {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

/// Lists the files of a BND4 container (the format of ER0000.sl2).
pub fn entries(data: &[u8]) -> Result<Vec<Entry>> {
    if data.get(0..4) != Some(b"BND4") {
        return Err(SaveError::new("not a BND4 file"));
    }
    let mut r = Reader::at(data, 0x0c);
    let count = r.u32()? as usize;
    r.pos = 0x20;
    let entry_size = r.u64()? as usize;
    if count > 64 || entry_size < 0x18 {
        return Err(SaveError::new(format!("unexpected BND4 header ({count} entries, {entry_size:#x} bytes each)")));
    }

    (0..count)
        .map(|i| {
            let mut r = Reader::at(data, 0x40 + i * entry_size);
            r.skip(8)?;
            let size = r.u64()? as usize;
            let offset = r.u32()? as usize;
            let name_offset = r.u32()? as usize;
            if offset.checked_add(size).is_none_or(|end| end > data.len()) {
                return Err(SaveError::new(format!("entry {i} is out of bounds")));
            }
            let name = Reader::at(data, name_offset).wstr(32)?;
            Ok(Entry { name, offset, size })
        })
        .collect()
}
