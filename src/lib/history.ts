// Analysis of the play journal (one record per save that changed something):
// sessions, attempts per boss, where deaths happen, the journey.
import type { Entry, MapPos, MarkerPos } from "./types";

export interface JournalRecord {
  t: number;
  deaths: number;
  level: number;
  runes: number;
  played: number;
  pos: MapPos | null;
  death_pos?: MapPos | null;
  gained?: string[];
  lost?: string[];
}

/** A pause longer than this starts a new session. */
const SESSION_GAP = 30 * 60 * 1000;
/** A death this close to an undefeated boss counts as an attempt on it. */
const BOSS_RADIUS = 150;
const BOSS_HEIGHT = 40;

export interface Session {
  start: number;
  end: number;
  /** In-game seconds played during the session. */
  played: number;
  deaths: number;
  levels: number;
  gained: string[];
  records: JournalRecord[];
}

export function sessions(journal: JournalRecord[]): Session[] {
  const out: Session[] = [];
  let current: JournalRecord[] = [];
  const flush = () => {
    if (!current.length) return;
    const first = current[0];
    const last = current[current.length - 1];
    out.push({
      start: first.t,
      end: last.t,
      played: Math.max(0, last.played - first.played),
      deaths: Math.max(0, last.deaths - first.deaths),
      levels: Math.max(0, last.level - first.level),
      gained: current.flatMap((r) => r.gained ?? []),
      records: current,
    });
  };
  for (const r of journal) {
    if (current.length && r.t - current[current.length - 1].t > SESSION_GAP) {
      flush();
      current = [];
    }
    current.push(r);
  }
  flush();
  // Deaths between the end of a session and the start of the next one belong to the next.
  for (let i = 1; i < out.length; i++) {
    out[i].deaths += Math.max(0, out[i].records[0].deaths - out[i - 1].records[out[i - 1].records.length - 1].deaths);
  }
  return out;
}

export interface BossStats {
  id: string;
  attempts: number;
  firstTry: number;
  lastTry: number;
  /** When the boss was defeated, if it happened while recording. */
  defeatedAt: number | null;
}

export interface DeathSpot {
  pos: MapPos;
  count: number;
  t: number;
  /** Boss the deaths were counted against, if any. */
  boss: string | null;
}

export interface Analysis {
  bosses: BossStats[];
  deaths: DeathSpot[];
  /** Deaths recorded since the journal started. */
  recordedDeaths: number;
}

/**
 * Attributes each recorded death to the closest boss that was not yet defeated
 * at that time. Bosses defeated before recording started are never candidates.
 */
export function analyse(
  journal: JournalRecord[],
  bosses: Entry[],
  positions: Map<string, MarkerPos>,
  completedNow: Set<string>,
): Analysis {
  const defeatedAt = new Map<string, number>();
  for (const r of journal) for (const id of r.gained ?? []) if (id.startsWith("boss:") && !defeatedAt.has(id)) defeatedAt.set(id, r.t);

  const stats = new Map<string, BossStats>();
  const deaths: DeathSpot[] = [];
  let recordedDeaths = 0;
  for (let i = 1; i < journal.length; i++) {
    const r = journal[i];
    const count = r.deaths - journal[i - 1].deaths;
    if (count <= 0) continue;
    recordedDeaths += count;
    const at = r.death_pos ?? r.pos;
    if (!at) continue;
    let best: { id: string; d: number } | null = null;
    for (const b of bosses) {
      const p = positions.get(b.id);
      if (!p || p.map !== at.map) continue;
      const beaten = defeatedAt.get(b.id);
      const aliveThen = beaten !== undefined ? beaten >= r.t : !completedNow.has(b.id);
      if (!aliveThen) continue;
      const d = Math.hypot(p.x - at.x, p.y - at.y);
      if (d < BOSS_RADIUS && Math.abs(p.height - at.height) < BOSS_HEIGHT && (!best || d < best.d)) best = { id: b.id, d };
    }
    deaths.push({ pos: at, count, t: r.t, boss: best?.id ?? null });
    if (best) {
      const s = stats.get(best.id) ?? { id: best.id, attempts: 0, firstTry: r.t, lastTry: r.t, defeatedAt: null };
      s.attempts += count;
      s.lastTry = r.t;
      stats.set(best.id, s);
    }
  }
  // A boss beaten on the first try still deserves a line.
  for (const [id, t] of defeatedAt) {
    const s = stats.get(id) ?? { id, attempts: 0, firstTry: t, lastTry: t, defeatedAt: null };
    s.defeatedAt = t;
    stats.set(id, s);
  }
  return {
    bosses: [...stats.values()].sort((a, b) => b.attempts - a.attempts || a.firstTry - b.firstTry),
    deaths,
    recordedDeaths,
  };
}

export function formatDuration(ms: number): string {
  const min = Math.round(ms / 60000);
  if (min < 60) return `${min} min`;
  return `${Math.floor(min / 60)} h ${String(min % 60).padStart(2, "0")}`;
}
