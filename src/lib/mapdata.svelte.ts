// Extraction of maps and game data from the install: status, progress, and the
// automatic refresh when the game or the app's data format changed.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { gamedata } from "./gamedata.svelte";
import { tracker } from "./tracker.svelte";
import { clearPathCache } from "./pathfind";
import type { MapProgress, MapStatus } from "./types";

class MapData {
  status = $state<MapStatus | null>(null);
  progress = $state<MapProgress | null>(null);
  running = $state(false);
  error = $state<string | null>(null);

  percent = $derived(this.progress && this.progress.total ? Math.round((this.progress.done / this.progress.total) * 100) : 0);

  private listening = false;

  /** Reads the status; re-extracts by itself when existing data is outdated. */
  async init() {
    if (!this.listening) {
      this.listening = true;
      await listen<MapProgress>("maps://progress", (e) => this.onProgress(e.payload));
    }
    await this.refresh();
    const s = this.status;
    if (s?.manifest && s.outdated && s.game_dir && !s.running) await this.start(s.game_dir);
  }

  async refresh() {
    this.status = await invoke<MapStatus>("map_status", { gameDir: null });
    this.running = this.status.running;
    if (this.status.manifest) await gamedata.load();
  }

  async start(gameDir: string) {
    this.error = null;
    this.progress = null;
    try {
      await invoke("extract_maps", { gameDir });
      this.running = true;
    } catch (e) {
      this.error = String(e);
    }
  }

  private onProgress(p: MapProgress) {
    this.progress = p;
    if (!p.finished) return;
    this.running = false;
    if (p.error) this.error = p.error;
    else {
      clearPathCache();
      this.refresh().then(() => tracker.reload());
    }
  }

  step(): string {
    const p = this.progress;
    if (p?.step === "tiles") return `Tuiles de carte ${p.done} / ${p.total}`;
    if (p?.step === "bosses") return `Recherche des boss ${p.done} / ${p.total}`;
    return "Lecture des données du jeu…";
  }
}

export const mapdata = new MapData();
