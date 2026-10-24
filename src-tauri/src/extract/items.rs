//! Item pickups: MSB treasure events -> ItemLotParam_map (what you get and the
//! event flag set once it is picked up) -> names from the game's text.

use super::bosses::{tile_pos, Text};
use super::fmg::Fmg;
use super::msb::Treasure;
use super::param::Param;
use super::world::{MapPos, Projector};
use super::{ExtractError, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
pub struct Pickup {
    pub flag: u32,
    pub lot: i32,
    pub block: String,
    #[serde(flatten)]
    pub pos: MapPos,
    pub items: Vec<LotItem>,
}

#[derive(Serialize)]
pub struct LotItem {
    /// 1 goods, 2 weapon, 3 armour, 4 talisman, 5 ash of war
    pub cat: u8,
    pub id: i32,
    pub name: Text,
    pub count: u8,
    /// EquipParamGoods.goodsType, for goods.
    pub goods_type: Option<u8>,
    /// Artwork id (MENU_Knowledge_<icon>), served as img/icon/<icon>.png.
    pub icon: Option<u16>,
}

const TABLES: [&str; 6] = ["", "GoodsName", "WeaponName", "ProtectorName", "AccessoryName", "GemName"];

fn lookup(table: Option<&Fmg>, cat: u8, id: i32) -> Option<String> {
    let t = table?;
    // Weapons carry their upgrade level and affinity in the id.
    let candidates: &[i32] = if cat == 2 { &[id, id / 100 * 100, id / 10000 * 10000] } else { &[id] };
    candidates.iter().find_map(|i| t.get(i).cloned())
}

pub fn extract(
    treasures: &[(String, Treasure)],
    params: &HashMap<String, Vec<u8>>,
    projector: &Projector,
    en: &HashMap<String, Fmg>,
    fr: &HashMap<String, Fmg>,
) -> Result<Vec<Pickup>> {
    let get = |name: &str| params.get(name).ok_or_else(|| ExtractError::new(format!("{name} missing")));
    let lot_param = Param::read(get("ItemLotParam_map")?)?;
    let lots: HashMap<i32, _> = lot_param.iter().map(|r| (r.id, r)).collect();
    let goods_param = Param::read(get("EquipParamGoods")?)?;
    let goods_types: HashMap<i32, u8> = goods_param.iter().map(|r| (r.id, r.u8(0x3e))).collect();
    // Icon id per category: goods, weapon, armour (male), talisman, ash of war.
    let icons: Vec<HashMap<i32, u16>> = [("", 0), ("EquipParamGoods", 0x30), ("EquipParamWeapon", 0xbe), ("EquipParamProtector", 0xa6), ("EquipParamAccessory", 0x26), ("EquipParamGem", 0x4)]
        .iter()
        .map(|&(name, at)| {
            params
                .get(name)
                .and_then(|d| Param::read(d).ok())
                .map(|p| p.iter().map(|r| (r.id, r.u16(at))).collect())
                .unwrap_or_default()
        })
        .collect();

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (map, t) in treasures {
        let Some(row) = lots.get(&t.item_lot) else { continue };
        let flag = row.u32(0x80);
        // Without a flag the pickup respawns and the save cannot tell it was taken.
        if flag == 0 || !seen.insert(flag) {
            continue;
        }
        // A pickup spans the following rows that share its flag (e.g. a full armour set).
        let chain: Vec<_> = (0..)
            .map_while(|k| lots.get(&(t.item_lot + k)).filter(|r| k == 0 || r.u32(0x80) == flag))
            .collect();
        let items: Vec<LotItem> = chain
            .iter()
            .flat_map(|row| (0..8).map(move |i| (row, i)))
            .filter_map(|(row, i)| {
                let (id, cat) = (row.i32(i * 4), row.i32(0x20 + i * 4));
                if id <= 0 || !(1..=5).contains(&cat) {
                    return None;
                }
                let cat = cat as u8;
                let table = TABLES[cat as usize];
                Some(LotItem {
                    cat,
                    id,
                    name: Text { en: lookup(en.get(table), cat, id)?, fr: lookup(fr.get(table), cat, id) },
                    count: row.u8(0x8a + i).max(1),
                    goods_type: (cat == 1).then(|| goods_types.get(&id).copied()).flatten(),
                    icon: icons[cat as usize].get(&id).or_else(|| icons[cat as usize].get(&(id / 10000 * 10000))).copied(),
                })
            })
            .collect();
        if items.is_empty() {
            continue;
        }
        let Some(tile) = tile_pos(map, t.pos) else { continue };
        let Some(pos) = projector.project(tile) else { continue };
        out.push(Pickup { flag, lot: t.item_lot, block: map.get(..9).unwrap_or(map).to_string(), pos, items });
    }
    Ok(out)
}
