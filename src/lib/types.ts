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
  /** Vigor, mind, endurance, strength, dexterity, intelligence, faith, arcane. */
  stats: number[];
  /** Owned item ids, category in the top nibble (0 weapon, 1 armour, 2 talisman, 4 goods, 8 ash of war). */
  inventory: number[];
  deaths: number;
  /** Where the character last died. */
  bloodstain: Position | null;
  runes: number;
  /** Absolute offset of the held runes (u32 LE) in the save file. */
  runes_offset: number;
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

/** One entry of bosses.json, written by the extraction. */
export interface BossSheet {
  flag: number;
  npc_param: number | null;
  msb: string | null;
  runes: number | null;
  /** % absorbed: standard, slash, strike, pierce, magic, fire, lightning, holy */
  absorption: number[] | null;
  /** build-up thresholds (999 = immune): poison, rot, bleed, death, frost, sleep, madness */
  resistance: number[] | null;
  legendary: boolean;
  remembrance: { en: string; fr: string | null } | null;
}

/** One entry of markers.json, written by the extraction. */
export interface MarkerPos extends MapPos {
  flag: number;
  kind: "grace" | "boss" | "item" | "place";
  /** Map block in the game's frame, e.g. "m30_04_00" (a catacomb) or "m60_42_36". */
  block: string;
}

type Text = { en: string; fr: string | null };

export interface CatalogWeapon {
  id: number;
  name: Text;
  info: Text | null;
  rarity: number;
  weight: number;
  /** Physical, magic, fire, lightning, holy. */
  attack: number[];
  attack_max: number[];
  /** Strength, dexterity, intelligence, faith, arcane. */
  scaling: number[];
  scaling_max: number[];
  requirements: number[];
  max_level: number;
  upgrade: string;
  infusable: boolean;
}

export interface CatalogArmor {
  id: number;
  name: Text;
  rarity: number;
  weight: number;
  /** 0 head, 1 chest, 2 arms, 3 legs */
  slot: number;
  /** physical, strike, slash, pierce, magic, fire, lightning, holy */
  absorption: number[];
  resistance: number[];
}

export interface CatalogTalisman {
  id: number;
  name: Text;
  info: Text | null;
  rarity: number;
  weight: number;
  group: number;
}

export interface CatalogGoods {
  id: number;
  name: Text;
  info: Text | null;
  goods_type: number;
  rarity: number;
  spell: { fp: number; slots: number; requirements: number[] } | null;
}

export interface Catalog {
  upgrades: Record<string, { level: number; runes: number; materials: { id: number; count: number; name: Text }[] }[]>;
  weapons: CatalogWeapon[];
  armor: CatalogArmor[];
  talismans: CatalogTalisman[];
  goods: CatalogGoods[];
  ashes_of_war: { id: number; name: Text; info: Text | null }[];
}

/** One item of a pickup (items.json). */
export interface LotItem {
  /** 1 goods, 2 weapon, 3 armour, 4 talisman, 5 ash of war */
  cat: number;
  id: number;
  name: { en: string; fr: string | null };
  count: number;
  goods_type: number | null;
}

/** A place where items are picked up once (items.json). */
export interface ItemPickup extends MapPos {
  flag: number;
  lot: number;
  block: string;
  items: LotItem[];
}

/** Named point of the in-game map (places.json): dungeon entrances and landmarks. */
export interface Place extends MapPos {
  icon: number;
  block: string;
  name: { en: string; fr: string | null };
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
