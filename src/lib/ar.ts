// Attack rating, computed like the game does from the extracted params:
// base attack × upgrade multiplier, plus a bonus per scaling stat read on the
// weapon's curves (CalcCorrectGraph), for the stats each damage type uses
// (AttackElementCorrectParam). An unmet requirement turns that damage type's
// bonus into a 40 % penalty.
import type { Catalog, CatalogWeapon, Curve, WeaponVariant } from "./types";

/** Character stat index (vigor, mind, endurance, str, dex, int, fai, arc) of each scaling stat. */
export const SCALING_STATS = [3, 4, 5, 6, 7];
export const STAT_NAMES = ["Vigueur", "Esprit", "Endurance", "Force", "Dextérité", "Intelligence", "Foi", "Arcane"];
export const STAT_SHORT = ["Vig", "Esp", "End", "For", "Dex", "Int", "Foi", "Arc"];

/** French names of the weapon types (wepType). */
export const WEAPON_KINDS: Record<number, string> = {
  1: "Dagues",
  3: "Épées droites",
  5: "Grandes épées",
  7: "Épées colossales",
  9: "Épées courbes",
  11: "Grandes épées courbes",
  13: "Katanas",
  14: "Épées doubles",
  15: "Épées d'estoc",
  16: "Épées d'estoc lourdes",
  17: "Haches",
  19: "Grandes haches",
  21: "Marteaux",
  23: "Grands marteaux",
  24: "Fléaux",
  25: "Lances",
  28: "Grandes lances",
  29: "Hallebardes",
  31: "Faux",
  35: "Poings",
  37: "Griffes",
  39: "Fouets",
  41: "Armes colossales",
  50: "Arcs courts",
  51: "Arcs",
  53: "Grands arcs",
  55: "Arbalètes",
  56: "Balistes",
  88: "Arts martiaux",
  91: "Lames de lancer",
  92: "Lames inversées",
  93: "Grandes épées légères",
  94: "Grands katanas",
  95: "Griffes de bête",
};

/** Bows, crossbows and ballistas get nothing from holding them with both hands. */
const NO_TWO_HAND_BONUS = new Set([50, 51, 53, 55, 56]);
/** Glintstone staffs and sacred seals: their number is a spell power, not a melee hit. */
export const CATALYSTS = new Set([57, 61]);
/** Arrows, bolts and the bare fist are not weapons to build around. */
const NOT_WEAPONS = new Set([33, 81, 83, 85, 86]);

export function isBuildWeapon(w: CatalogWeapon): boolean {
  return !NOT_WEAPONS.has(w.kind) && w.variants.length > 0;
}

/** Somber (unique) weapons go to +10, the others to +25: the same smithing effort. */
export function equivalentLevel(level: number, fromMax: number, toMax: number): number {
  if (fromMax === toMax || !fromMax) return Math.min(level, toMax);
  return Math.min(toMax, Math.round((level * toMax) / fromMax));
}

function curveValue(c: Curve | undefined, x: number): number {
  if (!c) return 0;
  const { stat, grow, exp } = c;
  if (x <= stat[0]) return grow[0] / 100;
  for (let i = 0; i < 4; i++) {
    if (x > stat[i + 1] && i < 3) continue;
    const span = stat[i + 1] - stat[i];
    let ratio = span > 0 ? Math.min(1, (x - stat[i]) / span) : 1;
    const e = exp[i];
    ratio = e > 0 ? ratio ** e : 1 - (1 - ratio) ** -e;
    return (grow[i] + (grow[i + 1] - grow[i]) * ratio) / 100;
  }
  return grow[4] / 100;
}

export interface AttackRating {
  /** Physical, magic, fire, lightning, holy. */
  byType: number[];
  total: number;
  /** Stats below the weapon's requirements, as character stat indexes. */
  missing: number[];
}

/**
 * @param stats the eight character stats
 * @param level upgrade level (+0...+25, or +10 for somber weapons)
 */
