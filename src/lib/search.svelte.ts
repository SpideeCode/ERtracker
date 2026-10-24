// Search over bosses, graces, items and dungeons, in French or English.
// Shared by the header search and the map search.
import { db, tracker } from "./tracker.svelte";
import { gamedata } from "./gamedata.svelte";
import { categoryById, pickupCategory, pickupTitle } from "./items";
import { DUNGEON_ICONS } from "./route";
import { PLACE_KINDS } from "./places";
import { zoneData } from "./zoneData.svelte";
import { spoiler } from "./spoiler.svelte";

export interface SearchResult {
  id: string;
  title: string;
  subtitle: string;
  kind: string;
  color: string;
  done: boolean;
  rank: number;
}

const normalize = (s: string) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

export function search(query: string, limit = 14): SearchResult[] {
  const q = normalize(query.trim());
  if (q.length < 2) return [];
  // Prefix matches first, then matches anywhere.
  const rankOf = (...names: (string | null | undefined)[]) => {
    let best = Infinity;
    for (const n of names) {
      if (!n) continue;
      const i = normalize(n).indexOf(q);
      if (i === 0) best = Math.min(best, 0);
      else if (i > 0) best = Math.min(best, 1);
    }
    return best;
  };
  const out: SearchResult[] = [];
  for (const e of db.entries) {
    if (e.category !== "boss" && e.category !== "grace") continue;
    const fr = gamedata.name(e);
    const rank = rankOf(fr, e.name);
    if (rank === Infinity) continue;
    out.push({
      id: e.id,
      title: fr,
      subtitle: fr !== e.name ? e.name : (gamedata.region(e) ?? ""),
      kind: e.category === "boss" ? "Boss" : "Grâce",
      color: e.category === "boss" ? "#e0574a" : "#e8c46a",
      done: tracker.completed.has(e.id),
      rank,
    });
  }
  const visited = new Set(zoneData.dungeons.filter((d) => d.done).map((d) => d.id));
  gamedata.places.forEach((p, i) => {
    if (!DUNGEON_ICONS.has(p.icon)) return;
    const rank = rankOf(p.name.fr, p.name.en);
    if (rank === Infinity) return;
    const kind = PLACE_KINDS[p.icon]?.[0] ?? "donjon";
    out.push({
      id: `place:${i}`,
      title: p.name.fr ?? p.name.en,
      subtitle: p.name.fr && p.name.fr !== p.name.en ? p.name.en : "",
      kind: kind[0].toUpperCase() + kind.slice(1),
      color: "#9fb4c8",
      done: visited.has(`place:${i}`),
      rank: rank + 0.25,
    });
  });
  for (const [id, p] of gamedata.items) {
    const rank = Math.min(...p.items.map((i) => rankOf(i.name.fr, i.name.en)));
    if (rank === Infinity) continue;
    const first = p.items[0];
    const cat = categoryById.get(pickupCategory(p));
    out.push({
      id,
      title: pickupTitle(p),
      subtitle: first.name.fr && first.name.fr !== first.name.en ? first.name.en : (cat?.name ?? ""),
      kind: "Objet",
      color: cat?.color ?? "#ccc",
      done: tracker.completed.has(id),
      rank: rank + 0.5,
    });
  }
  // Without spoilers, what is still to find only shows up once its whole name is typed.
  const exact = (r: SearchResult) => normalize(r.title) === q || normalize(r.subtitle) === q;
  return out
    .filter((r) => !spoiler.hidden(r.id) || exact(r))
    .sort((a, b) => a.rank - b.rank || Number(a.done) - Number(b.done) || a.title.localeCompare(b.title)).slice(0, limit);
}
