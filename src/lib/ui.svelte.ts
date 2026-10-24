export type View =
  | { kind: "dashboard" }
  | { kind: "map" }
  | { kind: "category"; id: string }
  | { kind: "bosses" }
  | { kind: "items" }
  | { kind: "builds" }
  | { kind: "runes" }
  | { kind: "stats" }
  | { kind: "explore" }
  | { kind: "quests" };

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
  view = $state<View>({ kind: "dashboard" });
  /** Entry shown in the side sheet (boss id), if any. */
  sheet = $state<string | null>(null);
  /** Entry the map should centre on and draw a route to. */
  focus = $state<string | null>(null);
  flyTo = $state<FlyTarget | null>(null);
  /** Chosen starting point per route target: a grace id, or "me" for the last saved position. */
  routeStart = $state<Record<string, string>>({});

  /** Quest to unfold when the quests view opens. */
  questOpen = $state<string | null>(null);

  openQuest(id: string) {
    this.questOpen = id;
    this.view = { kind: "quests" };
  }

  openSheet(id: string) {
    this.sheet = id;
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
