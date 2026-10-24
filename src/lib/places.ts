// What lies in a named place of the map (dungeon entrance): its own map block,
// or the surroundings of the entrance for caves drawn inside an open-world tile.
import { DUNGEON_ICONS } from "./route";
import type { Entry, ItemPickup, MarkerPos, Place } from "./types";

/** Map icon -> kind of place (singular, plural). */
export const PLACE_KINDS: Record<number, [string, string]> = {
  4: ["catacombe", "catacombes"],
  13: ["grotte", "grottes"],
  14: ["tunnel", "tunnels"],
  15: ["puits", "puits"],
  16: ["tombe de héros", "tombes de héros"],
  23: ["tour divine", "tours divines"],
  45: ["donjon", "donjons"],
  46: ["donjon", "donjons"],
  210: ["donjon", "donjons"],
  211: ["donjon", "donjons"],
  213: ["donjon", "donjons"],
  217: ["donjon", "donjons"],
  218: ["donjon", "donjons"],
  230: ["catacombe", "catacombes"],
  231: ["geôle", "geôles"],
  232: ["forge en ruine", "forges en ruine"],
  233: ["grotte", "grottes"],
  234: ["grotte", "grottes"],
};

export const isDungeon = (p: Place) => DUNGEON_ICONS.has(p.icon);

const NEARBY = 80;
const isOpen = (block: string) => [60, 61, 12].includes(Number(block.slice(1, 3)));

export interface PlaceContents {
  /** The dungeon's own block, or null when the place is part of an open-world tile. */
  block: string | null;
  bosses: Entry[];
  graces: Entry[];
  items: [string, ItemPickup][];
  /** Set foot inside: inner grace discovered, boss defeated or an item picked up. */
  visited: boolean;
  /** Every boss inside defeated (or visited, when there is no boss). */
  done: boolean;
}

export function placeContents(
  place: Place,
  entries: Entry[],
  positions: Map<string, MarkerPos>,
  items: Map<string, ItemPickup>,
  completed: Set<string>,
): PlaceContents {
  let block: string | null = isOpen(place.block) ? null : place.block;
  if (!block) {
    const grace = entries.find((e) => e.category === "grace" && e.name === place.name.en);
    const g = grace ? positions.get(grace.id) : undefined;
    if (g && !isOpen(g.block)) block = g.block;
  }
  const inside = (b: string | undefined, x: number, y: number, map: string) =>
    block ? b === block || !!b?.startsWith(block) : map === place.map && Math.hypot(x - place.x, y - place.y) < NEARBY;
  const at = (id: string) => {
    const p = positions.get(id);
    return !!p && inside(p.block, p.x, p.y, p.map);
  };
  const bosses = entries.filter((e) => e.category === "boss" && at(e.id));
  // A grace named like the place belongs to it even outside the block (cave entrances).
  const graces = entries.filter((e) => e.category === "grace" && (e.name === place.name.en || (block !== null && at(e.id))));
  const inItems = [...items].filter(([, p]) => inside(p.block, p.x, p.y, p.map));
  const visited =
    graces.some((g) => completed.has(g.id)) || bosses.some((b) => completed.has(b.id)) || inItems.some(([id]) => completed.has(id));
  const done = bosses.length ? bosses.every((b) => completed.has(b.id)) : visited;
  return { block, bosses, graces, items: inItems, visited, done };
}
