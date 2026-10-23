// Lookups over the extracted equipment catalogue, plus what the current
// character owns and where things lie on the map.
import { gamedata } from "./gamedata.svelte";
import { tracker } from "./tracker.svelte";
import { locationIndex, ownedSet } from "./equipment";
import type { CatalogArmor, CatalogGoods, CatalogTalisman, CatalogWeapon } from "./types";

const byId = <T extends { id: number }>(list: T[] | undefined) => new Map((list ?? []).map((x) => [x.id, x]));

class CatalogLookup {
  weapons = $derived(byId<CatalogWeapon>(gamedata.catalog?.weapons));
  armor = $derived(byId<CatalogArmor>(gamedata.catalog?.armor));
  talismans = $derived(byId<CatalogTalisman>(gamedata.catalog?.talismans));
  goods = $derived(byId<CatalogGoods>(gamedata.catalog?.goods));
  ashesOfWar = $derived(byId(gamedata.catalog?.ashes_of_war));

  owned = $derived(ownedSet(tracker.character?.inventory));
  /** Goods names owned (spirit ash upgrades change the id, not the name). */
  ownedGoodsNames = $derived(
    new Set(
      [...this.owned.keys]
        .filter((k) => k.startsWith("1:"))
        .map((k) => this.goods.get(Number(k.slice(2)))?.name.en)
        .filter(Boolean),
    ),
  );
  locations = $derived(locationIndex(gamedata.items));

  /** cat: 1 goods, 2 weapon, 3 armour, 4 talisman, 5 ash of war */
  isOwned(cat: number, id: number): boolean {
    if (this.owned.keys.has(`${cat}:${id}`)) return true;
    if (cat === 1) {
      const name = this.goods.get(id)?.name.en;
      return !!name && this.ownedGoodsNames.has(name);
    }
    return false;
  }

  /** Pickup ids where this item lies on the map. */
  whereIs(cat: number, id: number): string[] {
    return this.locations.get(`${cat}:${cat === 2 ? Math.floor(id / 10000) * 10000 : id}`) ?? [];
  }

  rarity(cat: number, id: number): number {
    switch (cat) {
      case 1:
        return this.goods.get(id)?.rarity ?? 0;
      case 2:
        return this.weapons.get(Math.floor(id / 10000) * 10000)?.rarity ?? 0;
      case 3:
        return this.armor.get(id)?.rarity ?? 0;
      case 4:
        return this.talismans.get(id)?.rarity ?? 0;
      default:
        return 0;
    }
  }
}

export const catalog = new CatalogLookup();
