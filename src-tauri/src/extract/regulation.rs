//! regulation.bin: AES-256-CBC encrypted, DCX-compressed BND4 of PARAM files.

use super::{binder, dcx, ExtractError, Result};
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use std::collections::HashMap;
use std::path::Path;

/// Publicly documented key (SoulsFormats / Smithbox `ER_REGULATION_KEY`).
const KEY: [u8; 32] = [
    0x99, 0xBF, 0xFC, 0x36, 0x6A, 0x6B, 0xC8, 0xC6, 0xF5, 0x82, 0x7D, 0x09, 0x36, 0x02, 0xD6, 0x76, 0xC4, 0x28, 0x92,
    0xA0, 0x1C, 0x20, 0x7F, 0xB0, 0x24, 0xD3, 0xAF, 0x4E, 0x49, 0x3F, 0xEF, 0x99,
];

/// Param name (e.g. "BonfireWarpParam") -> raw PARAM bytes.
pub fn load(game_dir: &Path) -> Result<HashMap<String, Vec<u8>>> {
    let raw = std::fs::read(game_dir.join("regulation.bin"))?;
    let bytes = if raw.starts_with(b"BND4") || dcx::is_dcx(&raw) { raw } else { decrypt(&raw)? };
    let files = binder::read_owned(&bytes)?;
    Ok(files
        .into_iter()
        .map(|(_, name, data)| {
            let file = name.rsplit(['\\', '/']).next().unwrap_or(&name);
            (file.trim_end_matches(".param").to_string(), data)
        })
        .collect())
}

fn decrypt(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() < 32 {
        return Err(ExtractError::new("regulation.bin is too small"));
    }
    let (iv, body) = raw.split_at(16);
    let mut buf = body[..body.len() / 16 * 16].to_vec();
    cbc::Decryptor::<aes::Aes256>::new(&KEY.into(), iv.into())
        .decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut buf)
        .map_err(|e| ExtractError::new(format!("regulation decryption failed: {e}")))?;
    Ok(buf)
}
