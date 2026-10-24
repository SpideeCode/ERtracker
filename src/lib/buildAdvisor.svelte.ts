// Build advisor from what the character owns: the best weapons right now, what
// one, two or three more items (weapon, whetblade, Ash of War) would bring, where
// to put the next levels, and the strongest builds the game allows.
import { gamedata } from "./gamedata.svelte";
import { tracker } from "./tracker.svelte";
import { catalog } from "./catalog.svelte";
import {
  AFFINITY_WHETBLADE,
  CATALYSTS,
  SCALING_STATS,
  WHETSTONE_KNIFE,
  attackRating,
  bestStats,
  equivalentLevel,
  isBuildWeapon,
  levelOf,
  variantName,
  type AttackRating,
} from "./ar";
import type { AshOfWar, Catalog, CatalogWeapon, WeaponVariant } from "./types";

/** Shields, torches and perfume bottles are left out of the melee rankings. */
const SIDE_KINDS = new Set([65, 67, 69, 87, 89, 90]);

export interface Need {
  kind: "weapon" | "whetblade" | "ash" | "knife";
  label: string;
  /** What to open to find it: pickup ids on the map, or a whetblade id. */
  where: string[];
}

export interface Option {
  weapon: CatalogWeapon;
  variant: WeaponVariant;
  name: string;
  level: number;
  twoHanded: boolean;
  ar: AttackRating;
  /** Held in one hand, as the equipment screen shows it by default. */
  oneHand: number;
  /** Items still to get for this option. */
  needs: Need[];
  /** Ash of War to put on it, when the affinity changes. */
  ash: AshOfWar | null;
}

interface Owned {
  weapons: Map<number, { level: number; affinity: number }>;
  ashes: Set<number>;
}

function owned(inventory: number[]): Owned {
  const weapons = new Map<number, { level: number; affinity: number }>();
  const ashes = new Set<number>();
  for (const raw of inventory) {
    const nibble = raw >>> 28;
    const id = raw & 0x0fffffff;
    if (nibble === 8) ashes.add(id);
    if (nibble !== 0) continue;
    const base = Math.floor(id / 10000) * 10000;
    const level = id % 100;
    const affinity = Math.floor((id % 10000) / 100);
    const prev = weapons.get(base);
    if (!prev || level > prev.level) weapons.set(base, { level, affinity });
  }
  return { weapons, ashes };
}

const goodsByName = (en: string) => gamedata.catalog?.goods.find((g) => g.name.en === en);
const WHETBLADE_NAMES: Record<string, string> = {
  "whetblade:60130": "Whetstone Knife",
  "whetblade:65610": "Iron Whetblade",
  "whetblade:65640": "Red-Hot Whetblade",
  "whetblade:65660": "Sanctified Whetblade",
  "whetblade:65680": "Glintstone Whetblade",
  "whetblade:65700": "Black Whetblade",
};

function whetbladeNeed(id: string): Need {
  const g = goodsByName(WHETBLADE_NAMES[id]);
  return {
    kind: id === WHETSTONE_KNIFE ? "knife" : "whetblade",
    label: g ? (g.name.fr ?? g.name.en) : WHETBLADE_NAMES[id],
    where: g ? catalog.whereIs(1, g.id) : [],
  };
}

/** Both ways of holding it; the stronger one wins. */
function rate(c: Catalog, w: CatalogWeapon, v: WeaponVariant, level: number, stats: number[]) {
  const one = attackRating(c, w, v, level, stats, false);
  const two = attackRating(c, w, v, level, stats, true);
  return two.total > one.total ? { ar: two, twoHanded: true, oneHand: one.total } : { ar: one, twoHanded: false, oneHand: one.total };
}

class BuildAdvisor {
  stats = $derived(tracker.character?.stats ?? [10, 10, 10, 10, 10, 10, 10, 10]);
  own = $derived(owned(tracker.character?.inventory ?? []));
  ready = $derived(!!gamedata.catalog?.reinforce && !!tracker.character);

  /** Highest upgrade reached on each track: what a new weapon could be brought to. */
  reach = $derived.by(() => {
    let regular = 0;
    let somber = 0;
    for (const [base, o] of this.own.weapons) {
      const w = catalog.weapons.get(base);
      if (!w) continue;
      if (w.max_level > 10) regular = Math.max(regular, o.level);
      else somber = Math.max(somber, o.level);
    }
    return {
      regular: Math.max(regular, equivalentLevel(somber, 10, 25)),
      somber: Math.max(somber, equivalentLevel(regular, 25, 10)),
    };
  });

