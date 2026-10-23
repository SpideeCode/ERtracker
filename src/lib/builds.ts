// Build advisor: ranks the game's equipment for a play style, using the
// extracted stats and the official (French and English) descriptions.
import { estimate } from "./equipment";
import type { Catalog, CatalogArmor, CatalogGoods, CatalogTalisman, CatalogWeapon } from "./types";

export interface Theme {
  id: string;
  name: string;
  /** Damage type index (physical, magic, fire, lightning, holy) the weapons should deal. */
  damage: number;
  /** Scaling stat indexes (str, dex, int, fai, arc) the build invests in. */
  stats: number[];
  /** Words (lower case, French and English) that make a talisman, tear or spell relevant. */
  keywords: string[];
  /** Goods types of the spells used: 5/17 sorceries, 16/18 incantations. */
  spells: number[];
  /** Words for spells, when the general keywords are too broad (every incantation mentions faith). */
  spellKeywords?: string[];
  affinity: string;
  advice: string;
}

export const THEMES: Theme[] = [
  {
    id: "holy",
    name: "Sacré (Foi)",
    damage: 4,
    stats: [3],
    keywords: ["holy", "sacré", "sacrée", "incantation", "faith", "foi", "golden order", "ordre d'or"],
    spells: [16, 18],
    spellKeywords: ["holy", "sacré", "sacrée", "golden", "doré", "ordre d'or", "erdtree", "arbre-monde", "lumière", "light"],
    affinity: "Sacrée",
    advice: "Foi à 60 puis 80, Vigueur 50-60, Esprit pour les incantations ; Force/Dextérité au minimum requis par l'arme.",
  },
  {
    id: "magic",
    name: "Magie (Intelligence)",
    damage: 1,
    stats: [2],
    keywords: ["magic", "magie", "glintstone", "pierre d'éclat", "sorcery", "sorcellerie", "sort", "intelligence"],
    spells: [5, 17],
    affinity: "Magique",
    advice: "Intelligence à 60 puis 80, Vigueur 50-60, Esprit pour les sorts ; un bâton à fort scaling Intelligence.",
  },
  {
    id: "fire",
    name: "Feu",
    damage: 2,
    stats: [3, 0],
    keywords: ["fire", "feu", "flame", "flamme", "burn", "brûl"],
    spells: [16, 18],
    affinity: "Feu ou Flamme sacrée",
    advice: "Foi (incantations de feu) ou Force (affinité Feu), Vigueur 50-60.",
  },
  {
    id: "lightning",
    name: "Foudre",
    damage: 3,
    stats: [1, 3],
    keywords: ["lightning", "foudre", "thunder", "électr"],
    spells: [16, 18],
    affinity: "Foudre",
    advice: "Dextérité (affinité Foudre) ou Foi (incantations de foudre), Vigueur 50-60.",
  },
  {
    id: "strength",
    name: "Force",
    damage: 0,
    stats: [0],
    keywords: ["strength", "force", "charged attack", "attaque chargée", "jump", "saut", "heavy", "lourde", "poise", "posture"],
    spells: [],
    affinity: "Lourde",
    advice: "Force à 54 (×1,5 à deux mains = 80) puis 80, Vigueur 50-60, Endurance pour les armes lourdes.",
  },
  {
    id: "dexterity",
    name: "Dextérité",
    damage: 0,
    stats: [1],
    keywords: ["dexterity", "dextérité", "successive", "consécutive", "critical", "critique", "skill", "compétence"],
    spells: [],
    affinity: "Tranchante",
    advice: "Dextérité à 60 puis 80, Vigueur 50-60.",
  },
  {
    id: "bleed",
    name: "Saignement (Arcane)",
    damage: 0,
    stats: [4, 1],
    keywords: ["blood loss", "hémorragie", "saignement", "bleed", "blood", "sang", "arcane"],
    spells: [],
    affinity: "Ensanglantée (ou Occulte)",
    advice: "Arcane à 45-60 pour l'accumulation de saignement, Dextérité pour les dégâts, Vigueur 50-60.",
  },
  {
    id: "frost",
    name: "Givre",
    damage: 1,
    stats: [2, 1],
    keywords: ["frost", "givre", "frostbite", "gelure", "cold", "froid", "glace"],
    spells: [5, 17],
    affinity: "Glacée",
    advice: "Affinité Glacée sur une arme rapide (Dextérité/Intelligence), Vigueur 50-60.",
  },
  {
    id: "rot",
    name: "Poison et putréfaction",
    damage: 0,
    stats: [4],
    keywords: ["poison", "scarlet rot", "putréfaction", "pourriture", "rot", "toxique"],
    spells: [],
    affinity: "Empoisonnée",
    advice: "Arcane pour l'accumulation, Vigueur 50-60 ; résistance (Immunité) contre ses propres effets.",
  },
];

