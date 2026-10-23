// "How do I get there": which grace to warp to and what to do next.
//
// Map pixels are metres at level 0 (x grows east, y grows south) and every
// position carries its height. Bosses inside dungeons (catacombs, caves, legacy
// dungeons) are reached through the dungeon's own grace or its entrance, never
// in a straight line from the nearest grace outside. In the open world, height
// differences weigh more than distance: a grace 200 m above the boss is rarely
// the shortest way.
import { db } from "./tracker.svelte";
import type { Entry, MarkerPos, Place } from "./types";

export interface GraceHop {
  grace: Entry;
  pos: MarkerPos;
  distance: number;
  direction: string;
  /** Target height minus grace height, in metres. */
  climb: number;
  discovered: boolean;
}

/** Bold text; `place` makes it clickable to show that spot on the map. */
export type StepPart = string | { strong: string; place?: { map: string; x: number; y: number } };

export interface RouteStep {
  parts: StepPart[];
  note?: string;
  warning?: boolean;
}

export interface RoutePlan {
  steps: RouteStep[];
  /** Points to draw on the map, from the warp grace to the boss. */
  path: { x: number; y: number }[];
  warp?: GraceHop;
  entrance?: Place;
  /** Name of the dungeon the boss is in, if any. */
  dungeon?: string;
  /** Undiscovered grace that most likely opens the area the boss is in. */
  gate?: GraceHop;
}

const DIRECTIONS = ["nord", "nord-est", "est", "sud-est", "sud", "sud-ouest", "ouest", "nord-ouest"];
/** Metres of horizontal walking one metre of height difference is worth. */
const CLIMB_WEIGHT = 2.5;
const BIG_CLIMB = 40;
/** Map icons of places that are dungeon entrances (catacombs, caves, tunnels, hero's graves...). */
export const DUNGEON_ICONS = new Set([4, 13, 14, 15, 16, 23, 45, 46, 210, 211, 213, 217, 218, 230, 231, 232, 233, 234]);

export function heading(dx: number, dy: number): string {
  // 0 = north, clockwise
  const angle = (Math.atan2(dx, -dy) * 180) / Math.PI;
  return DIRECTIONS[Math.round(((angle + 360) % 360) / 45) % 8];
}

export function formatDistance(m: number): string {
  return m >= 1000 ? `${(m / 1000).toFixed(1).replace(".", ",")} km` : `${Math.round(m / 10) * 10} m`;
}

export function formatClimb(m: number): string {
  if (Math.abs(m) < 15) return "au même niveau";
  return m > 0 ? `${Math.round(m)} m plus haut` : `${Math.round(-m)} m plus bas`;
}

const area = (block: string) => Number(block.slice(1, 3));
/** Open world: the overworld, the Realm of Shadow and the underground caverns. */
const isOpen = (block: string) => [60, 61, 12].includes(area(block));

function hop(grace: Entry, pos: MarkerPos, target: { x: number; y: number; height: number }, discovered: boolean): GraceHop {
  const dx = target.x - pos.x;
  const dy = target.y - pos.y;
  return { grace, pos, distance: Math.hypot(dx, dy), direction: heading(dx, dy), climb: target.height - pos.height, discovered };
}

const cost = (h: GraceHop) => h.distance + CLIMB_WEIGHT * Math.abs(h.climb);

/**
 * When the grace closest to a spot has not been discovered and is much closer
 * than any discovered one, the spot is almost certainly in an area the player
 * has not reached yet (another way in, a lift, a bridge...).
 */
function findGate(hops: GraceHop[], best: GraceHop | undefined, allowed: (h: GraceHop) => boolean): GraceHop | undefined {
  const gate = hops.filter((h) => !h.discovered && allowed(h)).sort((a, b) => cost(a) - cost(b))[0];
  if (!gate || cost(gate) > 500) return undefined;
  return !best || (cost(gate) * 1.5 < cost(best) && cost(best) - cost(gate) > 100) ? gate : undefined;
}

