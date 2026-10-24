// Gamified progression from the save: a rank that grows with everything done,
// region medals, and achievements with their progress.
import { db, tracker } from "./tracker.svelte";
import { gamedata } from "./gamedata.svelte";
import { zoneData } from "./zoneData.svelte";

export interface Achievement {
  id: string;
  title: string;
  text: string;
  done: number;
  goal: number;
  unlocked: boolean;
  /** "bronze" | "silver" | "gold" */
  tier: Tier;
}

export type Tier = "bronze" | "silver" | "gold";

export interface RegionProgress {
  region: string;
  name: string;
  done: number;
  total: number;
  pct: number;
  medal: Tier | null;
  /** Where to fly on the map. */
  map: string;
  x: number;
  y: number;
}

/** Ranks, each reached at this share of everything the game has to do. */
const RANKS: [number, string][] = [
  [0, "Sans-éclat"],
  [0.03, "Voyageur"],
  [0.08, "Chevalier errant"],
  [0.16, "Chevalier"],
  [0.27, "Champion"],
  [0.4, "Seigneur de guerre"],
  [0.55, "Demi-dieu"],
  [0.72, "Seigneur d'Elden"],
  [0.9, "Légende de l'Entre-terre"],
];

/** Points per thing done. */
export const POINTS = { boss: 10, legendary: 40, grace: 3, dungeon: 8, quest: 15, greatRune: 30, item: 1 };

const medal = (pct: number): Tier | null => (pct >= 100 ? "gold" : pct >= 80 ? "silver" : pct >= 50 ? "bronze" : null);

class Achievements {
  private ids = (category: string) => db.entries.filter((e) => e.category === category).map((e) => e.id);
  private doneOf = (ids: string[]) => ids.filter((id) => tracker.completed.has(id)).length;

  counts = $derived.by(() => {
    const bosses = this.ids("boss");
    const legendary = bosses.filter((id) => gamedata.bosses.get(id)?.legendary);
    const quests = db.quests.filter((q) => tracker.completed.has(`${q.id}#${q.steps.length - 1}`)).length;
    return {
      bosses: this.doneOf(bosses),
      bossesTotal: bosses.length,
      legendary: this.doneOf(legendary),
      legendaryTotal: legendary.length,
      graces: this.doneOf(this.ids("grace")),
      gracesTotal: this.ids("grace").length,
      greatRunes: this.doneOf(this.ids("great_rune")),
      greatRunesTotal: this.ids("great_rune").length,
      dungeons: zoneData.dungeons.filter((d) => d.done).length,
      dungeonsTotal: zoneData.dungeons.length,
      quests,
      questsTotal: db.quests.length,
      items: [...gamedata.items.keys()].filter((id) => tracker.completed.has(id)).length,
      itemsTotal: gamedata.items.size,
      deaths: tracker.character?.deaths ?? 0,
    };
  });

  private points(bosses: number, legendary: number, graces: number, dungeons: number, quests: number, greatRunes: number, items: number) {
    return (
      (bosses - legendary) * POINTS.boss +
      legendary * POINTS.legendary +
      graces * POINTS.grace +
      dungeons * POINTS.dungeon +
      quests * POINTS.quest +
      greatRunes * POINTS.greatRune +
      items * POINTS.item
    );
  }

  score = $derived.by(() => {
    const c = this.counts;
    return this.points(c.bosses, c.legendary, c.graces, c.dungeons, c.quests, c.greatRunes, c.items);
  });

  /** Score with everything done. */
  maxScore = $derived.by(() => {
    const c = this.counts;
    return this.points(c.bossesTotal, c.legendaryTotal, c.gracesTotal, c.dungeonsTotal, c.questsTotal, c.greatRunesTotal, c.itemsTotal);
  });

  rank = $derived.by(() => {
    const at = (share: number) => Math.round(share * this.maxScore);
    let i = 0;
    while (i + 1 < RANKS.length && this.score >= at(RANKS[i + 1][0])) i++;
    const from = at(RANKS[i][0]);
    const next = RANKS[i + 1] ?? null;
    return {
      level: i + 1,
      title: RANKS[i][1],
      next: next ? { title: next[1], at: at(next[0]) } : null,
      progress: next ? (this.score - from) / Math.max(1, at(next[0]) - from) : 1,
      missing: next ? at(next[0]) - this.score : 0,
    };
  });

