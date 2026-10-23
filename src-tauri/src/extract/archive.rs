//! Game archives: DataN.bhd (RSA-encrypted BHD5 index) + DataN.bdt (file data,
//! partly AES-128-ECB encrypted, usually DCX-compressed). Files are looked up by
//! a 64-bit hash of their path.

use super::{ExtractError, Result};
use aes::cipher::{BlockDecrypt, KeyInit};
use base64::Engine;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Public RSA keys of the archive headers (from Smithbox `ArchiveKeys.cs`, MIT).
const KEYS: [(&str, &str); 5] = [
    ("Data0", "MIIBCwKCAQEA9Rju2whruXDVQZpfylVEPeNxm7XgMHcDyaaRUIpXQE0qEo+6Y36LP0xpFvL0H0kKxHwpuISsdgrnMHJ/yj4S61MWzhO8y4BQbw/zJehhDSRCecFJmFBz3I2JC5FCjoK+82xd9xM5XXdfsdBzRiSghuIHL4qk2WZ/0f/nK5VygeWXn/oLeYBLjX1S8wSSASza64JXjt0bP/i6mpV2SLZqKRxo7x2bIQrR1yHNekSF2jBhZIgcbtMBxjCywn+7p954wjcfjxB5VWaZ4hGbKhi1bhYPccht4XnGhcUTWO3NmJWslwccjQ4ksutLq3uRjLMM0IeTkQO6Pv8/R7UNFtdCWwIERzH8IQ=="),
    ("Data1", "MIIBCwKCAQEAxaBCHQJrtLJiJNdG9nq3deA9sY4YCZ4dbTOHO+v+YgWRMcE6iK6oZIJq+nBMUNBbGPmbRrEjkkH9M7LAypAFOPKC6wMHzqIMBsUMuYffulBuOqtEBD11CAwfx37rjwJ+/1tnEqtJjYkrK9yyrIN6Y+jy4ftymQtjk83+L89pvMMmkNeZaPON4O9q5M9PnFoKvK8eY45ZV/Jyk+Pe+xc6+e4h4cx8ML5U2kMM3VDAJush4z/05hS3/bC4B6K9+7dPwgqZgKx1J7DBtLdHSAgwRPpijPeOjKcAa2BDaNp9Cfon70oC+ZCB+HkQ7FjJcF7KaHsH5oHvuI7EZAl2XTsLEQIENa/2JQ=="),
    ("Data2", "MIIBDAKCAQEA0iDVVQ230RgrkIHJNDgxE7I/2AaH6Li1Eu9mtpfrrfhfoK2e7y4OWU+lj7AGI4GIgkWpPw8JHaV970Cr6+sTG4Tr5eMQPxrCIH7BJAPCloypxcs2BNfTGXzm6veUfrGzLIDp7wy24lIA8r9ZwUvpKlN28kxBDGeCbGCkYeSVNuF+R9rN4OAMRYh0r1Q950xc2qSNloNsjpDoSKoYN0T7u5rnMn/4mtclnWPVRWU940zr1rymv4Jc3umNf6cT1XqrS1gSaK1JWZfsSeD6Dwk3uvquvfY6YlGRygIlVEMAvKrDRMHylsLtqqhYkZNXMdy0NXopf1rEHKy9poaHEmJldwIFAP////8="),
    ("Data3", "MIIBCwKCAQEAvRRNBnVq3WknCNHrJRelcEA2v/OzKlQkxZw1yKll0Y2Kn6G9ts94SfgZYbdFCnIXy5NEuyHRKrxXz5vurjhrcuoYAI2ZUhXPXZJdgHywac/i3S/IY0V/eDbqepyJWHpP6I565ySqlol1p/BScVjbEsVyvZGtWIXLPDbx4EYFKA5B52uK6Gdz4qcyVFtVEhNoMvg+EoWnyLD7EUzuB2Khl46CuNictyWrLlIHgpKJr1QD8a0ld0PDPHDZn03q6QDvZd23UW2d9J+/HeBt52j08+qoBXPwhndZsmPMWngQDaik6FM7EVRQetKPi6h5uprVmMAS5wR/jQIVTMpTj/zJdwIEXszeQw=="),
    ("DLC", "MIIBCwKCAQEAmYJ/5GJU4boJSvZ81BFOHYTGdBWPHnWYly3yWo01BYjGRnz8NTkzDHUxsbjIgtG5XqsQfZstZILQ97hgSI5AaAoCGrT8sn0PeXg2i0mKwL21gRjRUdvPDp1Y+7hgrGwuTkjycqqsQ/qILm4NvJHvGRd7xLOJ9rs2zwYhceRVrq9XU2AXbdY4pdCQ3+HuoaFiJ0dW0ly5qdEXjbSv2QEYe36nWCtsd6hEY9LjbBX8D1fK3D2c6C0gNdHJGH2iEONUN6DMK9t0v2JBnwCOZQ7W+Gt7SpNNrkx8xKEM8gH9na10g9ne11MiO1FnLm8i4zOxVdPHQBKICkKcGS1o3C2dfwIEXw/f3w=="),
];

