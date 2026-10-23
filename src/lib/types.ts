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

export interface Snapshot {
  save_path: string;
  characters: Character[];
  slot: number;
  completed: string[];
  loaded_at: number;
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
