// Equipment catalogue (catalog.json) helpers: rarity, scaling grades, damage
// estimates with the character's stats, ownership from the save, and where
// each piece can be picked up.
import type { Catalog, CatalogWeapon, ItemPickup } from "./types";

export const DAMAGE_TYPES = ["Physique", "Magie", "Feu", "Foudre", "Sacré"];
export const STATS = ["Force", "Dextérité", "Intelligence", "Foi", "Arcane"];
/** Character stat index (vigor, mind, endurance, str, dex, int, fai, arc) for each scaling stat. */
const STAT_INDEX = [3, 4, 5, 6, 7];

export const RARITY = [
  { label: "Commun", color: "#c9c2b0" },
  { label: "Peu commun", color: "#7fb4e8" },
  { label: "Rare", color: "#b48be8" },
  { label: "Légendaire", color: "#f3d27a" },
];
export const rarityOf = (r: number) => RARITY[Math.min(Math.max(r, 0), 3)];

/** In-game scaling letter for a scaling value. */
export function grade(v: number): string {
  if (v <= 0) return "–";
  if (v >= 175) return "S";
  if (v >= 140) return "A";
  if (v >= 90) return "B";
  if (v >= 60) return "C";
  if (v >= 25) return "D";
  return "E";
}

/** Share of the scaling bonus unlocked at a stat level (default soft-cap curve). */
function curve(stat: number): number {
  const points: [number, number][] = [
    [1, 0],
    [18, 0.25],
    [60, 0.75],
    [80, 0.9],
    [150, 1],
  ];
  if (stat >= 150) return 1;
  for (let i = 1; i < points.length; i++) {
    const [x1, y1] = points[i];
    const [x0, y0] = points[i - 1];
    if (stat <= x1) return y0 + ((y1 - y0) * (stat - x0)) / (x1 - x0);
  }
  return 1;
}

/** Which scaling stats feed each damage type (simplified: str/dex/arc physical, int magic, fai fire and holy, dex lightning). */
const FEEDS: number[][] = [[0, 1, 4], [2], [3], [1], [3]];

export interface DamageEstimate {
  /** Per damage type, fully upgraded, with the given stats. */
  byType: number[];
  total: number;
  meetsRequirements: boolean;
}

/** Rough attack rating at max upgrade. Requirement shortfall halves the weapon's scaled damage. */
export function estimate(w: CatalogWeapon, stats: number[] | undefined): DamageEstimate {
  const s = stats ?? [10, 10, 10, 20, 20, 20, 20, 20];
  const meets = w.requirements.every((req, i) => s[STAT_INDEX[i]] >= req);
  const byType = w.attack_max.map((base, t) => {
    if (!base) return 0;
    const bonus = FEEDS[t].reduce((sum, stat) => sum + (w.scaling_max[stat] / 100) * curve(s[STAT_INDEX[stat]]), 0);
    return Math.round((base + base * bonus) * (meets ? 1 : 0.6));
  });
  return { byType, total: byType.reduce((a, b) => a + b, 0), meetsRequirements: meets };
}

/** Items the character owns, as "cat:id" keys (weapons by base id, with best level). */
export function ownedSet(inventory: number[] | undefined): { keys: Set<string>; weaponLevel: Map<number, number> } {
  const keys = new Set<string>();
  const weaponLevel = new Map<number, number>();
  for (const raw of inventory ?? []) {
    const nibble = raw >>> 28;
    const id = raw & 0x0fffffff;
    switch (nibble) {
      case 0: {
        const base = Math.floor(id / 10000) * 10000;
        keys.add(`2:${base}`);
        weaponLevel.set(base, Math.max(weaponLevel.get(base) ?? 0, id % 100));
        break;
      }
      case 1:
        keys.add(`3:${id}`);
        break;
      case 2:
        keys.add(`4:${id}`);
        break;
      case 4:
        keys.add(`1:${id}`);
        break;
      case 8:
        keys.add(`5:${id}`);
        break;
    }
  }
  return { keys, weaponLevel };
}

/** "cat:id" -> pickup ids where it lies on the map (weapons by base id). */
export function locationIndex(items: Map<string, ItemPickup>): Map<string, string[]> {
  const index = new Map<string, string[]>();
  for (const [pid, p] of items) {
    for (const it of p.items) {
      const id = it.cat === 2 ? Math.floor(it.id / 10000) * 10000 : it.id;
      const key = `${it.cat}:${id}`;
      index.set(key, [...(index.get(key) ?? []), pid]);
    }
  }
  return index;
}

export interface UpgradeCost {
  runes: number;
  materials: { name: string; count: number }[];
}

/** Total runes and materials to go from one level to another. */
export function upgradeCost(catalog: Catalog, w: CatalogWeapon, from: number, to: number): UpgradeCost {
  const steps = (catalog.upgrades[w.upgrade] ?? []).filter((s) => s.level > from && s.level <= to);
  const materials = new Map<string, number>();
  let runes = 0;
  for (const s of steps) {
    runes += s.runes;
    for (const m of s.materials) {
      const name = m.name.fr ?? m.name.en;
      materials.set(name, (materials.get(name) ?? 0) + m.count);
    }
  }
  return { runes, materials: [...materials].map(([name, count]) => ({ name, count })) };
}
