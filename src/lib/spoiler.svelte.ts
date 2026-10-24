// Spoiler-free mode: what the player has not found yet stays unnamed
// on the map and in the lists, until revealed one by one.
import { prefs } from "./prefs.svelte";
import { tracker } from "./tracker.svelte";
import { SvelteSet } from "svelte/reactivity";

const PLACEHOLDERS: Record<string, string> = {
  boss: "Boss inconnu",
  grace: "Grâce non découverte",
  item: "Objet à trouver",
};

class Spoiler {
  /** Ids revealed by hand this session. */
  revealed = new SvelteSet<string>();

  hidden(id: string): boolean {
    return prefs.spoilerFree && id.split(":")[0] in PLACEHOLDERS && !tracker.completed.has(id) && !this.revealed.has(id);
  }

  /** The name to show for an entry, boss, grace or pickup. */
  name(id: string, name: string): string {
    return this.hidden(id) ? PLACEHOLDERS[id.split(":")[0]] : name;
  }
}

export const spoiler = new Spoiler();
