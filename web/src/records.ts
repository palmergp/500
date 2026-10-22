// Your records and bidding stats across games: high game, best hand, highest bid made, and how
// often each bid you declared was made. Kept in this browser only; storage may be unavailable,
// so every access is guarded.

import type { HandView } from "./engine";

export interface BidStat {
  bids: number;
  made: number;
}

export interface Records {
  /** Your side's highest final score in a finished game. */
  highGame: number | null;
  /** Most points your side scored on one hand. */
  bestHand: { points: number; label: string } | null;
  /** Your highest bid that made (by its value). */
  highBid: { label: string; value: number } | null;
  gamesPlayed: number;
  gamesWon: number;
  /** Bids you declared, by label ("8♦"). */
  bids: Record<string, BidStat>;
  /** Hands and games already counted ("game:hand", "game:end"), so a reload or undo can't count one twice. */
  counted: string[];
}

const KEY = "fh500.records";
const EMPTY: Records = { highGame: null, bestHand: null, highBid: null, gamesPlayed: 0, gamesWon: 0, bids: {}, counted: [] };

export function loadRecords(): Records {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? { ...structuredClone(EMPTY), ...JSON.parse(raw) } : structuredClone(EMPTY);
  } catch {
    return structuredClone(EMPTY);
  }
}

function save(records: Records) {
  try {
    localStorage.setItem(KEY, JSON.stringify(records));
  } catch {
    // Records are a convenience; ignore failures.
  }
}

export function resetRecords() {
  save(structuredClone(EMPTY));
}

/** Counts a hand just scored (seat 0 is you, team 0 your side). Returns the records it broke. */
export function recordHand(game: string, hand: HandView): string[] {
  const records = loadRecords();
  const key = `${game}:${hand.number}`;
  if (!hand.contract || records.counted.includes(key)) return [];
  records.counted = [...records.counted, key].slice(-500);
  const broken: string[] = [];
  const points = hand.scoreChange[0];
  if (points > 0 && (records.bestHand === null || points > records.bestHand.points)) {
    if (records.bestHand !== null) broken.push(`best hand: ${points} points`);
    records.bestHand = { points, label: hand.contract.label };
  }
  if (hand.contract.declarer === 0) {
    const stat = (records.bids[hand.contract.label] ??= { bids: 0, made: 0 });
    stat.bids++;
    if (hand.made) {
      stat.made++;
      if (records.highBid === null || hand.contract.value > records.highBid.value) {
        if (records.highBid !== null) broken.push(`highest bid made: ${hand.contract.label}`);
        records.highBid = { label: hand.contract.label, value: hand.contract.value };
      }
    }
  }
  save(records);
  return broken;
}

/** Counts a finished game: your side's final score and whether you won. Returns the records it broke. */
export function recordGame(game: string, score: number, won: boolean): string[] {
  const records = loadRecords();
  const key = `${game}:end`;
  if (records.counted.includes(key)) return [];
  records.counted = [...records.counted, key].slice(-500);
  records.gamesPlayed++;
  if (won) records.gamesWon++;
  const broken: string[] = [];
  if (records.highGame === null || score > records.highGame) {
    if (records.highGame !== null) broken.push(`high game: ${score} points`);
    records.highGame = score;
  }
  save(records);
  return broken;
}
