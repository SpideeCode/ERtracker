// Armour advice: the best head/chest/arms/legs under a weight budget, so the
// character keeps the roll they want. Absorptions multiply (1 - Π(1 - a)), so
// their logs add up, which makes it an exact multiple-choice knapsack over weight.
import type { CatalogArmor } from "./types";

/** Equip load from Endurance (game curve: 45 at 8, 72 at 25, 120 at 60, 160 at 99). */
export function equipLoad(endurance: number): number {
  const e = Math.max(8, Math.min(99, endurance));
  if (e <= 25) return 45 + 27 * ((e - 8) / 17) ** 1.1;
  if (e <= 60) return 72 + 48 * ((e - 25) / 35);
  return 120 + 40 * ((e - 60) / 39);
}

export type Roll = "light" | "medium";
export const ROLL_LIMIT: Record<Roll, number> = { light: 0.3, medium: 0.7 };
export type Goal = "defense" | "poise";

export interface ArmorSet {
  pieces: CatalogArmor[];
  weight: number;
  /** % of physical damage absorbed (average of standard, strike, slash, pierce). */
  physical: number;
  /** % of magic, fire, lightning and holy absorbed (average). */
  elemental: number;
  poise: number;
}

const avg = (a: number[]) => a.reduce((s, v) => s + v, 0) / a.length;
const physOf = (p: CatalogArmor) => avg(p.absorption.slice(0, 4));
const elemOf = (p: CatalogArmor) => avg(p.absorption.slice(4, 8));
const stack = (list: number[]) => (1 - list.reduce((k, v) => k * (1 - v / 100), 1)) * 100;

function value(p: CatalogArmor, goal: Goal): number {
  const defense = -Math.log(1 - physOf(p) / 100) * 100 - Math.log(1 - elemOf(p) / 100) * 40;
  return goal === "poise" ? (p.poise ?? 0) * 3 + defense * 0.3 : defense + (p.poise ?? 0) * 0.15;
}

/**
 * Best set under `budget` kg. Pieces with no name worth showing (the "bare"
 * slots) are allowed: a slot can stay empty.
 */
export function bestSet(armor: CatalogArmor[], budget: number, goal: Goal): ArmorSet | null {
  const STEP = 10; // 0.1 kg units
  const cap = Math.max(0, Math.floor(budget * STEP));
  const slots = [0, 1, 2, 3].map((s) => armor.filter((p) => p.slot === s));
  // dp[w] = best value with weight exactly <= w; keep choices to rebuild the set.
  let dp = new Float64Array(cap + 1);
  const choice: Int32Array[] = [];
  for (const list of slots) {
    const next = new Float64Array(cap + 1).fill(-Infinity);
    const pick = new Int32Array(cap + 1).fill(-1);
    for (let w = 0; w <= cap; w++) {
      // Leaving the slot empty.
      if (dp[w] > next[w]) {
        next[w] = dp[w];
        pick[w] = -1;
      }
      list.forEach((p, i) => {
        const pw = Math.round(p.weight * STEP);
        if (pw > w) return;
        const v = dp[w - pw] + value(p, goal);
        if (v > next[w]) {
          next[w] = v;
          pick[w] = i;
        }
      });
    }
    choice.push(pick);
    dp = next;
  }
  if (!Number.isFinite(dp[cap])) return null;
  const pieces: CatalogArmor[] = [];
  let w = cap;
  for (let s = 3; s >= 0; s--) {
    const i = choice[s][w];
    if (i >= 0) {
      const p = slots[s][i];
      pieces.unshift(p);
      w -= Math.round(p.weight * STEP);
    }
  }
  return {
    pieces,
    weight: pieces.reduce((s, p) => s + p.weight, 0),
    physical: stack(pieces.map(physOf)),
    elemental: stack(pieces.map(elemOf)),
    poise: pieces.reduce((s, p) => s + (p.poise ?? 0), 0),
  };
}
