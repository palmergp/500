// Time played: counted while you're at the table (or in a story scene, or in Learn) with the window
// in view and someone at the keyboard: after a few minutes with no click, tap or key, the clock
// stops until you're back, so a game left open doesn't count. Kept in this browser only, by mode.

export type PlayMode = "solo" | "tournament" | "story" | "friends" | "puzzles" | "learn";

export const PLAY_MODES: { mode: PlayMode; label: string }[] = [
  { mode: "solo", label: "Solo games" },
  { mode: "tournament", label: "Tournaments" },
  { mode: "story", label: "Story" },
  { mode: "friends", label: "With friends" },
  { mode: "puzzles", label: "Full-hand puzzles" },
  { mode: "learn", label: "Learn" },
];

export interface PlayTime {
  /** Seconds, by mode. */
  modes: Record<PlayMode, number>;
  /** Days with any time counted, and the last of them (your local date, YYYY-MM-DD). */
  days: number;
  lastDay: string | null;
  /** The first day counted. */
  since: string | null;
}

const KEY = "fh500.playtime";
/** No input for this long and the clock stops. */
export const IDLE_MS = 3 * 60_000;

function empty(): PlayTime {
  return { modes: { solo: 0, tournament: 0, story: 0, friends: 0, puzzles: 0, learn: 0 }, days: 0, lastDay: null, since: null };
}

export function loadPlayTime(): PlayTime {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return empty();
    const stored = JSON.parse(raw) as Partial<PlayTime>;
    return { ...empty(), ...stored, modes: { ...empty().modes, ...stored.modes } };
  } catch {
    return empty();
  }
}

/** Adds `seconds` of play in `mode`. */
export function addPlayTime(mode: PlayMode, seconds: number) {
  if (seconds <= 0) return;
  const t = loadPlayTime();
  t.modes[mode] += seconds;
  const today = new Date().toLocaleDateString("en-CA");
  if (t.lastDay !== today) {
    t.days += 1;
    t.lastDay = today;
  }
  t.since ??= today;
  try {
    localStorage.setItem(KEY, JSON.stringify(t));
  } catch {
    // Without storage, the time isn't kept.
  }
}

export function resetPlayTime() {
  try {
    localStorage.removeItem(KEY);
  } catch {
    // Nothing kept.
  }
}

/** Seconds at the table (every mode but Learn). */
export function timePlayed(t: PlayTime): number {
  return PLAY_MODES.filter((m) => m.mode !== "learn").reduce((s, m) => s + t.modes[m.mode], 0);
}

/** "3 h 25 min", "45 min", "under a minute". */
export function duration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  if (minutes < 1) return seconds > 0 ? "under a minute" : "none yet";
  const hours = Math.floor(minutes / 60);
  return hours > 0 ? `${hours} h ${minutes % 60} min` : `${minutes} min`;
}
