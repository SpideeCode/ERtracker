//! Equipment catalogue for item sheets and the build advisor: weapon damage and
//! scaling (base and fully upgraded), upgrade costs, armour absorptions,
//! talismans, spells, physick tears... with French and English text.

use super::bosses::Text;
use super::fmg::Fmg;
use super::param::Param;
use super::{ExtractError, Result};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize)]
pub struct Catalog {
    /// Upgrade paths, keyed "reinforceType:materialSet:price".
    upgrades: BTreeMap<String, Vec<UpgradeStep>>,
    weapons: Vec<Weapon>,
    armor: Vec<Armor>,
    talismans: Vec<Talisman>,
    goods: Vec<Goods>,
    ashes_of_war: Vec<Simple>,
}

#[derive(Serialize)]
struct UpgradeStep {
    /// Level reached by this step (+1, +2...).
    level: u8,
    runes: u32,
    materials: Vec<Material>,
}

#[derive(Serialize)]
struct Material {
    id: i32,
    count: i8,
    name: Text,
}

#[derive(Serialize)]
struct Weapon {
    id: i32,
    name: Text,
    info: Option<Text>,
    rarity: u8,
    weight: f32,
    /// Physical, magic, fire, lightning, holy at +0 and fully upgraded.
    attack: [u16; 5],
    attack_max: [u16; 5],
    /// Strength, dexterity, intelligence, faith, arcane scaling at +0 and fully upgraded.
    scaling: [f32; 5],
    scaling_max: [f32; 5],
    requirements: [u8; 5],
    max_level: u8,
    upgrade: String,
    /// Can take another Ash of War / affinity.
    infusable: bool,
}

#[derive(Serialize)]
struct Armor {
    id: i32,
    name: Text,
    rarity: u8,
    weight: f32,
    /// 0 head, 1 chest, 2 arms, 3 legs
    slot: u8,
    /// % absorbed: physical, strike, slash, pierce, magic, fire, lightning, holy.
    absorption: [f32; 8],
    /// Immunity, robustness, focus, vitality.
    resistance: [u16; 4],
    poise: f32,
}

#[derive(Serialize)]
struct Talisman {
    id: i32,
    name: Text,
    info: Option<Text>,
    rarity: u8,
    weight: f32,
    /// Talismans of the same group cannot be worn together.
    group: i16,
}

#[derive(Serialize)]
struct Goods {
    id: i32,
    name: Text,
    info: Option<Text>,
    goods_type: u8,
    rarity: u8,
    spell: Option<Spell>,
}

#[derive(Serialize)]
struct Spell {
    fp: i16,
    slots: u8,
    /// Intelligence, faith, arcane.
    requirements: [u8; 3],
}

#[derive(Serialize)]
struct Simple {
    id: i32,
    name: Text,
    info: Option<Text>,
}

/// Goods kept in the catalogue: spells, spirit ashes, physick tears.
const GOODS_TYPES: [u8; 7] = [5, 7, 8, 10, 16, 17, 18];

struct Texts<'a> {
    en: &'a HashMap<String, Fmg>,
    fr: &'a HashMap<String, Fmg>,
}

impl Texts<'_> {
    fn get(&self, table: &str, id: i32) -> Option<Text> {
        let en = self.en.get(table)?.get(&id)?.clone();
        Some(Text { en, fr: self.fr.get(table).and_then(|t| t.get(&id)).cloned() })
    }
}

struct Rows<'a> {
    data: &'a [u8],
    size: usize,
    index: HashMap<i32, usize>,
    order: Vec<i32>,
}

impl<'a> Rows<'a> {
    fn new(params: &'a HashMap<String, Vec<u8>>, name: &str) -> Result<Self> {
        let data = params.get(name).ok_or_else(|| ExtractError::new(format!("{name} missing")))?;
        let p = Param::read(data)?;
        Ok(Self { data, size: p.row_size(), index: p.rows.iter().copied().collect(), order: p.rows.iter().map(|r| r.0).collect() })
    }
    fn get(&self, id: i32) -> Option<&'a [u8]> {
        let off = *self.index.get(&id)?;
        self.data.get(off..off + self.size)
    }
}

