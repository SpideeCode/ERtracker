// Access conditions for bosses behind a quest, a key item or story progress
// (data/boss_meta.json), checked against the save.
import meta from "../../data/boss_meta.json";
import { db } from "./tracker.svelte";
import type { Entry } from "./types";

interface Rule {
  text: string;
  quest?: string;
  items?: string[];
  greatRunes?: number;
  afterBosses?: string[];
}

export interface Check {
  label: string;
  done: boolean;
}

export interface Access {
  text: string;
  checks: Check[];
  quest?: { id: string; npc: string; step: number; total: number; description: string | null };
  /** True when every checkable condition is met (quests alone cannot be confirmed). */
  met: boolean;
}

const rules = meta.access as { regions: Record<string, Rule>; bosses: Record<string, Rule> };
const bossByName = new Map(db.entries.filter((e) => e.category === "boss").map((e) => [e.name, e]));

export interface AccessContext {
  completed: Set<string>;
  /** English key item name -> owned + display name. */
  keyItem: (en: string) => { owned: boolean; name: string };
  bossName: (e: Entry) => string;
}

function currentStep(questId: string, steps: number, completed: Set<string>): number {
  for (let i = steps - 1; i >= 0; i--) if (completed.has(`${questId}#${i}`)) return i;
  return -1;
}

export function accessFor(entry: Entry, ctx: AccessContext): Access[] {
  const found = [entry.region ? rules.regions[entry.region] : undefined, rules.bosses[entry.name]].filter(Boolean) as Rule[];
  return found.map((rule) => {
    const checks: Check[] = [];
    for (const en of rule.items ?? []) {
      const item = ctx.keyItem(en);
      checks.push({ label: item.name, done: item.owned });
    }
    if (rule.greatRunes) {
      const have = db.entries.filter((e) => e.category === "great_rune" && ctx.completed.has(e.id)).length;
      checks.push({ label: `${rule.greatRunes} grandes runes (tu en as ${have})`, done: have >= rule.greatRunes });
    }
    for (const name of rule.afterBosses ?? []) {
      const boss = bossByName.get(name);
      if (boss) checks.push({ label: `Vaincre ${ctx.bossName(boss)}`, done: ctx.completed.has(boss.id) });
    }
    let quest: Access["quest"];
    const q = rule.quest ? db.quests.find((x) => x.id === rule.quest) : undefined;
    if (q) {
      const step = currentStep(q.id, q.steps.length, ctx.completed);
      quest = { id: q.id, npc: q.npc, step, total: q.steps.length, description: step >= 0 ? q.steps[step].description : null };
    }
    return { text: rule.text, checks, quest, met: checks.length > 0 && checks.every((c) => c.done) };
  });
}
