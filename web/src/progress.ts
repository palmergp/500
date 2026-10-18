// Your progress across games: graded decisions per game, counting-quiz and drill scores. Kept in
// this browser only; storage may be unavailable, so every access is guarded.

import type { GradeView, Phase, Verdict } from "./engine";

export interface GameRecord {
  id: string;
  date: string;
  finished: boolean;
  won: boolean | null;
  decisions: number;
  verdicts: Record<Verdict, number>;
  /** Expected points given up, in total and by phase. */
  loss: number;
  lossByPhase: Partial<Record<Phase, number>>;
  decisionsByPhase: Partial<Record<Phase, number>>;
}

export interface Score {
  asked: number;
  correct: number;
}

export interface Progress {
  games: GameRecord[];
  quiz: Score;
  bidding: Score;
  puzzles: Score;
}

const KEY = "fh500.progress";
const EMPTY: Progress = { games: [], quiz: { asked: 0, correct: 0 }, bidding: { asked: 0, correct: 0 }, puzzles: { asked: 0, correct: 0 } };

export function loadProgress(): Progress {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? { ...EMPTY, ...JSON.parse(raw) } : structuredClone(EMPTY);
  } catch {
    return structuredClone(EMPTY);
  }
}

function save(progress: Progress) {
  try {
    localStorage.setItem(KEY, JSON.stringify(progress));
  } catch {
    // Progress is a convenience; ignore failures.
  }
}

/** Updates the record for game `id` from its grades. */
export function recordGrades(id: string, grades: GradeView[], finished = false, won: boolean | null = null) {
  const progress = loadProgress();
  const verdicts: Record<Verdict, number> = { best: 0, good: 0, inaccuracy: 0, mistake: 0, blunder: 0 };
  const lossByPhase: Partial<Record<Phase, number>> = {};
  const decisionsByPhase: Partial<Record<Phase, number>> = {};
  for (const g of grades) {
    verdicts[g.verdict]++;
    lossByPhase[g.phase] = (lossByPhase[g.phase] ?? 0) + g.loss;
    decisionsByPhase[g.phase] = (decisionsByPhase[g.phase] ?? 0) + 1;
  }
  const existing = progress.games.find((g) => g.id === id);
  const record: GameRecord = {
    id,
    date: existing?.date ?? new Date().toISOString().slice(0, 10),
    finished: finished || (existing?.finished ?? false),
    won: won ?? existing?.won ?? null,
    decisions: grades.length,
    verdicts,
    loss: grades.reduce((s, g) => s + g.loss, 0),
    lossByPhase,
    decisionsByPhase,
  };
  progress.games = [...progress.games.filter((g) => g.id !== id), record].slice(-200);
  save(progress);
}

export function recordScore(kind: "quiz" | "bidding" | "puzzles", correct: boolean) {
  const progress = loadProgress();
  progress[kind].asked++;
  if (correct) progress[kind].correct++;
  save(progress);
}

export function resetProgress() {
  save(structuredClone(EMPTY));
}
