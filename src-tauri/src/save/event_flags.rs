use std::collections::HashMap;
use std::sync::OnceLock;

pub const EVENT_FLAGS_SIZE: usize = 0x1BF99F;

/// Block (flag id / 1000) -> position of that block in the bitfield, in units of 125 bytes.
/// Mirrors the game's CSFD4VirtualMemoryFlag tree. Source: see third_party/er-save-manager.
static BST: OnceLock<HashMap<u32, u32>> = OnceLock::new();

fn bst() -> &'static HashMap<u32, u32> {
    BST.get_or_init(|| {
        include_str!("../../../data/eventflag_bst.txt")
            .lines()
            .filter_map(|line| {
                let (block, pos) = line.trim().split_once(',')?;
                Some((block.parse().ok()?, pos.parse().ok()?))
            })
            .collect()
    })
}

/// Byte index and bit index (0 = least significant) of a flag in the bitfield.
pub fn position(id: u32) -> Option<(usize, u8)> {
    let pos = *bst().get(&(id / 1000))? as usize;
    let rem = (id % 1000) as usize;
    Some((pos * 125 + rem / 8, 7 - (rem % 8) as u8))
}

/// `None` when the flag id is not part of the game's flag table.
pub fn get(flags: &[u8], id: u32) -> Option<bool> {
    let (byte, bit) = position(id)?;
    flags.get(byte).map(|b| b >> bit & 1 == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_positions() {
        // Reference values from ER-Save-Editor's precomputed table.
        assert_eq!(position(300), Some((0x25, 3)));
        assert_eq!(position(62010), Some((0x5dd, 5)));
        assert_eq!(position(10000800), Some((0x151c33, 7)));
        assert_eq!(position(11000850), Some((0x152dcd, 5)));
        assert_eq!(position(31000800), Some((0x16ff99, 7)));
    }
}
