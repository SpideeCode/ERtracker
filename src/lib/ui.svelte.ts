export type ProgressTab = "bosses" | "items" | "graces" | "quests" | "collections";
export type SettingsTab = "general" | "overlay" | "runes";

export type View =
  | { kind: "now" }
  | { kind: "map" }
  | { kind: "progress"; tab: ProgressTab }
  | { kind: "explore" }
  | { kind: "journey" }
  | { kind: "builds" }
  | { kind: "settings"; tab: SettingsTab };

/** A place to centre the map on, e.g. a grace named in a route. */
export interface FlyTarget {
  map: string;
  x: number;
  y: number;
  label: string;
  /** Boss whose route should stay drawn. */
  route?: string;
  /** Highlight an area of this many metres around the point. */
  radius?: number;
}

class Ui {
  view = $state<View>({ kind: "now" });
  #sheet = $state<string | null>(null);
  /** Sheets opened before the current one, most recent last. */
  sheetHistory = $state<string[]>([]);

  /** Entry shown in the side sheet, if any. Opening one from another keeps the way back. */
  get sheet() {
    return this.#sheet;
  }
  set sheet(id: string | null) {
    if (id === this.#sheet) return;
    if (!id) this.sheetHistory = [];
    else if (this.#sheet) this.sheetHistory = [...this.sheetHistory.filter((s) => s !== id), this.#sheet].slice(-20);
    this.#sheet = id;
  }

  back() {
    const prev = this.sheetHistory.at(-1);
    if (!prev) return;
    this.sheetHistory = this.sheetHistory.slice(0, -1);
    this.#sheet = prev;
  }
  /** Entry the map should centre on and draw a route to. */
  focus = $state<string | null>(null);
  flyTo = $state<FlyTarget | null>(null);
  /** Chosen starting point per route target: a grace id, or "me" for the last saved position. */
  routeStart = $state<Record<string, string>>({});
  /** Quest to unfold when the quests tab opens. */
  questOpen = $state<string | null>(null);

  go(view: View) {
    this.view = view;
  }

  progress(tab: ProgressTab) {
    this.view = { kind: "progress", tab };
  }

  openQuest(id: string) {
    this.questOpen = id;
    this.progress("quests");
  }

  openSheet(id: string) {
    this.sheet = id;
  }

  /** Opens whatever a completed id refers to: a quest step unfolds its quest, the rest has a sheet. */
  open(id: string) {
    if (id.startsWith("quest:")) this.openQuest(id.split("#")[0]);
    else this.sheet = id;
  }

  showPlace(target: FlyTarget) {
    this.flyTo = target;
    this.view = { kind: "map" };
  }

  showOnMap(id: string) {
    this.focus = id;
    this.view = { kind: "map" };
  }
}

export const ui = new Ui();
