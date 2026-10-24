// Spoiler-free exploration hints: the world is cut into zones (a region, split
// into sectors around its centre) and each zone tells how much is left to find
// in it, without naming anything.
import { heading } from "./route";
import type { Entry, ItemPickup, MapId, MarkerPos } from "./types";

export interface ZoneContent {
  id: string;
  kind: "boss" | "grace" | "item" | "dungeon";
  /** For dungeons: catacombe, grotte, tunnel... */
  subtype?: string;
  pos: { map: string; x: number; y: number };
  done: boolean;
  /** How much it matters: bosses and legendary items weigh more. */
  weight: number;
}

export interface Zone {
  key: string;
  map: MapId;
  /** English region name, e.g. "Caelid". */
  region: string;
  /** "nord", "sud-est"... or "centre". */
  sector: string;
  x: number;
  y: number;
  radius: number;
  contents: ZoneContent[];
  left: { boss: number; grace: number; item: number; dungeon: number };
  total: { boss: number; grace: number; item: number; dungeon: number };
  /** Dungeons never visited, by kind. */
  dungeonsLeft: Record<string, number>;
  /** Graces discovered in the zone: the player has been there. */
  visited: boolean;
  /** Remaining weight. */
  score: number;
}

const SECTOR_FRACTION = 0.35;

export interface ZoneInput {
  bosses: Entry[];
  graces: Entry[];
  items: [string, ItemPickup][];
  positions: Map<string, MarkerPos>;
  completed: Set<string>;
  /** Item pickups worth pointing at (no materials or runes). */
  important: (p: ItemPickup) => boolean;
  /** 0-3 */
  rarity: (p: ItemPickup) => number;
  /** Dungeon entrances; "done" means visited. */
  dungeons: { id: string; subtype: string; pos: { map: string; x: number; y: number }; done: boolean }[];
}

export function buildZones(input: ZoneInput): Zone[] {
  const { positions, completed } = input;

  // Region of every grace, and each region's centre and size per map.
  const graces = input.graces.flatMap((g) => {
    const p = positions.get(g.id);
    return p && g.region ? [{ g, p }] : [];
  });
  const regionKey = (map: string, region: string) => `${map}|${region}`;
  const regions = new Map<string, { map: string; region: string; xs: number[]; ys: number[] }>();
  for (const { g, p } of graces) {
    const k = regionKey(p.map, g.region!);
    const r = regions.get(k) ?? { map: p.map, region: g.region!, xs: [], ys: [] };
    r.xs.push(p.x);
    r.ys.push(p.y);
    regions.set(k, r);
  }
  const centres = new Map(
    [...regions].map(([k, r]) => {
      const cx = r.xs.reduce((a, b) => a + b, 0) / r.xs.length;
      const cy = r.ys.reduce((a, b) => a + b, 0) / r.ys.length;
      const spread = Math.max(200, ...r.xs.map((x, i) => Math.hypot(x - cx, r.ys[i] - cy)));
      return [k, { cx, cy, spread }];
    }),
  );

  const nearestRegion = (map: string, x: number, y: number) => {
    let best: { region: string; d: number } | null = null;
    for (const { g, p } of graces) {
      if (p.map !== map) continue;
      const d = Math.hypot(p.x - x, p.y - y);
      if (!best || d < best.d) best = { region: g.region!, d };
    }
    return best?.region ?? null;
  };

  const contents: (ZoneContent & { region: string })[] = [];
  const add = (c: ZoneContent) => {
    const region = nearestRegion(c.pos.map, c.pos.x, c.pos.y);
    if (region) contents.push({ ...c, region });
  };
  for (const b of input.bosses) {
    const p = positions.get(b.id);
    if (p) add({ id: b.id, kind: "boss", pos: p, done: completed.has(b.id), weight: 4 });
  }
  for (const { g, p } of graces) add({ id: g.id, kind: "grace", pos: p, done: completed.has(g.id), weight: 1 });
  for (const [id, p] of input.items) {
    if (!input.important(p)) continue;
    add({ id, kind: "item", pos: p, done: completed.has(id), weight: 1 + input.rarity(p) });
  }
  for (const d of input.dungeons) add({ id: d.id, kind: "dungeon", subtype: d.subtype, pos: d.pos, done: d.done, weight: 3 });

  const zones = new Map<string, Zone>();
  for (const c of contents) {
    const rk = regionKey(c.pos.map, c.region);
    const centre = centres.get(rk);
    if (!centre) continue;
    const dx = c.pos.x - centre.cx;
    const dy = c.pos.y - centre.cy;
    const sector = Math.hypot(dx, dy) < centre.spread * SECTOR_FRACTION ? "centre" : heading(dx, dy);
    const key = `${rk}|${sector}`;
    const z =
      zones.get(key) ??
      ({
        key,
        map: c.pos.map as MapId,
        region: c.region,
        sector,
        x: 0,
        y: 0,
        radius: 0,
        contents: [],
        left: { boss: 0, grace: 0, item: 0, dungeon: 0 },
        total: { boss: 0, grace: 0, item: 0, dungeon: 0 },
        dungeonsLeft: {},
        visited: false,
        score: 0,
      } satisfies Zone);
    z.contents.push(c);
    z.total[c.kind]++;
    if (!c.done) {
      z.left[c.kind]++;
      z.score += c.weight;
      if (c.subtype) z.dungeonsLeft[c.subtype] = (z.dungeonsLeft[c.subtype] ?? 0) + 1;
    }
    if (c.kind === "grace" && c.done) z.visited = true;
    zones.set(key, z);
  }

  for (const z of zones.values()) {
    z.x = z.contents.reduce((a, c) => a + c.pos.x, 0) / z.contents.length;
    z.y = z.contents.reduce((a, c) => a + c.pos.y, 0) / z.contents.length;
    z.radius = Math.max(120, ...z.contents.map((c) => Math.hypot(c.pos.x - z.x, c.pos.y - z.y)));
  }
  return [...zones.values()];
}

/** "Est de Caelid", "Centre de Liurnia". */
export function zoneName(z: Zone, region: string): string {
  const s = z.sector === "centre" ? "Centre" : z.sector.charAt(0).toUpperCase() + z.sector.slice(1);
  const de = /^[aeiouyéèêàâîôû]/i.test(region) ? "d'" : "de ";
  return `${s} ${de}${region}`;
}

/** 0-1 share of what the zone holds that is done. */
export const progress = (z: Zone) => {
  const total = z.contents.reduce((a, c) => a + c.weight, 0);
  return total ? 1 - z.score / total : 1;
};
