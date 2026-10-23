// Access conditions evaluated against the current save and catalogue.
import { accessFor, type Access } from "./access";
import { catalog } from "./catalog.svelte";
import { gamedata } from "./gamedata.svelte";
import { tracker } from "./tracker.svelte";
import type { Entry } from "./types";

export function accessRules(entry: Entry): Access[] {
  const goods = [...catalog.goods.values()];
  return accessFor(entry, {
    completed: tracker.completed,
    keyItem: (en) => {
      const g = goods.find((x) => x.name.en === en);
      return { owned: g ? catalog.isOwned(1, g.id) : false, name: g?.name.fr ?? en };
    },
    bossName: (e) => gamedata.name(e),
  });
}

/** True when the boss has access conditions that are not all met yet. */
export function locked(entry: Entry): boolean {
  return accessRules(entry).some((r) => !r.met);
}