/** Steps through an undiscovered grace: reach it first, then go on from there. */
function gateSteps(gate: GraceHop, from: GraceHop | undefined, goal: string, nameOf: (e: Entry) => string): RouteStep[] {
  const leg = from ? hop(gate.grace, from.pos, gate.pos, true) : undefined;
  return [
    {
      warning: true,
      parts: ["Passe d'abord par ", { strong: nameOf(gate.grace), place: gate.pos }, " (pas encore découverte)"],
      note:
        "Elle est bien plus proche du boss que tes grâces : c'est souvent le vrai chemin, par une zone à débloquer ou un accès particulier" +
        (leg ? ` (elle est à ${formatDistance(leg.distance)} vers le ${leg.direction} de ta grâce, ${formatClimb(-leg.climb)}).` : "."),
    },
    {
      parts: ["Depuis cette grâce, ", goal, " est à ", { strong: formatDistance(gate.distance) }, " vers le ", { strong: gate.direction }],
      note: `${formatClimb(gate.climb).replace(/^./, (c) => c.toUpperCase())} que la grâce.`,
    },
  ];
}

/** Graces on the same map as the target, closest first (plain distance). */
export function nearestGraces(target: MarkerPos, positions: Map<string, MarkerPos>, completed: Set<string>, limit = 3): GraceHop[] {
  return allHops(target, positions, completed)
    .sort((a, b) => a.distance - b.distance)
    .slice(0, limit);
}

function allHops(target: { x: number; y: number; height: number; map: string }, positions: Map<string, MarkerPos>, completed: Set<string>) {
  return db.entries
    .filter((e) => e.category === "grace")
    .flatMap((grace) => {
      const pos = positions.get(grace.id);
      return pos && pos.map === target.map ? [hop(grace, pos, target, completed.has(grace.id))] : [];
    });
}

function findEntrance(target: MarkerPos, places: Place[], graceNamesInBlock: Set<string>): Place | undefined {
  const sameMap = places.filter((p) => p.map === target.map);
  const byBlock = sameMap.filter((p) => p.block === target.block || graceNamesInBlock.has(p.name.en));
  const near = (list: Place[]) =>
    list.sort((a, b) => Math.hypot(a.x - target.x, a.y - target.y) - Math.hypot(b.x - target.x, b.y - target.y))[0];
  if (byBlock.length) return near(byBlock);
  // Dungeon frames are anchored next to their entrance: take the closest dungeon icon.
  const candidate = near(sameMap.filter((p) => DUNGEON_ICONS.has(p.icon)));
  return candidate && Math.hypot(candidate.x - target.x, candidate.y - target.y) < 600 ? candidate : undefined;
}

