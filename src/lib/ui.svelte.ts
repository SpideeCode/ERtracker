export type View =
  | { kind: "dashboard" }
  | { kind: "map" }
  | { kind: "category"; id: string }
  | { kind: "bosses" }
  | { kind: "items" }
  | { kind: "quests" };

/** A place to centre the map on, e.g. a grace named in a route. */
export interface FlyTarget {
  map: string;
  x: number;
  y: number;
  label: string;
  /** Boss whose route should stay drawn. */
  route?: string;
}

class Ui {
  view = $state<View>({ kind: "dashboard" });
  /** Entry shown in the side sheet (boss id), if any. */
  sheet = $state<string | null>(null);
  /** Entry the map should centre on and draw a route to. */
  focus = $state<string | null>(null);
  flyTo = $state<FlyTarget | null>(null);

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