export function attackRating(
  catalog: Catalog,
  weapon: CatalogWeapon,
  variant: WeaponVariant,
  level: number,
  stats: number[],
  twoHanded = false,
): AttackRating {
  const levels = catalog.reinforce[variant.reinforce] ?? [];
  const rate = levels[Math.min(level, levels.length - 1)] ?? { attack: [1, 1, 1, 1, 1], scaling: [1, 1, 1, 1, 1] };
  const elements = catalog.elements[variant.elements];
  const effective = SCALING_STATS.map((i) => stats[i] ?? 10);
  if (twoHanded && !NO_TWO_HAND_BONUS.has(weapon.kind)) effective[0] = Math.floor(effective[0] * 1.5);
  const missing = weapon.requirements.flatMap((req, s) => (effective[s] < req ? [SCALING_STATS[s]] : []));

  const byType = variant.attack.map((base0, t) => {
    if (!base0) return 0;
    const base = base0 * rate.attack[t];
    const uses = elements?.uses[t] ?? [];
    const used = [0, 1, 2, 3, 4].filter((s) => uses[s]);
    if (used.some((s) => effective[s] < weapon.requirements[s])) return Math.floor(base * 0.6);
    let bonus = 0;
    for (const s of used) {
      const over = elements?.overwrite[t][s] ?? -1;
      const scaling = ((over >= 0 ? over : variant.scaling[s]) * rate.scaling[s]) / 100;
      bonus += base * scaling * curveValue(catalog.curves[variant.curves[t]], effective[s]);
    }
    return Math.floor(base + bonus);
  });
  return { byType, total: byType.reduce((a, b) => a + b, 0), missing };
}

export const AFFINITY_WHETBLADE: Record<number, string> = {
  1: "whetblade:65610",
  2: "whetblade:65610",
  3: "whetblade:65610",
  4: "whetblade:65640",
  5: "whetblade:65640",
  6: "whetblade:65660",
  7: "whetblade:65660",
  8: "whetblade:65680",
  9: "whetblade:65680",
  10: "whetblade:65700",
  11: "whetblade:65700",
  12: "whetblade:65700",
};
/** Needed to change a weapon's Ash of War at all. */
export const WHETSTONE_KNIFE = "whetblade:60130";

export function variantName(w: CatalogWeapon, v: WeaponVariant): string {
  return v.name?.fr ?? v.name?.en ?? w.name.fr ?? w.name.en;
}

/** Character level for a set of stats (every class sums to level + 79). */
export const levelOf = (stats: number[]) => stats.reduce((a, b) => a + b, 0) - 79;

/**
 * Spreads `points` over the scaling stats to get the most out of a weapon:
 * requirements first, then a few points at a time where they add the most.
 */
export function bestStats(
  catalog: Catalog,
  weapon: CatalogWeapon,
  variant: WeaponVariant,
  level: number,
  base: number[],
  points: number,
  twoHanded = false,
): { stats: number[]; ar: AttackRating } {
  const stats = [...base];
  let left = points;
  const reqStr = twoHanded && !NO_TWO_HAND_BONUS.has(weapon.kind) ? Math.ceil(weapon.requirements[0] / 1.5) : weapon.requirements[0];
  const reqs = [reqStr, ...weapon.requirements.slice(1)];
  SCALING_STATS.forEach((i, s) => {
    const need = Math.max(0, reqs[s] - stats[i]);
    const add = Math.min(need, left);
    stats[i] += add;
    left -= add;
  });
  let ar = attackRating(catalog, weapon, variant, level, stats, twoHanded);
  const STEP = 3;
  while (left > 0) {
    let best = -1;
    let bestAr = ar;
    for (const i of SCALING_STATS) {
      if (stats[i] >= 99) continue;
      const add = Math.min(STEP, left, 99 - stats[i]);
      stats[i] += add;
      const next = attackRating(catalog, weapon, variant, level, stats, twoHanded);
      stats[i] -= add;
      if (next.total > bestAr.total) {
        best = i;
        bestAr = next;
      }
    }
    if (best < 0) break;
    const add = Math.min(STEP, left, 99 - stats[best]);
    stats[best] += add;
    left -= add;
    ar = bestAr;
  }
  return { stats, ar };
}
