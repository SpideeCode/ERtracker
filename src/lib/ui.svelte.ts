export type View =
  | { kind: "dashboard" }
  | { kind: "map" }
  | { kind: "category"; id: string }
  | { kind: "bosses" }
  | { kind: "quests" };

class Ui {
  view = $state<View>({ kind: "dashboard" });
  /** Entry shown in the side sheet (boss id), if any. */
  sheet = $state<string | null>(null);
  /** Entry the map should centre on and draw a route to. */
  focus = $state<string | null>(null);

  openSheet(id: string) {
    this.sheet = id;
  }

  showOnMap(id: string) {
    this.focus = id;
    this.view = { kind: "map" };
  }
}

export const ui = new Ui();
