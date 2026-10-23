// Walking routes computed by the backend over the extracted ground samples.
import { invoke } from "@tauri-apps/api/core";

export interface WalkPath {
  found: boolean;
  /** [x, y, height] map points. */
  points: [number, number, number][];
  length: number;
  ascent: number;
  descent: number;
}

type P = { x: number; y: number; height: number };

const cache = new Map<string, Promise<WalkPath>>();

export function walkPath(map: string, from: P, to: P): Promise<WalkPath> {
  const key = [map, from.x, from.y, from.height, to.x, to.y, to.height].map((v) => (typeof v === "number" ? Math.round(v) : v)).join(",");
  let result = cache.get(key);
  if (!result) {
    result = invoke<WalkPath>("find_path", {
      map,
      from: { x: from.x, y: from.y, z: from.height },
      to: { x: to.x, y: to.y, z: to.height },
    }).catch(() => ({ found: false, points: [], length: 0, ascent: 0, descent: 0 }));
    cache.set(key, result);
  }
  return result;
}

export function clearPathCache() {
  cache.clear();
}
