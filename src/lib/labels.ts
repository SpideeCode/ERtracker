import { db } from "./tracker.svelte";
import { categoryById } from "./items";

const categoryNames = new Map(db.categories.map((c) => [c.id, c.name.fr]));

export function categoryName(id: string): string {
  if (id === "quest") return "Quête";
  if (id.startsWith("item:")) return categoryById.get(id.slice(5))?.name ?? "Objet";
  return categoryNames.get(id) ?? id;
}

/** Verb shown in notifications, e.g. "Boss vaincu". */
export function achievement(category: string): string {
  switch (category) {
    case "boss":
      return "Boss vaincu";
    case "achievement":
      return "Haut fait débloqué";
    case "grace":
      return "Site de grâce découvert";
    case "great_rune":
      return "Grande rune obtenue";
    case "map_reveal":
      return "Carte révélée";
    case "item:blessing":
    case "item:weapon":
    case "item:armor":
    case "item:talisman":
    case "item:spell":
    case "item:ash":
    case "item:aow":
    case "item:tear":
    case "item:key":
      return "Objet récupéré";
    default:
      return "Obtenu";
  }
}

export function playtime(seconds: number): string {
  return `${Math.floor(seconds / 3600)} h ${String(Math.floor(seconds / 60) % 60).padStart(2, "0")}`;
}

export function ago(ms: number, now: number): string {
  const s = Math.max(0, Math.round((now - ms) / 1000));
  if (s < 60) return `il y a ${s} s`;
  if (s < 3600) return `il y a ${Math.floor(s / 60)} min`;
  return `il y a ${Math.floor(s / 3600)} h`;
}
