export type Localized = { fr: string; en: string };

export interface Category {
  id: string;
  name: Localized;
}

export interface Entry {
  id: string;
  category: string;
  name: string;
  region: string | null;
  flag: number;
}

export interface QuestStep {
  description: string;
  location: string | null;
  flags: [number, number][];
}

export interface Quest {
  id: string;
  npc: string;
  steps: QuestStep[];
}

export interface Db {
  version: number;
  categories: Category[];
  entries: Entry[];
  quests: Quest[];
}

export interface Position {
  x: number;
  y: number;
  z: number;
  map: string;
}

export interface Character {
  slot: number;
  name: string;
  level: number;
  seconds_played: number;
  version: number;
  checksum_ok: boolean;
  position: Position | null;
}

export type MapId = "M00" | "M01" | "M10" | "M11";

/** Pixel position on a level-0 map image. */
export interface MapPos {
  map: MapId;
  x: number;
  y: number;
  height: number;
}

export interface Snapshot {
  save_path: string;
  characters: Character[];
  slot: number;
  completed: string[];
  map_position: MapPos | null;
  loaded_at: number;
}

export interface MapInfo {
  id: MapId;
  width: number;
  height: number;
  max_zoom: number;
  min_zoom: number;
}

export interface MapStatus {
  manifest: { format: number; game_build: string; maps: MapInfo[] } | null;
  game_dir: string | null;
  outdated: boolean;
  running: boolean;
}

export interface MapProgress {
  step: string;
  done: number;
  total: number;
  error: string | null;
  finished: boolean;
}

/** One entry of markers.json, written by the extraction. */
export interface MarkerPos extends MapPos {
  flag: number;
  kind: "grace" | "boss";
}

export interface Update {
  snapshot: Snapshot;
  diff: { gained: string[]; lost: string[] };
}

export interface SaveLocation {
  path: string;
  steam_id: string;
  seamless_coop: boolean;
  modified: number;
}