/// Path hash used by Elden Ring archives.
pub fn path_hash(path: &str) -> u64 {
    let mut p = path.to_ascii_lowercase().replace('\\', "/");
    if !p.starts_with('/') {
        p.insert(0, '/');
    }
    p.bytes().fold(0u64, |h, c| h.wrapping_mul(0x85).wrapping_add(c as u64))
}

struct FileEntry {
    padded_size: usize,
    unpadded_size: usize,
    offset: u64,
    aes: Option<([u8; 16], Vec<(i64, i64)>)>,
}

pub struct Archive {
    bdt: PathBuf,
    files: HashMap<u64, FileEntry>,
}

/// Minimal PKCS#1 `RSAPublicKey ::= SEQUENCE { modulus INTEGER, exponent INTEGER }` parser.
fn parse_rsa_key(b64: &str) -> Result<(BigUint, BigUint)> {
    let der = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| ExtractError::new(format!("bad RSA key: {e}")))?;
    let mut pos = 0;
    let mut read_tlv = |expected_tag: u8| -> Result<(usize, usize)> {
        let bad = || ExtractError::new("bad RSA key DER");
        if *der.get(pos).ok_or_else(bad)? != expected_tag {
            return Err(bad());
        }
        let first = *der.get(pos + 1).ok_or_else(bad)? as usize;
        let (len, header) = if first < 0x80 {
            (first, 2)
        } else {
            let n = first & 0x7F;
            let len = der.get(pos + 2..pos + 2 + n).ok_or_else(bad)?.iter().fold(0usize, |a, &b| a << 8 | b as usize);
            (len, 2 + n)
        };
        let start = pos + header;
        pos = if expected_tag == 0x30 { start } else { start + len };
        Ok((start, len))
    };
    read_tlv(0x30)?;
    let (n_start, n_len) = read_tlv(0x02)?;
    let (e_start, e_len) = read_tlv(0x02)?;
    Ok((
        BigUint::from_bytes_be(&der[n_start..n_start + n_len]),
        BigUint::from_bytes_be(&der[e_start..e_start + e_len]),
    ))
}

/// "Public key decryption" (raw RSA with e), 256-byte blocks -> 255-byte blocks.
fn rsa_decrypt(data: &[u8], key: &str) -> Result<Vec<u8>> {
    let (n, e) = parse_rsa_key(key)?;
    let in_size = (n.bits() as usize).div_ceil(8);
    let out_size = in_size - 1;
    let mut out = Vec::with_capacity(data.len() / in_size * out_size);
    for block in data.chunks(in_size) {
        let m = BigUint::from_bytes_be(block).modpow(&e, &n).to_bytes_be();
        out.resize(out.len() + out_size - m.len(), 0);
        out.extend_from_slice(&m);
    }
    Ok(out)
}

