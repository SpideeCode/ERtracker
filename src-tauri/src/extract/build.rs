//! One-off extraction run: map tile pyramids + marker positions, written to a
//! folder on the player's machine.
//!
//! Output layout:
//!   tiles/{M00,M01,M10,M11}/{z}/{x}/{y}.jpg   Leaflet pyramid, z = MAX_ZOOM is level 0
//!   markers.json                              positions keyed by event flag
//!   catalog.json                              equipment stats, upgrade costs, spells, tears
//!   items.json                                item pickups (flag, contents, position)
//!   bosses.json                               boss sheets (resistances, runes, remembrances)
//!   places.json                               named map points (dungeon entrances, landmarks)
//!   translations.json                         French names for the progress database
//!   legacy_conv.param                         raw WorldMapLegacyConvParam
//!   manifest.json                             what was extracted, from which game build

use super::archive::Archive;
use super::tiles::{TileSet, TILE};
use super::world::{MapId, MapPos, Projector};
use super::{bosses, catalog, fmg, items, oodle, regulation, ExtractError, Result};
use image::{imageops, Rgb, RgbImage, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Zoom level showing level-0 tiles at 1:1 (64 tiles of 256 px >= 41 tiles).
pub const MAX_ZOOM: u32 = 6;
const MIN_ZOOM: u32 = 2;
const JPEG_QUALITY: u8 = 82;
const BACKGROUND: Rgb<u8> = Rgb([14, 13, 11]);

const MAPS: [MapId; 4] = [MapId::M00, MapId::M01, MapId::M10, MapId::M11];

pub struct Progress<'a> {
    pub step: &'a str,
    pub done: usize,
    pub total: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MapInfo {
    pub id: MapId,
    /// Level-0 size in pixels.
    pub width: u32,
    pub height: u32,
    pub max_zoom: u32,
    pub min_zoom: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub format: u32,
    /// Size and modification time of regulation.bin, to notice game updates.
    pub game_build: String,
    pub maps: Vec<MapInfo>,
}

#[derive(Serialize)]
struct Marker {
    flag: u32,
    kind: &'static str,
    /// Map block in the game's own frame: tells dungeons from the open world.
    block: String,
    #[serde(flatten)]
    pos: MapPos,
}

/// A named place drawn on the in-game map (dungeon entrance, church, ruins...).
#[derive(Serialize)]
struct Place {
    icon: u16,
    block: String,
    name: bosses::Text,
    #[serde(flatten)]
    pos: MapPos,
}

pub const FORMAT: u32 = 6;
pub const LEGACY_CONV_FILE: &str = "legacy_conv.param";

pub fn game_build(game: &Path) -> String {
    std::fs::metadata(game.join("regulation.bin"))
        .map(|m| {
            let t = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
            format!("{}-{t}", m.len())
        })
        .unwrap_or_default()
}

/// `game_root` is the "ELDEN RING" folder (containing `Game`).
pub fn build(game_root: &Path, out: &Path, mut progress: impl FnMut(Progress)) -> Result<Manifest> {
    let game = game_root.join("Game");
    oodle::init(&game)?;
    std::fs::create_dir_all(out)?;

    progress(Progress { step: "texts", done: 0, total: 1 });
    let params = regulation::load(&game)?;
    let get = |name: &str| params.get(name).ok_or_else(|| ExtractError::new(format!("{name} missing from regulation.bin")));
    let data0 = Archive::open(&game, "Data0")?;
    let en = fmg::load_language(&data0, "engus")?;
    let fr = fmg::load_language(&data0, "frafr")?;

    let extracted = bosses::extract(&game, &params, &en, &fr, |done, total| progress(Progress { step: "bosses", done, total }))?;
    write_json(&out.join("bosses.json"), &extracted.bosses)?;

    let projector = Projector::new(get("WorldMapLegacyConvParam")?)?;
    let graces = super::graces(get("BonfireWarpParam")?)?;
    let mut markers = Vec::new();
    for g in &graces {
        if let Some(pos) = projector.project(g.pos) {
            markers.push(Marker { flag: g.flag, kind: "grace", block: g.pos.block(), pos });
        }
    }
    for b in &extracted.placements {
        if let Some(pos) = projector.project(b.pos) {
            markers.push(Marker { flag: b.flag, kind: "boss", block: b.block.clone(), pos });
        }
    }
    write_json(&out.join("markers.json"), &markers)?;
    write_json(&out.join("catalog.json"), &catalog::extract(&params, &en, &fr)?)?;
    write_json(&out.join("items.json"), &items::extract(&extracted.treasures, &params, &projector, &en, &fr)?)?;
    write_json(&out.join("places.json"), &places(get("WorldMapPointParam")?, &projector, &en, &fr)?)?;
    write_json(&out.join("translations.json"), &translations(&graces, &en, &fr))?;
    // Kept to place the player's saved position on the map later.
    std::fs::write(out.join(LEGACY_CONV_FILE), get("WorldMapLegacyConvParam")?)?;

    let tiles = TileSet::open(&data0)?;
    let mut maps = Vec::new();
    let total: usize = MAPS.iter().map(|m| tiles.grid(&format!("{m:?}"), 0)).map(|(w, h)| (w * h) as usize).sum();
    let mut done = 0;
    for map in MAPS {
        let name = format!("{map:?}");
        let (w, h) = tiles.grid(&name, 0);
        if w == 0 {
            continue;
        }
        let dir = out.join("tiles").join(&name);
        let _ = std::fs::remove_dir_all(&dir);
        build_pyramid(&tiles, &name, w, h, &dir, |n| {
            done += n;
            progress(Progress { step: "tiles", done, total });
        })?;
        maps.push(MapInfo { id: map, width: w * TILE, height: h * TILE, max_zoom: MAX_ZOOM, min_zoom: MIN_ZOOM });
    }

    let manifest = Manifest { format: FORMAT, game_build: game_build(&game), maps };
    std::fs::write(out.join("manifest.json"), serde_json::to_vec_pretty(&manifest).map_err(|e| ExtractError::new(e.to_string()))?)?;
    Ok(manifest)
}

/// WorldMapPointParam rows that have a name: dungeon entrances and landmarks.
fn places(param_data: &[u8], projector: &Projector, en: &HashMap<String, fmg::Fmg>, fr: &HashMap<String, fmg::Fmg>) -> Result<Vec<Place>> {
    let (Some(en), Some(fr)) = (en.get("PlaceName"), fr.get("PlaceName")) else { return Ok(Vec::new()) };
    Ok(super::param::Param::read(param_data)?
        .iter()
        .filter_map(|r| {
            let text = r.i32(0x30);
            let name = bosses::Text { en: en.get(&text)?.clone(), fr: fr.get(&text).cloned() };
            let tile = super::TilePos::read(&r, 0x20);
            Some(Place { icon: r.u16(0x0c), block: tile.block(), name, pos: projector.project(tile)? })
        })
        .collect())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    std::fs::write(path, serde_json::to_vec(value).map_err(|e| ExtractError::new(e.to_string()))?)?;
    Ok(())
}

#[derive(Serialize)]
struct Translations {
    /// db entry id -> French name
    names: HashMap<String, String>,
    /// English region name -> French
    regions: HashMap<String, String>,
}

/// French names for the progress database, looked up in the game's own text:
/// graces by their text id, everything else by matching the English name.
fn translations(graces: &[super::Grace], en: &HashMap<String, fmg::Fmg>, fr: &HashMap<String, fmg::Fmg>) -> Translations {
    let places = bosses::reverse(en.get("PlaceName"), fr.get("PlaceName"));
    let npcs = bosses::reverse(en.get("NpcName"), fr.get("NpcName"));
    let goods = bosses::reverse(en.get("GoodsName"), fr.get("GoodsName"));
    let place_names = fr.get("PlaceName");
    let grace_names: HashMap<u32, &String> =
        graces.iter().filter_map(|g| Some((g.flag, place_names?.get(&g.text_id)?))).collect();

    let mut names = HashMap::new();
    let mut regions = HashMap::new();
    for e in &crate::progress::db().entries {
        if let Some(region) = &e.region {
            if let Some(f) = places.get(&region.to_lowercase()) {
                regions.insert(region.clone(), f.clone());
            }
        }
        let name = match e.category.as_str() {
            "grace" => grace_names.get(&e.flag).map(|s| s.to_string()),
            "boss" => {
                // "Night's Cavalry (Limgrave)": translate the name and the place separately.
                let (base, suffix) = match e.name.split_once(" (") {
                    Some((b, s)) => (b, Some(s.trim_end_matches(')'))),
                    None => (e.name.as_str(), None),
                };
                npcs.get(&base.to_lowercase()).map(|b| match suffix {
                    Some(s) => format!("{b} ({})", places.get(&s.to_lowercase()).map_or(s, |p| p.as_str())),
                    None => b.clone(),
                })
            }
            _ => goods.get(&e.name.to_lowercase()).cloned(),
        };
        if let Some(n) = name {
            names.insert(e.id.clone(), n);
        }
    }
    Translations { names, regions }
}

fn save_jpeg(img: &RgbImage, path: &Path) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    image::codecs::jpeg::JpegEncoder::new_with_quality(file, JPEG_QUALITY)
        .encode_image(img)
        .map_err(|e| ExtractError::new(e.to_string()))
}

