// Your records and bidding stats across games: high game, best hand, highest bid made, and how
// often each bid you declared was made. Kept in this browser only; storage may be unavailable,
// so every access is guarded.

import type { HandView } from "./engine";
import { standings, tournamentKind, type Tournament, type TournamentKind } from "./tournament";

export interface BidStat {
  bids: number;
  made: number;
}

export interface Records {
  /** Your side's highest final score in a finished game. */
  highGame: number | null;
  /** Most points your side scored on one hand, with its bid: who declared it (0 you, 1 left,
   * 2 partner, 3 right; absent in records kept before it was noted) and whether it made. */
  bestHand: { points: number; label: string; declarer?: number; made?: boolean } | null;
  /** Your highest bid that made (by its value). */
  highBid: { label: string; value: number } | null;
  gamesPlayed: number;
  gamesWon: number;
  /** Bids you declared, by label ("8♦"). */
  bids: Record<string, BidStat>;
  /** Finished tournaments: how many, won (first place, shared or not), on the podium (top 3), the
   * sum of your places (for the average), and your highest total with the place it earned. `wonBy`
   * splits the wins by kind; it began with the kinds, and every tournament before them was random. */
  tournaments: {
    played: number;
    won: number;
    wonBy?: Partial<Record<TournamentKind, number>>;
    podiums: number;
    placeSum: number;
    high: { score: number; place: number } | null;
    ids: string[];
  };
  /** Hands and games already counted ("game:hand", "game:end"), so a reload or undo can't count one twice. */
  counted: string[];
}

const KEY = "fh500.records";
const EMPTY: Records = {
  highGame: null,
  bestHand: null,
  highBid: null,
  gamesPlayed: 0,
  gamesWon: 0,
  bids: {},
  tournaments: { played: 0, won: 0, podiums: 0, placeSum: 0, high: null, ids: [] },
  counted: [],
};

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

/** Tournaments you've won of a kind (the random ones include every win from before the kinds). */
export function tournamentsWon(records: Records, kind: TournamentKind): number {
  const by = records.tournaments.wonBy ?? {};
  return kind === "random" ? records.tournaments.won - (by.expert ?? 0) - (by.chaos ?? 0) : (by[kind] ?? 0);
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
    const first = records.bestHand === null;
    records.bestHand = { points, label: hand.contract.label, declarer: hand.contract.declarer, made: !!hand.made };
    if (!first) broken.push(`best hand: ${points} points (${bestHandBid(records.bestHand)})`);
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

/** A best hand's bid in words: "your 10♥ bid", "partner's 8♦ bid", "defending East's 7♠ bid". */
export function bestHandBid(best: NonNullable<Records["bestHand"]>): string {
  const by = [`your ${best.label} bid`, `West's ${best.label} bid`, `partner's ${best.label} bid`, `East's ${best.label} bid`];
  if (best.declarer === undefined) return best.label;
  return best.declarer % 2 === 0 ? by[best.declarer] : `defending ${by[best.declarer]}`;
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

/** Counts a finished tournament: your place among the 12 and your total. Returns the records it broke. */
export function recordTournament(t: Tournament): string[] {
  const records = loadRecords();
  const stats = records.tournaments;
  // Tournaments keep their own list: a finished one stays in storage until the next one starts.
  if (stats.ids.includes(t.id)) return [];
  stats.ids = [...stats.ids, t.id].slice(-100);
  const you = standings(t).find((s) => s.participant.id === 0)!;
  stats.played++;
  if (you.place === 1) {
    stats.won++;
    const kind = tournamentKind(t);
    stats.wonBy = { ...stats.wonBy, [kind]: (stats.wonBy?.[kind] ?? 0) + 1 };
  }
  if (you.place <= 3) stats.podiums++;
  stats.placeSum += you.place;
  const broken: string[] = [];
  if (stats.high === null || you.total > stats.high.score) {
    if (stats.high !== null) broken.push(`highest tournament score: ${you.total} points`);
    stats.high = { score: you.total, place: you.place };
  }
  save(records);
  return broken;
}

/** "1st", "2nd", "3rd", "4th"… */
export function ordinal(n: number): string {
  const tens = n % 100;
  const suffix = tens >= 11 && tens <= 13 ? "th" : (["th", "st", "nd", "rd"][n % 10] ?? "th");
  return `${n}${suffix}`;
}
