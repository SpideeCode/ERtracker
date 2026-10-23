import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import rawDb from "../../data/db.json";
import type { Db, Entry, SaveLocation, Snapshot, Update } from "./types";
import { gamedata } from "./gamedata.svelte";
import { pickupCategory, pickupTitle } from "./items";

export const db = rawDb as Db;
export const entriesById = new Map<string, Entry>(db.entries.map((e) => [e.id, e]));
const questsById = new Map(db.quests.map((q) => [q.id, q]));

export interface LogItem {
  at: number;
  id: string;
  label: string;
  category: string;
  gained: boolean;
}

export interface Toast {
  key: number;
  label: string;
  category: string;
}

/** Human-readable label for a completed id ("boss:10000800" or "quest:ranni#3"). */
export function describe(id: string): { label: string; category: string } {
  const entry = entriesById.get(id);
  if (entry) return { label: gamedata.name(entry), category: entry.category };
  const pickup = gamedata.items.get(id);
  if (pickup) return { label: pickupTitle(pickup), category: `item:${pickupCategory(pickup)}` };
  const [questId, step] = id.split("#");
  const quest = questsById.get(questId);
  if (quest) return { label: `${quest.npc} : étape ${Number(step) + 1}`, category: "quest" };
  return { label: id, category: "" };
}

class Tracker {
  snapshot = $state<Snapshot | null>(null);
  saves = $state<SaveLocation[]>([]);
  error = $state<string | null>(null);
  log = $state<LogItem[]>([]);
  toasts = $state<Toast[]>([]);
  completed = $derived(new Set(this.snapshot?.completed ?? []));
  character = $derived(this.snapshot?.characters.find((c) => c.slot === this.snapshot?.slot) ?? null);

  private toastKey = 0;

  async start() {
    await listen<Update>("tracker://update", (e) => this.apply(e.payload, true));
    this.saves = await invoke<SaveLocation[]>("list_saves");
    await this.run(invoke<Update | null>("load_state"));
  }

  /** Re-reads the save, e.g. once new game data (item flags) has been extracted. */
  reload() {
    return this.run(invoke<Update | null>("load_state"));
  }

  select(savePath: string | null, slot: number | null) {
    this.log = [];
    return this.run(invoke<Update | null>("select", { savePath, slot }));
  }

  private async run(request: Promise<Update | null>) {
    try {
      const update = await request;
      this.error = null;
      if (update) this.apply(update, false);
    } catch (e) {
      this.error = String(e);
    }
  }

  private apply(update: Update, live: boolean) {
    this.snapshot = update.snapshot;
    if (!live) return;
    const at = update.snapshot.loaded_at;
    const items = [
      ...update.diff.gained.map((id) => ({ at, id, gained: true, ...describe(id) })),
      ...update.diff.lost.map((id) => ({ at, id, gained: false, ...describe(id) })),
    ];
    this.log = [...items, ...this.log].slice(0, 200);
    // No toast for quest steps or everyday pickups (materials, runes, consumables).
    const quiet = ["quest", "item:material", "item:rune", "item:consumable", "item:upgrade"];
    for (const item of items.filter((i) => i.gained && !quiet.includes(i.category))) {
      this.toast(item.label, item.category);
    }
  }

  private toast(label: string, category: string) {
    const key = ++this.toastKey;
    this.toasts = [...this.toasts, { key, label, category }].slice(-4);
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.key !== key)), 6000);
  }
}

export const tracker = new Tracker();