fn rd<const N: usize>(d: &[u8], at: usize) -> Result<[u8; N]> {
    d.get(at..at + N)
        .map(|b| b.try_into().unwrap())
        .ok_or_else(|| ExtractError::new("BHD5 read out of bounds"))
}
fn rd_u32(d: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(rd(d, at)?))
}
fn rd_u64(d: &[u8], at: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(rd(d, at)?))
}

impl Archive {
    /// `name` is "Data0".."Data3" or "DLC".
    pub fn open(game_dir: &Path, name: &str) -> Result<Self> {
        let key = KEYS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, k)| *k)
            .ok_or_else(|| ExtractError::new(format!("unknown archive {name}")))?;
        let bhd = rsa_decrypt(&std::fs::read(game_dir.join(format!("{name}.bhd")))?, key)?;
        if !bhd.starts_with(b"BHD5") {
            return Err(ExtractError::new(format!("{name}.bhd did not decrypt to BHD5")));
        }
        let bucket_count = rd_u32(&bhd, 0x10)? as usize;
        let buckets = rd_u32(&bhd, 0x14)? as usize;
        let mut files = HashMap::new();
        for b in 0..bucket_count {
            let count = rd_u32(&bhd, buckets + b * 8)? as usize;
            let headers = rd_u32(&bhd, buckets + b * 8 + 4)? as usize;
            for i in 0..count {
                let h = headers + i * 0x28;
                let hash = rd_u64(&bhd, h)?;
                let aes_offset = rd_u64(&bhd, h + 0x20)? as usize;
                let aes = if aes_offset != 0 {
                    let key: [u8; 16] = rd(&bhd, aes_offset)?;
                    let n = rd_u32(&bhd, aes_offset + 16)? as usize;
                    let ranges = (0..n)
                        .map(|r| {
                            let at = aes_offset + 20 + r * 16;
                            Ok((rd_u64(&bhd, at)? as i64, rd_u64(&bhd, at + 8)? as i64))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Some((key, ranges))
                } else {
                    None
                };
                files.insert(
                    hash,
                    FileEntry {
                        padded_size: rd_u32(&bhd, h + 8)? as usize,
                        unpadded_size: rd_u32(&bhd, h + 12)? as usize,
                        offset: rd_u64(&bhd, h + 16)?,
                        aes,
                    },
                );
            }
        }
        Ok(Self { bdt: game_dir.join(format!("{name}.bdt")), files })
    }

    pub fn contains(&self, path: &str) -> bool {
        self.files.contains_key(&path_hash(path))
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Decrypted file contents (still DCX-compressed if the file is).
    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let entry = self
            .files
            .get(&path_hash(path))
            .ok_or_else(|| ExtractError::new(format!("{path} not found in archive")))?;
        let mut file = File::open(&self.bdt)?;
        file.seek(SeekFrom::Start(entry.offset))?;
        let mut data = vec![0u8; entry.padded_size];
        file.read_exact(&mut data)?;
        if let Some((key, ranges)) = &entry.aes {
            let cipher = aes::Aes128::new(key.into());
            for &(start, end) in ranges {
                if start < 0 || end <= start {
                    continue;
                }
                let (start, end) = (start as usize, (end as usize).min(data.len()));
                for block in data[start..end].chunks_exact_mut(16) {
                    cipher.decrypt_block(block.into());
                }
            }
        }
        if entry.unpadded_size > 0 && entry.unpadded_size <= data.len() {
            data.truncate(entry.unpadded_size);
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash_is_case_and_separator_insensitive() {
        use super::path_hash;
        assert_eq!(path_hash("/menu/71_maptile.tpfbhd"), path_hash("MENU\\71_MapTile.tpfbhd"));
    }
}
