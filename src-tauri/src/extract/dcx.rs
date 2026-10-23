//! DCX: FromSoftware's compression wrapper (zlib, Zstandard or Oodle Kraken).

use super::{oodle, ExtractError, Result};
use std::io::Read;

pub fn is_dcx(data: &[u8]) -> bool {
    data.starts_with(b"DCX\0")
}

fn be_u32(data: &[u8], at: usize) -> Result<u32> {
    data.get(at..at + 4)
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
        .ok_or_else(|| ExtractError::new("truncated DCX header"))
}

/// Returns `data` unchanged when it is not DCX-compressed.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    if !is_dcx(data) {
        return Ok(data.to_vec());
    }
    // Header fields are big-endian. The compressed stream starts after "DCA\0" + its size.
    let uncompressed = be_u32(data, 0x1C)? as usize;
    let compressed = be_u32(data, 0x20)? as usize;
    let format = data.get(0x28..0x2C).ok_or_else(|| ExtractError::new("truncated DCX header"))?;
    let dca = data.windows(4).position(|w| w == b"DCA\0").ok_or_else(|| ExtractError::new("DCX without DCA block"))?;
    let start = dca + be_u32(data, dca + 4)? as usize;
    let body = data
        .get(start..start + compressed)
        .ok_or_else(|| ExtractError::new("DCX body out of bounds"))?;

    let mut out = Vec::with_capacity(uncompressed);
    match format {
        b"DFLT" => {
            flate2::read::ZlibDecoder::new(body).read_to_end(&mut out)?;
        }
        b"ZSTD" => {
            out = zstd::stream::decode_all(body)?;
        }
        b"KRAK" => {
            out = oodle::decompress(body, uncompressed)?;
        }
        other => return Err(ExtractError::new(format!("unsupported DCX format {:?}", String::from_utf8_lossy(other)))),
    }
    if out.len() != uncompressed {
        return Err(ExtractError::new(format!("DCX size mismatch: {} != {uncompressed}", out.len())));
    }
    Ok(out)
}