const text = (t: { en: string; fr: string | null } | null | undefined) => `${t?.en ?? ""} ${t?.fr ?? ""}`.toLowerCase();
const relevance = (theme: Theme, ...texts: string[]) => {
  const all = texts.join(" ");
  return theme.keywords.reduce((n, k) => n + (all.includes(k) ? 1 : 0), 0);
};

/** Keeps one entry per English name (the game lists variants and NPC copies). */
function unique<T extends { name: { en: string } }>(list: T[]): T[] {
  const seen = new Set<string>();
  return list.filter((x) => (seen.has(x.name.en) ? false : (seen.add(x.name.en), true)));
}

/** Stats the advice is computed for: the character's, with the theme's stats raised to 60. */
export function targetStats(theme: Theme, stats: number[] | undefined): number[] {
  const s = [...(stats ?? [20, 15, 15, 15, 15, 10, 10, 10])];
  for (const i of theme.stats) s[i + 3] = Math.max(s[i + 3], 60);
  return s;
}

export interface WeaponPick {
  weapon: CatalogWeapon;
  score: number;
  damage: number;
  themed: number;
  meets: boolean;
}

export function bestWeapons(catalog: Catalog, theme: Theme, stats: number[], limit = 12): WeaponPick[] {
  return unique(catalog.weapons.filter((w) => w.weight > 0 && w.requirements.some((r) => r > 0)))
    .flatMap((weapon) => {
      // Damage themes need the element natively; physical themes need scaling in the stat.
      if (theme.damage > 0 && !weapon.attack_max[theme.damage]) return [];
      if (theme.damage === 0 && !theme.stats.some((s) => weapon.scaling_max[s] >= 60)) return [];
      const e = estimate(weapon, stats);
      const themed = theme.damage > 0 ? e.byType[theme.damage] : e.byType[0];
      const scalingBonus = theme.stats.reduce((n, s) => n + weapon.scaling_max[s], 0);
      return [{ weapon, damage: e.total, themed, meets: e.meetsRequirements, score: themed * 1.5 + e.total * 0.5 + scalingBonus }];
    })
    .sort((a, b) => b.score - a.score)
    .slice(0, limit);
}

export function bestTalismans(catalog: Catalog, theme: Theme, limit = 8): CatalogTalisman[] {
  const scored = catalog.talismans
    .map((t) => ({ t, r: relevance(theme, text(t.name), text(t.info)) }))
    .filter((x) => x.r > 0)
    .sort((a, b) => b.r - a.r || b.t.rarity - a.t.rarity || b.t.id - a.t.id);
  // +1/+2/+3 versions share a group: keep the best of each group.
  const groups = new Set<number>();
  return unique(scored.map((x) => x.t))
    .filter((t) => (t.group > 0 && groups.has(t.group) ? false : (groups.add(t.group), true)))
    .slice(0, limit);
}

export function bestGoods(catalog: Catalog, theme: Theme, types: number[], limit = 10, spells = false): CatalogGoods[] {
  const words = spells && theme.spellKeywords ? { ...theme, keywords: theme.spellKeywords } : theme;
  return unique(
    catalog.goods
      .filter((g) => types.includes(g.goods_type))
      .map((g) => ({ g, r: relevance(words, text(g.name), text(g.info)) }))
      .filter((x) => x.r > 0 || (spells && theme.id === "magic"))
      .sort((a, b) => b.r - a.r || b.g.rarity - a.g.rarity)
      .map((x) => x.g),
  ).slice(0, limit);
}

export function bestAshesOfWar(catalog: Catalog, theme: Theme, limit = 6) {
  return unique(
    catalog.ashes_of_war
      .map((a) => ({ a, r: relevance(theme, text(a.name), text(a.info)) }))
      .filter((x) => x.r > 0)
      .sort((a, b) => b.r - a.r)
      .map((x) => x.a),
  ).slice(0, limit);
}

/** Per slot: best protection, and best protection per unit of weight. */
export function bestArmor(catalog: Catalog): { slot: number; heavy: CatalogArmor[]; light: CatalogArmor[] }[] {
  const total = (a: CatalogArmor) => a.absorption.reduce((x, y) => x + y, 0);
  const wearable = unique(catalog.armor.filter((a) => a.weight > 0));
  return [0, 1, 2, 3].map((slot) => {
    const list = wearable.filter((a) => a.slot === slot);
    return {
      slot,
      heavy: [...list].sort((a, b) => total(b) - total(a)).slice(0, 3),
      light: [...list].sort((a, b) => total(b) / (b.weight + 1) - total(a) / (a.weight + 1)).slice(0, 3),
    };
  });
}
