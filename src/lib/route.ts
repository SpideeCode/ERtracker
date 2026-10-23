// "How do I get there": nearest Sites of Grace to a position, with distance and
// heading. Map pixels are metres at level 0, x grows east and y grows south.
import { db } from "./tracker.svelte";
import type { Entry, MarkerPos } from "./types";

export interface GraceHop {
  grace: Entry;
  pos: MarkerPos;
  distance: number;
  direction: string;
  /** Target height minus grace height, in metres. */
  climb: number;
  discovered: boolean;
}

const DIRECTIONS = ["nord", "nord-est", "est", "sud-est", "sud", "sud-ouest", "ouest", "nord-ouest"];

export function heading(dx: number, dy: number): string {
  // 0 = north, clockwise
  const angle = (Math.atan2(dx, -dy) * 180) / Math.PI;
  return DIRECTIONS[Math.round(((angle + 360) % 360) / 45) % 8];
}

export function nearestGraces(
  target: MarkerPos,
  positions: Map<string, MarkerPos>,
  completed: Set<string>,
  limit = 3,
): GraceHop[] {
  return db.entries
    .filter((e) => e.category === "grace")
    .flatMap((grace) => {
      const pos = positions.get(grace.id);
      if (!pos || pos.map !== target.map) return [];
      const dx = target.x - pos.x;
      const dy = target.y - pos.y;
      return [
        {
          grace,
          pos,
          distance: Math.hypot(dx, dy),
          direction: heading(dx, dy),
          climb: target.height - pos.height,
          discovered: completed.has(grace.id),
        },
      ];
    })
    .sort((a, b) => a.distance - b.distance)
    .slice(0, limit);
}

export function formatDistance(m: number): string {
  return m >= 1000 ? `${(m / 1000).toFixed(1).replace(".", ",")} km` : `${Math.round(m / 10) * 10} m`;
}

export function formatClimb(m: number): string {
  if (Math.abs(m) < 15) return "au même niveau";
  return m > 0 ? `${Math.round(m)} m plus haut` : `${Math.round(-m)} m plus bas`;
}