fn u8_(d: &[u8], o: usize) -> u8 {
    d[o]
}
fn i8_(d: &[u8], o: usize) -> i8 {
    d[o] as i8
}
fn u16_(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}
fn i16_(d: &[u8], o: usize) -> i16 {
    u16_(d, o) as i16
}
fn i32_(d: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn f32_(d: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}

pub fn extract(params: &HashMap<String, Vec<u8>>, en: &HashMap<String, Fmg>, fr: &HashMap<String, Fmg>) -> Result<Catalog> {
    let text = Texts { en, fr };
    let weapons_p = Rows::new(params, "EquipParamWeapon")?;
    let reinforce = Rows::new(params, "ReinforceParamWeapon")?;
    let materials = Rows::new(params, "EquipMtrlSetParam")?;

    let mut upgrades = BTreeMap::new();
    let mut weapons = Vec::new();
    for &id in &weapons_p.order {
        // Affinity and upgrade variants share the base name: keep base weapons.
        if id % 10000 != 0 || id <= 0 {
            continue;
        }
        let (Some(d), Some(name)) = (weapons_p.get(id), text.get("WeaponName", id)) else { continue };
        let rtype = i16_(d, 0xda) as i32;
        let price = i32_(d, 0x1c);
        let mset = i32_(d, 0x5c);
        let max_level = (1..=25).take_while(|l| reinforce.get(rtype + l).is_some()).last().unwrap_or(0) as u8;
        let top = reinforce.get(rtype + max_level as i32).or_else(|| reinforce.get(rtype));
        let attack = [u16_(d, 0xc8), u16_(d, 0xca), u16_(d, 0xcc), u16_(d, 0xce), u16_(d, 0x18c)];
        let scaling = [f32_(d, 0x24), f32_(d, 0x28), f32_(d, 0x2c), f32_(d, 0x30), f32_(d, 0x19c)];
        let (attack_max, scaling_max) = match top {
            Some(r) => {
                let a = [0x0, 0x4, 0x8, 0xc, 0x58].map(|o| f32_(r, o));
                let s = [0x1c, 0x20, 0x24, 0x28, 0x60].map(|o| f32_(r, o));
                (
                    std::array::from_fn(|i| (attack[i] as f32 * a[i]).round() as u16),
                    std::array::from_fn(|i| (scaling[i] * s[i] * 10.0).round() / 10.0),
                )
            }
            None => (attack, scaling),
        };
        let key = format!("{rtype}:{mset}:{price}");
        if max_level > 0 && !upgrades.contains_key(&key) && mset >= 0 {
            let steps = (1..=max_level as i32)
                .filter_map(|l| {
                    let r = reinforce.get(rtype + l)?;
                    let set = materials.get(mset + u8_(r, 0x56) as i32);
                    let mats = set
                        .map(|m| {
                            (0..6)
                                .filter(|&i| i32_(m, i * 4) > 0 && i8_(m, 0x20 + i) > 0)
                                .filter_map(|i| {
                                    let mid = i32_(m, i * 4);
                                    Some(Material { id: mid, count: i8_(m, 0x20 + i), name: text.get("GoodsName", mid)? })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(UpgradeStep { level: l as u8, runes: (price as f32 * f32_(r, 0x68)).round() as u32, materials: mats })
                })
                .collect();
            upgrades.insert(key.clone(), steps);
        }
        weapons.push(Weapon {
            id,
            name,
            info: text.get("WeaponInfo", id),
            rarity: u8_(d, 0x240),
            weight: (f32_(d, 0x10) * 10.0).round() / 10.0,
            attack,
            attack_max,
            scaling,
            scaling_max,
            requirements: [u8_(d, 0xf2), u8_(d, 0xf3), u8_(d, 0xf4), u8_(d, 0xf5), u8_(d, 0x195)],
            max_level,
            upgrade: key,
            infusable: u8_(d, 0x241) == 2,
        });
    }

    let protector = Rows::new(params, "EquipParamProtector")?;
    let cut = |d: &[u8], o: usize| ((1.0 - f32_(d, o)) * 1000.0).round() / 10.0;
    let armor = protector
        .order
        .iter()
        .filter_map(|&id| {
            let d = protector.get(id)?;
            Some(Armor {
                id,
                name: text.get("ProtectorName", id)?,
                rarity: u8_(d, 0xd7),
                weight: (f32_(d, 0x24) * 10.0).round() / 10.0,
                slot: u8_(d, 0xd6),
                absorption: [0xe4, 0xec, 0xe8, 0xf0, 0xf4, 0xf8, 0xfc, 0x11c].map(|o| cut(d, o)),
                resistance: [u16_(d, 0xc4), u16_(d, 0xc8), u16_(d, 0x0c), u16_(d, 0xca)],
                poise: (f32_(d, 0x14) * 1000.0).round() / 10.0,
            })
        })
        .collect();

    let accessory = Rows::new(params, "EquipParamAccessory")?;
    let talismans = accessory
        .order
        .iter()
        .filter_map(|&id| {
            let d = accessory.get(id)?;
            Some(Talisman {
                id,
                name: text.get("AccessoryName", id)?,
                info: text.get("AccessoryInfo", id),
                rarity: u8_(d, 0x41),
                weight: (f32_(d, 0xc) * 10.0).round() / 10.0,
                group: i16_(d, 0x48),
            })
        })
        .collect();

    let goods_p = Rows::new(params, "EquipParamGoods")?;
    let magic = Rows::new(params, "Magic")?;
    let goods = goods_p
        .order
        .iter()
        .filter_map(|&id| {
            let d = goods_p.get(id)?;
            let goods_type = u8_(d, 0x3e);
            if !GOODS_TYPES.contains(&goods_type) {
                return None;
            }
            Some(Goods {
                id,
                name: text.get("GoodsName", id)?,
                info: text.get("GoodsInfo", id),
                goods_type,
                rarity: u8_(d, 0x78),
                spell: magic.get(id).map(|m| Spell {
                    fp: i16_(m, 0x10),
                    slots: u8_(m, 0x21),
                    requirements: [u8_(m, 0x22), u8_(m, 0x23), u8_(m, 0x0e)],
                }),
            })
        })
        .collect();

    let ashes_of_war = en
        .get("GemName")
        .map(|names| {
            let mut ids: Vec<i32> = names.keys().copied().collect();
            ids.sort_unstable();
            ids.into_iter()
                .filter_map(|id| Some(Simple { id, name: text.get("GemName", id)?, info: text.get("GemInfo", id) }))
                .collect()
        })
        .unwrap_or_default();

    Ok(Catalog { upgrades, weapons, armor, talismans, goods, ashes_of_war })
}
