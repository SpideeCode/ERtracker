// Illustrations cut from the player's own game by the backend on first use:
// item artwork by icon id, and loading screen scenes for regions and bosses.
import raw from "../../data/illustrations.json";
import { assetBase } from "./gamedata.svelte";
import type { Catalog, Entry } from "./types";

const table = raw as { regions: Record<string, number>; bosses: Record<string, number>; generic: number[] };

export const iconUrl = (icon: number | null | undefined) => (icon ? `${assetBase}img/icon/${icon}.png` : null);
export const sceneUrl = (n: number) => `${assetBase}img/load/${n}.jpg`;

/** Scene of a region (English name), or a generic one picked from the name so it stays the same. */
export function regionScene(region: string | null | undefined): string {
  if (region && region in table.regions) return sceneUrl(table.regions[region]);
  const key = [...(region ?? "")].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7);
  return sceneUrl(table.generic[key % table.generic.length]);
}

/** A boss's own scene when the game has one, null otherwise. */
export function bossScene(boss: Entry): string | null {
  const hit = Object.entries(table.bosses).find(([name]) => boss.name.startsWith(name));
  return hit ? sceneUrl(hit[1]) : null;
}

/** Artwork of the remembrance (or other drop) a boss gives, by its English name. */
export function dropIcon(catalog: Catalog | null, nameEn: string | null | undefined): string | null {
  if (!catalog || !nameEn) return null;
  const g = catalog.goods.find((x) => x.name.en === nameEn);
  return iconUrl(g?.icon);
}