  /** Every weapon and affinity, with what is missing to use it. */
  options = $derived.by((): Option[] => {
    const c = gamedata.catalog;
    if (!c?.reinforce || !tracker.character) return [];
    const done = tracker.completed;
    const ashes = c.ashes_of_war;
    const out: Option[] = [];
    for (const w of c.weapons) {
      if (!isBuildWeapon(w) || SIDE_KINDS.has(w.kind) || CATALYSTS.has(w.kind)) continue;
      const mine = this.own.weapons.get(w.id);
      const level = mine ? mine.level : w.max_level > 10 ? this.reach.regular : this.reach.somber;
      for (const v of w.variants) {
        const current = mine ? mine.affinity : 0;
        const needs: Need[] = [];
        let ash: AshOfWar | null = null;
        if (!mine) {
          needs.push({ kind: "weapon", label: w.name.fr ?? w.name.en, where: catalog.whereIs(2, w.id) });
        }
        if (v.affinity !== current) {
          if (!w.infusable) continue;
          if (!done.has(WHETSTONE_KNIFE)) needs.push(whetbladeNeed(WHETSTONE_KNIFE));
          const blade = AFFINITY_WHETBLADE[v.affinity];
          if (blade && !done.has(blade)) needs.push(whetbladeNeed(blade));
          const fits = ashes.filter((a) => a.mounts.includes(w.kind) && a.affinities.includes(v.affinity));
          ash = fits.find((a) => this.own.ashes.has(a.id)) ?? null;
          if (!ash) {
            ash = fits.find((a) => catalog.whereIs(5, a.id).length) ?? fits[0] ?? null;
            if (!ash) continue;
            needs.push({ kind: "ash", label: (ash.name.fr ?? ash.name.en).replace(/^(Cendres de guerre|Ash of War)\s*:\s*/, ""), where: catalog.whereIs(5, ash.id) });
          }
        }
        const { ar, twoHanded, oneHand } = rate(c, w, v, Math.min(level, w.max_level), this.stats);
        out.push({ weapon: w, variant: v, name: variantName(w, v), level: Math.min(level, w.max_level), twoHanded, ar, oneHand, needs, ash });
      }
    }
    return out.sort((a, b) => b.ar.total - a.ar.total);
  });

  /** Weapon type the player wants to play, or null for any. */
  kind = $state<number | null>(null);
  /** Weapon types present in the options, for the filter. */
  kinds = $derived([...new Set(this.options.map((o) => o.weapon.kind))].sort((a, b) => a - b));
  private shown = $derived(this.kind === null ? this.options : this.options.filter((o) => o.weapon.kind === this.kind));

  /** Best affinity of each weapon usable right now. */
  current = $derived(bestPerWeapon(this.shown.filter((o) => o.needs.length === 0)));

  /** The strongest options needing 1, 2 or 3 more items, better or not than what is in hand. */
  upgrades = $derived([1, 2, 3].map((n) => bestPerWeapon(this.shown.filter((o) => o.needs.length === n)).slice(0, 6)));

  /** Weapons in hand not fully upgraded: what the forge would bring, best first. */
  smithing = $derived.by(() => {
    const c = gamedata.catalog;
    if (!c) return [];
    return this.current
      .filter((o) => o.level < o.weapon.max_level)
      .map((o) => ({ o, max: rate(c, o.weapon, o.variant, o.weapon.max_level, this.stats) }))
      .sort((a, b) => b.max.ar.total - a.max.ar.total)
      .slice(0, 6);
  });

  /** Where the next `points` levels go for a weapon, and what they bring. */
  nextLevels(o: Option, points: number) {
    const c = gamedata.catalog!;
    return bestStats(c, o.weapon, o.variant, o.level, this.stats, points, o.twoHanded);
  }

  /** Same level, scaling stats spread again from 10 (Larval Tear rebirth; the class sets the real floor). */
  respec(o: Option) {
    const c = gamedata.catalog!;
    const base = this.stats.map((v, i) => (SCALING_STATS.includes(i) ? Math.min(v, 10) : v));
    const points = this.stats.reduce((a, b) => a + b, 0) - base.reduce((a, b) => a + b, 0);
    return bestStats(c, o.weapon, o.variant, o.level, base, points, o.twoHanded);
  }
}

function bestPerWeapon(list: Option[]): Option[] {
  const seen = new Set<number>();
  return list.filter((o) => (seen.has(o.weapon.id) ? false : (seen.add(o.weapon.id), true)));
}

export interface TopBuild {
  weapon: CatalogWeapon;
  variant: WeaponVariant;
  name: string;
  stats: number[];
  ar: AttackRating;
  twoHanded: boolean;
  /** Main stats, e.g. "Force/Foi". */
  archetype: string;
}

const ARCHETYPE_STATS = ["Force", "Dextérité", "Intelligence", "Foi", "Arcane"];

/** Strongest weapon + affinity + stat spread at a character level, fully upgraded. Slow: call on demand. */
export function topBuilds(c: Catalog, level: number): TopBuild[] {
  // Survivable base: vigor 60, mind 20, endurance 25, scaling stats from 10.
  const total = level + 79;
  const vigor = Math.max(20, Math.min(60, total - 95));
  const base = [vigor, 20, 25, 10, 10, 10, 10, 10];
  const points = Math.max(0, total - base.reduce((a, b) => a + b, 0));
  const out: TopBuild[] = [];
  for (const w of c.weapons) {
    if (!isBuildWeapon(w) || SIDE_KINDS.has(w.kind) || CATALYSTS.has(w.kind)) continue;
    for (const v of w.variants) {
      let best: TopBuild | null = null;
      for (const twoHanded of [false, true]) {
        const r = bestStats(c, w, v, w.max_level, base, points, twoHanded);
        if (!best || r.ar.total > best.ar.total) {
          const main = SCALING_STATS.map((i, s) => [s, r.stats[i]] as const).filter(([, n]) => n >= 30);
          best = {
            weapon: w,
            variant: v,
            name: variantName(w, v),
            stats: r.stats,
            ar: r.ar,
            twoHanded,
            archetype: main.length ? main.map(([s]) => ARCHETYPE_STATS[s]).join("/") : "Équilibré",
          };
        }
      }
      if (best) out.push(best);
    }
  }
  return out.sort((a, b) => b.ar.total - a.ar.total);
}

export const advisor = new BuildAdvisor();
export { levelOf };