fn flatten(tile: &RgbaImage) -> RgbImage {
    RgbImage::from_fn(tile.width(), tile.height(), |x, y| {
        let p = tile.get_pixel(x, y);
        let a = p[3] as u16;
        let mix = |c: u8, bg: u8| ((c as u16 * a + bg as u16 * (255 - a)) / 255) as u8;
        Rgb([mix(p[0], BACKGROUND[0]), mix(p[1], BACKGROUND[1]), mix(p[2], BACKGROUND[2])])
    })
}

/// Writes level-0 tiles at MAX_ZOOM, then halves the resolution for each lower
/// zoom by merging 2x2 tiles, keeping only half-size copies in memory.
fn build_pyramid(tiles: &TileSet, map: &str, w: u32, h: u32, dir: &Path, mut advance: impl FnMut(usize)) -> Result<()> {
    // (x, y) at the current zoom -> tile downscaled to 128 px, ready for the next zoom.
    let mut halves: HashMap<(u32, u32), RgbImage> = HashMap::new();
    for x in 0..w {
        for row in 0..h {
            let y = h - 1 - row; // tile rows count from the south
            if let Some(variant) = tiles.revealed_variant(map, 0, x, row) {
                if let Some(tile) = tiles.tile(map, 0, x, row, variant)? {
                    let rgb = flatten(&tile);
                    save_jpeg(&rgb, &dir.join(format!("{MAX_ZOOM}/{x}/{y}.jpg")))?;
                    halves.insert((x, y), imageops::resize(&rgb, TILE / 2, TILE / 2, imageops::FilterType::Triangle));
                }
            }
            advance(1);
        }
    }
    for z in (MIN_ZOOM..MAX_ZOOM).rev() {
        let mut next = HashMap::new();
        let keys: std::collections::BTreeSet<(u32, u32)> = halves.keys().map(|&(x, y)| (x / 2, y / 2)).collect();
        for (x, y) in keys {
            let mut canvas = RgbImage::from_pixel(TILE, TILE, BACKGROUND);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                if let Some(part) = halves.get(&(x * 2 + dx, y * 2 + dy)) {
                    imageops::replace(&mut canvas, part, (dx * TILE / 2) as i64, (dy * TILE / 2) as i64);
                }
            }
            save_jpeg(&canvas, &dir.join(format!("{z}/{x}/{y}.jpg")))?;
            next.insert((x, y), imageops::resize(&canvas, TILE / 2, TILE / 2, imageops::FilterType::Triangle));
        }
        halves = next;
    }
    Ok(())
}
