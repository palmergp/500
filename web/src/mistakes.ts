// Your own mistakes, kept to replay as puzzles: positions from your solo games where the coach
// graded your bid or card a mistake or a blunder (and you hadn't asked for a hint). Kept in this
// browser only, latest 40.

import type { GradeView } from "./engine";

export interface Mistake {
  /** The game's save, cut off just before your decision. */
  save: string;
  phase: "bidding" | "playing";
  chosenLabel: string;
  bestLabel: string;
  verdict: GradeView["verdict"];
  date: string;
}

const KEY = "fh500.mistakes";

export function loadMistakes(): Mistake[] {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "[]") as Mistake[];
  } catch {
    return [];
  }
}

function store(list: Mistake[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(list.slice(-40)));
  } catch {
    // Not kept; the puzzles are a convenience.
  }
}

/** Keeps `grade` as a puzzle if it's worth one. `save` is the game's save after the decision. */
export function keepMistake(grade: GradeView, save: string) {
  if (grade.hinted || (grade.verdict !== "mistake" && grade.verdict !== "blunder")) return;
  if (grade.phase !== "bidding" && grade.phase !== "playing") return;
  try {
    const data = JSON.parse(save) as { log: number[] };
    data.log = data.log.slice(0, grade.actionNumber);
    const position = JSON.stringify(data);
    const list = loadMistakes().filter((m) => m.save !== position);
    list.push({
      save: position,
      phase: grade.phase,
      chosenLabel: grade.chosenLabel,
      bestLabel: grade.bestLabel,
      verdict: grade.verdict,
      date: new Date().toISOString().slice(0, 10),
    });
    store(list);
  } catch {
    // An unreadable save: nothing to keep.
  }
}

/** Solved: off the list. */
export function solveMistake(save: string) {
  store(loadMistakes().filter((m) => m.save !== save));
}
