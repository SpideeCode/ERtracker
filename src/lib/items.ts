// Item pickups extracted from the game (items.json) and how they are grouped.
import type { ItemPickup, LotItem } from "./types";

export interface ItemCategory {
  id: string;
  name: string;
  /** Shown on the map without asking. */
  important: boolean;
  color: string;
}

export const ITEM_CATEGORIES: ItemCategory[] = [
  { id: "blessing", name: "Graines, larmes sacrées et bénédictions", important: true, color: "#f3d27a" },
  { id: "weapon", name: "Armes et boucliers", important: true, color: "#d98b5f" },
  { id: "armor", name: "Armures", important: true, color: "#b9a37c" },
  { id: "talisman", name: "Talismans", important: true, color: "#c78be0" },
  { id: "spell", name: "Sorts et incantations", important: true, color: "#6fa8ff" },
  { id: "ash", name: "Cendres d'esprit", important: true, color: "#7fd0c4" },
  { id: "aow", name: "Cendres de guerre", important: true, color: "#e0a050" },
  { id: "tear", name: "Larmes de cristal", important: true, color: "#9fe0f0" },
  { id: "key", name: "Objets clés, livres, cartes", important: true, color: "#e8e0cc" },
  { id: "upgrade", name: "Pierres de forge", important: false, color: "#a0a0a0" },
  { id: "rune", name: "Runes", important: false, color: "#d4b25a" },
  { id: "consumable", name: "Consommables", important: false, color: "#8a9a6a" },
  { id: "material", name: "Matériaux d'artisanat", important: false, color: "#6c7a52" },
];

export const categoryById = new Map(ITEM_CATEGORIES.map((c) => [c.id, c]));

const BLESSINGS = ["golden seed", "sacred tear", "scadutree fragment", "revered spirit ash"];

/** Category of one item, from its kind, goods type and English name. */
export function itemCategory(item: LotItem): string {
  const en = item.name.en.toLowerCase();
  switch (item.cat) {
    case 2:
      return "weapon";
    case 3:
      return "armor";
    case 4:
      return "talisman";
    case 5:
      return "aow";
  }
  if (BLESSINGS.some((b) => en.startsWith(b))) return "blessing";
  switch (item.goods_type) {
    case 1:
    case 12:
      return "key";
    case 2:
      return "material";
    case 5:
    case 16:
    case 17:
    case 18:
      return "spell";
    case 7:
    case 8:
      return "ash";
    case 10:
      return "tear";
    case 14:
      return "upgrade";
  }
  if (en.includes("rune") && (en.startsWith("golden rune") || en.includes("'s rune") || en.startsWith("lord's") || en.startsWith("hero's"))) {
    return "rune";
  }
  return "consumable";
}

/** The pickup's category is the most notable of its items' categories. */
export function pickupCategory(p: ItemPickup): string {
  const order = ITEM_CATEGORIES.map((c) => c.id);
  return p.items.map(itemCategory).sort((a, b) => order.indexOf(a) - order.indexOf(b))[0];
}

export const itemLabel = (item: LotItem) => item.name.fr ?? item.name.en;

/** "Graine dorée", "Pierre de forge (2) ×3", "Armure de chevalier draconique (+2)". */
export function pickupTitle(p: ItemPickup): string {
  const [first, ...rest] = p.items;
  const name = itemLabel(first) + (first.count > 1 ? ` ×${first.count}` : "");
  return rest.length ? `${name} (+${rest.length})` : name;
}

export function matches(p: ItemPickup, query: string): boolean {
  return p.items.some((i) => i.name.en.toLowerCase().includes(query) || (i.name.fr ?? "").toLowerCase().includes(query));
}
