//! World map tiles from /menu/71_maptile.tpfbhd (256x256 BC7 textures).
//!
//! Tile names: MENU_MapTile_M{map}_L{level}_{x}_{y}_{variant}. Level 0 is the
//! most detailed (41x41 tiles = 10496 px, 1 px = 1 m). Row 0 is the south edge.
//!
//! The variant is a bitmask of world state (map fragments held, burned Erdtree,
//! ashen capital...). /menu/71_maptile.mtmskbnd holds, per cell, the variant that
//! shows the fully revealed map before world changes; cells without an entry use
//! variant 0.

use super::archive::Archive;
use super::{binder, dcx, ExtractError, Result};
use image::RgbaImage;
use std::collections::HashMap;

pub const TILE: u32 = 256;

pub struct TileSet {
    bdt: Vec<u8>,
    /// (map, level, x, y) -> [(variant, binder entry)]
    entries: HashMap<(String, u8, u32, u32), Vec<(u32, binder::Entry)>>,
    /// map -> (level * 10000 + x * 100 + y) -> fully revealed variant
    masks: HashMap<String, HashMap<u32, u32>>,
}

impl TileSet {
    pub fn open(data0: &Archive) -> Result<Self> {
        let bhd = dcx::decompress(&data0.read("/menu/71_maptile.tpfbhd")?)?;
        let bdt = data0.read("/menu/71_maptile.tpfbdt")?;
        let mut entries = HashMap::new();
        for e in binder::entries(&bhd)? {
            let stem = e.name.rsplit(['\\', '/']).next().unwrap_or(&e.name).trim_end_matches(".tpf.dcx");
            let p: Vec<&str> = stem.split('_').collect();
            let [_, _, map, level, x, y, variant] = p.as_slice() else { continue };
            let (Some(level), Ok(x), Ok(y), Ok(variant)) = (
                level.strip_prefix('L').and_then(|l| l.parse().ok()),
                x.parse(),
                y.parse(),
                u32::from_str_radix(variant, 16),
            ) else {
                continue;
            };
            entries.entry((map.to_string(), level, x, y)).or_insert_with(Vec::new).push((variant, e));
        }
        let masks = load_masks(&data0.read("/menu/71_maptile.mtmskbnd.dcx")?)?;
        Ok(Self { bdt, entries, masks })
    }

    /// Variant showing the cell fully revealed, if the cell exists.
    pub fn revealed_variant(&self, map: &str, level: u8, x: u32, y: u32) -> Option<u32> {
        let variants = self.entries.get(&(map.to_string(), level, x, y))?;
        let mask = self.masks.get(map).and_then(|m| m.get(&(level as u32 * 10000 + x * 100 + y)));
        let has = |v: u32| variants.iter().any(|(var, _)| *var == v);
        match mask {
            Some(&m) if has(m) => Some(m),
            _ if has(0) => Some(0),
            _ => variants.iter().map(|(v, _)| *v).min(),
        }
    }

    /// Grid size (columns, rows) of a map at a level, from the tiles present.
    pub fn grid(&self, map: &str, level: u8) -> (u32, u32) {
        self.entries
            .keys()
            .filter(|(m, l, _, _)| m == map && *l == level)
            .fold((0, 0), |(w, h), (_, _, x, y)| (w.max(x + 1), h.max(y + 1)))
    }

    pub fn variants(&self, map: &str, level: u8, x: u32, y: u32) -> Vec<u32> {
        let mut v: Vec<u32> = self.entries.get(&(map.to_string(), level, x, y)).map_or(Vec::new(), |e| e.iter().map(|(v, _)| *v).collect());
        v.sort();
        v
    }

    pub fn tile(&self, map: &str, level: u8, x: u32, y: u32, variant: u32) -> Result<Option<RgbaImage>> {
        let Some(entry) = self
            .entries
            .get(&(map.to_string(), level, x, y))
            .and_then(|v| v.iter().find(|(var, _)| *var == variant))
            .map(|(_, e)| e)
        else {
            return Ok(None);
        };
        let tpf = dcx::decompress(binder::slice(&self.bdt, entry)?)?;
        decode_tpf(&tpf).map(Some)
    }
}