  regions = $derived.by((): RegionProgress[] => {
    const by = new Map<string, RegionProgress>();
    for (const z of zoneData.zones) {
      const total = z.total.boss + z.total.grace + z.total.item + z.total.dungeon;
      const left = z.left.boss + z.left.grace + z.left.item + z.left.dungeon;
      const r = by.get(z.region) ?? { region: z.region, name: gamedata.regions[z.region] ?? z.region, done: 0, total: 0, pct: 0, medal: null, map: z.map, x: z.x, y: z.y };
      r.total += total;
      r.done += total - left;
      by.set(z.region, r);
    }
    return [...by.values()]
      .filter((r) => r.total >= 5)
      .map((r) => {
        const pct = Math.floor((r.done / r.total) * 100);
        return { ...r, pct, medal: medal(pct) };
      })
      .sort((a, b) => Number(b.pct > 0) - Number(a.pct > 0) || b.pct - a.pct || a.name.localeCompare(b.name));
  });

  list = $derived.by((): Achievement[] => {
    const c = this.counts;
    const out: Achievement[] = [];
    const add = (id: string, title: string, text: string, done: number, goal: number, tier: Tier) =>
      goal > 0 && out.push({ id, title, text, done: Math.min(done, goal), goal, unlocked: done >= goal, tier });

    add("boss-1", "Premier sang", "Vaincre un boss", c.bosses, 1, "bronze");
    add("boss-25", "Chasseur", "Vaincre 25 boss", c.bosses, 25, "bronze");
    add("boss-75", "Tueur de demi-dieux", "Vaincre 75 boss", c.bosses, 75, "silver");
    add("boss-150", "Fléau de l'Entre-terre", "Vaincre 150 boss", c.bosses, 150, "silver");
    add("boss-all", "Aucun ne m'a résisté", "Vaincre tous les boss", c.bosses, c.bossesTotal, "gold");
    add("legend-1", "Face à la légende", "Vaincre un boss légendaire", c.legendary, 1, "bronze");
    add("legend-all", "Panthéon", "Vaincre tous les boss légendaires", c.legendary, c.legendaryTotal, "gold");
    add("rune-2", "Porteur de runes", "Obtenir 2 Grandes runes", c.greatRunes, 2, "silver");
    add("rune-all", "Toutes les Grandes runes", "Réunir toutes les Grandes runes", c.greatRunes, c.greatRunesTotal, "gold");
    add("grace-50", "Pèlerin", "Découvrir 50 sites de grâce", c.graces, 50, "bronze");
    add("grace-200", "Cartographe", "Découvrir 200 sites de grâce", c.graces, 200, "silver");
    add("grace-all", "Chaque lueur", "Découvrir tous les sites de grâce", c.graces, c.gracesTotal, "gold");
    add("dungeon-10", "Spéléologue", "Visiter 10 donjons, grottes ou catacombes", c.dungeons, 10, "bronze");
    add("dungeon-40", "Pilleur de tombes", "Visiter 40 donjons", c.dungeons, 40, "silver");
    add("dungeon-all", "Rien ne m'échappe", "Visiter tous les donjons", c.dungeons, c.dungeonsTotal, "gold");
    add("quest-1", "À l'écoute", "Terminer une quête de PNJ", c.quests, 1, "bronze");
    add("quest-10", "Confident", "Terminer 10 quêtes de PNJ", c.quests, 10, "silver");
    add("item-500", "Collectionneur", "Ramasser 500 objets sur la carte", c.items, 500, "bronze");
    add("item-1500", "Rien ne traîne", "Ramasser 1 500 objets sur la carte", c.items, 1500, "silver");
    add("death-100", "Mourir, apprendre", "Mourir 100 fois", c.deaths, 100, "bronze");
    add("death-1000", "Sans-éclat jusqu'au bout", "Mourir 1 000 fois (et continuer)", c.deaths, 1000, "silver");
    for (const r of this.regions) {
      add(`region-${r.region}`, `Maître de ${r.name}`, `Finir ${r.name} à 100 %`, r.done, r.total, "gold");
    }
    return out;
  });

  /** Locked achievements closest to being unlocked. */
  next = $derived(
    this.list
      .filter((a) => !a.unlocked && a.done > 0)
      .sort((a, b) => b.done / b.goal - a.done / a.goal)
      .slice(0, 3),
  );

  unlockedIds = $derived(new Set(this.list.filter((a) => a.unlocked).map((a) => a.id)));
}

export const achievements = new Achievements();
