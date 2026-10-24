// Zones computed once from the current save and game data, shared by the
// home page, the explore page and the overlay.
import { db, tracker } from "./tracker.svelte";
import { gamedata } from "./gamedata.svelte";
import { catalog } from "./catalog.svelte";
import { categoryById, pickupCategory } from "./items";
import { PLACE_KINDS, isDungeon, placeContents } from "./places";
import { locked } from "./accessCheck.svelte";
import { buildZones, zoneName, type Zone } from "./zones";

export interface RankedZone {
  z: Zone;
  /** Metres from the player, when on the same map. */
  distance: number | null;
  locked: boolean;
}

class ZoneData {
  dungeons = $derived(
    gamedata.places.flatMap((p, i) => {
      if (!isDungeon(p)) return [];
      const c = placeContents(p, db.entries, gamedata.positions, gamedata.items, tracker.completed);
      return [{ id: `place:${i}`, subtype: PLACE_KINDS[p.icon]?.[0] ?? "donjon", pos: p, done: c.visited }];
    }),
  );

  zones = $derived(
    gamedata.loaded
      ? buildZones({
          bosses: db.entries.filter((e) => e.category === "boss"),
          graces: db.entries.filter((e) => e.category === "grace"),
          items: [...gamedata.items],
          positions: gamedata.positions,
          completed: tracker.completed,
          important: (p) => !!categoryById.get(pickupCategory(p))?.important,
          rarity: (p) => Math.max(...p.items.map((i) => catalog.rarity(i.cat, i.id))),
          dungeons: this.dungeons,
        })
      : [],
  );

  ranked = $derived.by((): RankedZone[] => {
    const me = tracker.snapshot?.map_position ?? null;
    return this.zones
      .filter((z) => z.score > 0)
      .map((z) => ({ z, distance: me && me.map === z.map ? Math.hypot(z.x - me.x, z.y - me.y) : null, locked: this.isLocked(z) }));
  });

  /** Reachable zones worth going to: close or already started, with the most left. */
  suggestions = $derived(
    this.ranked
      .filter((r) => !r.locked)
      .sort((a, b) => rank(b) - rank(a)),
  );

  /** The zone the player stands in. */
  current = $derived.by(() => {
    const me = tracker.snapshot?.map_position;
    if (!me) return null;
    return (
      this.zones
        .filter((z) => z.map === me.map)
        .map((z) => ({ z, d: Math.hypot(z.x - me.x, z.y - me.y) / z.radius }))
        .sort((a, b) => a.d - b.d)[0]?.z ?? null
    );
  });

  isLocked(z: Zone): boolean {
    const boss = db.entries.find((e) => e.category === "boss" && e.region === z.region);
    return boss ? locked(boss) && !z.visited : false;
  }

  name(z: Zone): string {
    return zoneName(z, gamedata.regions[z.region] ?? z.region);
  }
}

function rank(r: RankedZone) {
  const near = r.distance === null ? 0.6 : Math.max(0.3, 1 - r.distance / 4000);
  return r.z.score * near * (r.z.visited ? 1.3 : 1);
}

export const zoneData = new ZoneData();
