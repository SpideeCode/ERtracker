// Data extracted from the player's game install (see src-tauri/src/extract):
// marker positions, boss sheets and French names. Empty until maps are prepared.
import { convertFileSrc } from "@tauri-apps/api/core";
import type { BossSheet, Catalog, Entry, ItemPickup, MarkerPos, Place } from "./types";

// http://ertiles.localhost/ on Windows, ertiles://localhost/ elsewhere.
export const assetBase = convertFileSrc("", "ertiles");

async function fetchJson<T>(file: string): Promise<T | null> {
  try {
    const r = await fetch(assetBase + file);
    return r.ok ? ((await r.json()) as T) : null;
  } catch {
    return null;
  }
}

class GameData {
  /** "boss:10000800" -> position */
  positions = $state(new Map<string, MarkerPos>());
  bosses = $state(new Map<string, BossSheet>());
  names = $state<Record<string, string>>({});
  regions = $state<Record<string, string>>({});
  places = $state<Place[]>([]);
  /** "item:<flag>" -> pickup */
  items = $state(new Map<string, ItemPickup>());
  catalog = $state<Catalog | null>(null);
  loaded = $state(false);

  async load() {
    const [markers, bosses, translations, places, items, catalog] = await Promise.all([
      fetchJson<MarkerPos[]>("markers.json"),
      fetchJson<BossSheet[]>("bosses.json"),
      fetchJson<{ names: Record<string, string>; regions: Record<string, string> }>("translations.json"),
      fetchJson<Place[]>("places.json"),
      fetchJson<ItemPickup[]>("items.json"),
      fetchJson<Catalog>("catalog.json"),
    ]);
    this.catalog = catalog;
    this.items = new Map((items ?? []).map((p) => [`item:${p.flag}`, p]));
    this.places = places ?? [];
    const positions = new Map((markers ?? []).map((m) => [`${m.kind}:${m.flag}`, m]));
    for (const [id, p] of this.items) positions.set(id, { ...p, kind: "item" });
    // Named places (dungeon entrances...) are routed to as open-world points: the entrance itself.
    this.places.forEach((p, i) => positions.set(`place:${i}`, { ...p, flag: i, kind: "place", block: "m60_00_00" }));
    this.positions = positions;
    this.bosses = new Map((bosses ?? []).map((b) => [`boss:${b.flag}`, b]));
    this.names = translations?.names ?? {};
    this.regions = translations?.regions ?? {};
    this.loaded = markers !== null;
  }

  name(entry: Entry): string {
    return this.names[entry.id] ?? entry.name;
  }

  region(entry: Entry): string | null {
    return entry.region ? (this.regions[entry.region] ?? entry.region) : null;
  }
}

export const gamedata = new GameData();
