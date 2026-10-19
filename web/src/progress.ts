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
  /** Decisions made after a hint: graded, but left out of everything above and below. */
  hinted?: number;
  /** Slips (inaccuracies or worse) by kind, for finding recurring leaks. */
  patterns?: Record<string, { slips: number; loss: number }>;
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

/** Updates the record for game `id` from its grades. Decisions made after a hint don't count: the
 * hint shaped the choice, so it says nothing about how you play. */
export function recordGrades(id: string, all: GradeView[], finished = false, won: boolean | null = null) {
  const progress = loadProgress();
  const grades = all.filter((g) => !g.hinted);
  const patterns: Record<string, { slips: number; loss: number }> = {};
  for (const g of grades) {
    if (!g.pattern || g.verdict === "best" || g.verdict === "good") continue;
    const p = (patterns[g.pattern] ??= { slips: 0, loss: 0 });
    p.slips++;
    p.loss += g.loss;
  }
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
    hinted: all.length - grades.length,
    patterns,
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

/** Your costliest recurring slips over the last `games` games, worst first. */
export function leaks(progress: Progress, games = 20): { pattern: string; slips: number; loss: number }[] {
  const totals: Record<string, { slips: number; loss: number }> = {};
  for (const game of progress.games.slice(-games)) {
    for (const [pattern, p] of Object.entries(game.patterns ?? {})) {
      const t = (totals[pattern] ??= { slips: 0, loss: 0 });
      t.slips += p.slips;
      t.loss += p.loss;
    }
  }
  return Object.entries(totals)
    .map(([pattern, t]) => ({ pattern, ...t }))
    .filter((l) => l.slips >= 2)
    .sort((a, b) => b.loss - a.loss);
}

export function resetProgress() {
  save(structuredClone(EMPTY));
}
