// Full-hand puzzles (Learn → Full hands): you've won the auction with a hard bid and play it out,
// from the kitty, against The Oracle. Made by `fh-lab handpuzzles` (crates/lab/src/handpuzzles.rs):
// each can be made with every card seen, the expert makes it every time seeing only its own cards
// (so reasoning, not a lucky guess, gets there), and careless play goes down. The defenders answer
// the same plays the same way, so a puzzle doesn't change between tries.
//
// Which you've solved is kept in this browser only.

import data from "./handPuzzles.json";

export interface HandPuzzle {
  id: number;
  /** The bid you have to make: "9♥". */
  contract: string;
  /** The game saved at your first discard from the kitty (a one-hand game). */
  save: string;
}

export const HAND_PUZZLES: HandPuzzle[] = data as HandPuzzle[];

const KEY = "fh500.handPuzzles";

interface Stored {
  /** Made without a hint. */
  solved: number[];
  /** Made, but with a hint along the way. */
  helped: number[];
}

export function loadHandPuzzles(): Stored {
  try {
    const raw = localStorage.getItem(KEY);
    const stored = raw ? (JSON.parse(raw) as Partial<Stored>) : {};
    return { solved: stored.solved ?? [], helped: stored.helped ?? [] };
  } catch {
    return { solved: [], helped: [] };
  }
}

/** Notes a made puzzle: solved, or (with a hint) made with help. */
export function noteHandPuzzle(id: number, hinted: boolean) {
  const stored = loadHandPuzzles();
  if (hinted) {
    if (!stored.solved.includes(id) && !stored.helped.includes(id)) stored.helped.push(id);
  } else if (!stored.solved.includes(id)) {
    stored.solved.push(id);
    stored.helped = stored.helped.filter((x) => x !== id);
  }
  try {
    localStorage.setItem(KEY, JSON.stringify(stored));
  } catch {
    // Kept for this visit only.
  }
}