/// `<MapTileMask exists="1" id="..." mask="..."/>` rows of each MENU_MapTile_Mxx.mtmsk.
fn load_masks(mtmskbnd: &[u8]) -> Result<HashMap<String, HashMap<u32, u32>>> {
    let mut out = HashMap::new();
    for (_, name, data) in binder::read_owned(mtmskbnd)? {
        let file = name.rsplit(['\\', '/']).next().unwrap_or(&name);
        let Some(map) = file.strip_prefix("MENU_MapTile_").and_then(|f| f.split('.').next())
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&data);
        let attr = |tag: &str, key: &str| -> Option<u32> {
            let start = tag.find(&format!("{key}=\""))? + key.len() + 2;
            tag[start..].split('"').next()?.parse().ok()
        };
        let cells = text
            .split("<MapTileMask")
            .skip(1)
            .filter(|tag| attr(tag, "exists") == Some(1))
            .filter_map(|tag| Some((attr(tag, "id")?, attr(tag, "mask")?)))
            .collect();
        out.insert(map.to_string(), cells);
    }
    Ok(out)
}

/// First texture of a PC TPF, decoded to RGBA.
fn decode_tpf(tpf: &[u8]) -> Result<RgbaImage> {
    if !tpf.starts_with(b"TPF\0") {
        return Err(ExtractError::new("not a TPF"));
    }
    let u32_at = |d: &[u8], at: usize| -> Result<u32> {
        d.get(at..at + 4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .ok_or_else(|| ExtractError::new("truncated texture"))
    };
    let offset = u32_at(tpf, 0x10)? as usize;
    let size = u32_at(tpf, 0x14)? as usize;
    let dds = tpf.get(offset..offset + size).ok_or_else(|| ExtractError::new("TPF texture out of bounds"))?;
    if !dds.starts_with(b"DDS ") {
        return Err(ExtractError::new("texture is not a DDS"));
    }
    let height = u32_at(dds, 12)? as usize;
    let width = u32_at(dds, 16)? as usize;
    let fourcc = dds.get(84..88).ok_or_else(|| ExtractError::new("truncated DDS"))?;
    // Map legacy FourCCs onto DXGI formats. The first mip level comes first.
    let (format, start) = match fourcc {
        b"DX10" => (u32_at(dds, 128)?, 148),
        b"DXT1" => (71, 128),
        b"DXT5" => (77, 128),
        b"ATI1" | b"BC4U" => (80, 128),
        b"ATI2" | b"BC5U" => (83, 128),
        other => return Err(ExtractError::new(format!("unsupported DDS FourCC {:?}", String::from_utf8_lossy(other)))),
    };
    let blocks = &dds[start..];
    let mut pixels = vec![0u32; width * height];
    let decoded = match format {
        98 | 99 => texture2ddecoder::decode_bc7(blocks, width, height, &mut pixels),
        71 | 72 => texture2ddecoder::decode_bc1(blocks, width, height, &mut pixels),
        77 | 78 => texture2ddecoder::decode_bc3(blocks, width, height, &mut pixels),
        80 => texture2ddecoder::decode_bc4(blocks, width, height, &mut pixels),
        83 => texture2ddecoder::decode_bc5(blocks, width, height, &mut pixels),
        other => return Err(ExtractError::new(format!("unsupported DXGI format {other}"))),
    };
    decoded.map_err(|e| ExtractError::new(e.to_string()))?;
    // texture2ddecoder yields 0xAARRGGBB words.
    let rgba: Vec<u8> = pixels
        .iter()
        .flat_map(|p| {
            let [b, g, r, a] = p.to_le_bytes();
            [r, g, b, a]
        })
        .collect();
    RgbaImage::from_raw(width as u32, height as u32, rgba).ok_or_else(|| ExtractError::new("bad texture size"))
}