export function planRoute(
  target: MarkerPos,
  positions: Map<string, MarkerPos>,
  completed: Set<string>,
  places: Place[],
  nameOf: (e: Entry) => string,
): RoutePlan {
  const hops = allHops(target, positions, completed);
  const placeName = (p: Place) => p.name.fr ?? p.name.en;

  if (!isOpen(target.block)) {
    const inBlock = hops.filter((h) => h.pos.block === target.block);
    const entrance = findEntrance(target, places, new Set(inBlock.map((h) => h.grace.name)));
    const dungeon = entrance ? placeName(entrance) : inBlock[0] ? nameOf(inBlock[0].grace) : "le donjon";

    // Already found a grace inside: warp straight in.
    const inside = inBlock.filter((h) => h.discovered).sort((a, b) => Math.hypot(a.distance, a.climb) - Math.hypot(b.distance, b.climb))[0];
    if (inside) {
      return {
        dungeon,
        warp: inside,
        path: [inside.pos, target],
        steps: [
          { parts: ["Voyage rapide vers ", { strong: nameOf(inside.grace), place: inside.pos }], note: `Grâce à l'intérieur de ${dungeon}.` },
          {
            parts: ["Avance dans le donjon sur environ ", { strong: formatDistance(inside.distance) }, " jusqu'au boss"],
            note: `Le boss est ${formatClimb(inside.climb)} que la grâce : suis le chemin du donjon.`,
          },
        ],
      };
    }

    if (entrance) {
      const toEntrance = allHops(entrance, positions, completed)
        .filter((h) => h.discovered && isOpen(h.pos.block))
        .sort((a, b) => cost(a) - cost(b))[0];
      const depth = target.height - entrance.height;
      const explore: RouteStep = {
        parts: ["Entre dans ", { strong: dungeon, place: entrance }, " et progresse jusqu'au boss"],
        note: `Le boss est ${formatClimb(depth)} que l'entrée, à ${formatDistance(Math.hypot(target.x - entrance.x, target.y - entrance.y))} à vol d'oiseau.`,
      };
      const entranceHops = allHops(entrance, positions, completed).filter((h) => isOpen(h.pos.block));
      const gate = findGate(entranceHops, toEntrance, () => true);
      if (gate) {
        return {
          dungeon,
          entrance,
          gate,
          warp: toEntrance,
          path: toEntrance ? [toEntrance.pos, gate.pos, entrance, target] : [gate.pos, entrance, target],
          steps: [
            ...(toEntrance ? [{ parts: ["Voyage rapide vers ", { strong: nameOf(toEntrance.grace), place: toEntrance.pos }] }] : []),
            ...gateSteps(gate, toEntrance, "l'entrée", nameOf),
            explore,
          ],
        };
      }
      if (!toEntrance) {
        return {
          dungeon,
          entrance,
          path: [entrance, target],
          steps: [{ parts: ["Rejoins à pied l'entrée de ", { strong: dungeon, place: entrance }], note: "Aucune grâce découverte dans les environs." }, explore],
        };
      }
      return {
        dungeon,
        entrance,
        warp: toEntrance,
        path: [toEntrance.pos, entrance, target],
        steps: [
          { parts: ["Voyage rapide vers ", { strong: nameOf(toEntrance.grace), place: toEntrance.pos }] },
          {
            parts: ["Marche ", { strong: formatDistance(toEntrance.distance) }, " vers le ", { strong: toEntrance.direction }, " jusqu'à l'entrée de ", { strong: dungeon, place: entrance }],
            note: `L'entrée est ${formatClimb(toEntrance.climb)} que la grâce.`,
          },
          explore,
        ],
      };
    }
  }

  // Open world: weigh height differences, only from graces outside dungeons.
  const open = hops.filter((h) => h.discovered && isOpen(h.pos.block)).sort((a, b) => cost(a) - cost(b));
  const best = open[0];
  const gate = findGate(hops, best, (h) => isOpen(h.pos.block) || h.pos.block === target.block);
  if (gate) {
    return {
      gate,
      warp: best,
      path: best ? [best.pos, gate.pos, target] : [gate.pos, target],
      steps: [
        ...(best ? [{ parts: ["Voyage rapide vers ", { strong: nameOf(best.grace), place: best.pos }] }] : []),
        ...gateSteps(gate, best, "le boss", nameOf),
      ],
    };
  }
  if (!best) {
    return {
      path: [],
      steps: [{ parts: ["Aucune grâce découverte dans cette zone : il faudra y aller à pied."], warning: true }],
    };
  }
  const steps: RouteStep[] = [
    { parts: ["Voyage rapide vers ", { strong: nameOf(best.grace), place: best.pos }] },
    {
      parts: ["Marche ", { strong: formatDistance(best.distance) }, " vers le ", { strong: best.direction }],
      note: `Le boss est ${formatClimb(best.climb)} que la grâce.`,
    },
  ];
  if (Math.abs(best.climb) > BIG_CLIMB) {
    steps.push({
      warning: true,
      parts: [
        best.climb > 0 ? "Gros dénivelé : il faut monter de " : "Gros dénivelé : il faut descendre de ",
        { strong: `${Math.round(Math.abs(best.climb))} m` },
      ],
      note: "La ligne droite ne passe sans doute pas : cherche une pente, une échelle, un ascenseur ou un pont sur la carte.",
    });
    const level = open.find((h) => h !== best && Math.abs(h.climb) <= 25 && h.distance < best.distance * 1.8 + 300);
    if (level) {
      steps.push({
        parts: ["Ou, plus à niveau : ", { strong: nameOf(level.grace), place: level.pos }, ` (${formatDistance(level.distance)} vers le ${level.direction})`],
      });
    }
  }
  return { warp: best, path: [best.pos, target], steps };
}
