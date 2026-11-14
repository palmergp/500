// Per-browser settings and the saved game. Storage can be unavailable (private windows,
// blocked site data), so every access is guarded and the app works without it.

export type Speed = "slow" | "normal" | "fast";

export interface Settings {
  /** Persona ids for the other seats, or "any" for a random one. */
  west: string;
  north: string;
  east: string;
  /** A short game's number of hands, or 0 to play to 500. */
  hands: number;
  speed: Speed;
  /** Grade each of your decisions as you play. */
  grade: boolean;
  /** Show how many trumps you haven't seen yet. */
  trumpCounter: boolean;
  /** Show every card played this hand. */
  tracker: boolean;
  /** Mark the suits each player has shown out of. */
  voids: boolean;
  /** Ask counting questions during play. */
  quiz: boolean;
  /** Sound effects. */
  sound: boolean;
  /** Background music. */
  music: boolean;
  /** Four-color deck: diamonds blue, clubs green. */
  fourColor: boolean;
  /** Larger cards. */
  bigCards: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  west: "rock",
  north: "oracle",
  east: "gambler",
  hands: 0,
  speed: "normal",
  grade: true,
  trumpCounter: true,
  tracker: false,
  voids: true,
  quiz: false,
  sound: true,
  music: false,
  fourColor: false,
  bigCards: false,
};

/** The helpers on in `settings` (for an unaided tournament they must all be off), named for people. */
export function assistHelpers(settings: Settings): string[] {
  const named: [boolean, string][] = [
    [settings.grade, "grading"],
    [settings.trumpCounter, "the trump counter"],
    [settings.tracker, "the card tracker"],
    [settings.voids, "out-of-suit marks"],
  ];
  return named.filter(([on]) => on).map(([, name]) => name);
}

const SETTINGS_KEY = "fh500.settings";
const SAVE_KEY = "fh500.save";
const GRADES_KEY = "fh500.grades";

export const SPEED_FACTOR: Record<Speed, number> = { slow: 1.6, normal: 1, fast: 0.45 };

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Saving is a convenience; ignore failures.
  }
}

export function loadSettings(): Settings {
  try {
    const raw = read(SETTINGS_KEY);
    return raw ? { ...DEFAULT_SETTINGS, ...JSON.parse(raw) } : DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}

export function saveSettings(settings: Settings) {
  write(SETTINGS_KEY, JSON.stringify(settings));
}

export const savedGame = {
  load: () => read(SAVE_KEY),
  store: (save: string) => write(SAVE_KEY, save),
  clear: () => write(SAVE_KEY, null),
};

/** The coach's grades for the saved game, kept alongside it. */
export const savedGrades = {
  load: <T>(): T[] => {
    try {
      return JSON.parse(read(GRADES_KEY) ?? "[]") as T[];
    } catch {
      return [];
    }
  },
  store: (grades: unknown[]) => write(GRADES_KEY, JSON.stringify(grades)),
  clear: () => write(GRADES_KEY, null),
};
