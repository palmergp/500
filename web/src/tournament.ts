// Tournament mode: 12 players at 3 tables, 8 rounds of a 4-hand game each. A player's score for a
// round is their side's score at the end of that game, added to both partners' totals; everyone
// has a different partner every round. You play one table; the other two are played out by the
// engine (`simulateGame`). Kept in this browser's storage, so a tournament survives a reload.

import type { GradeView } from "./engine/types";

export const ROUNDS = 8;
export const HANDS = 4;
const PLAYERS = 12;

/** A tournament player: 0 is you; the others are bots, each with a randomly assigned style. */
export interface Participant {
  id: number;
  name: string;
  /** Persona id (the playing style); "human" for you. */
  persona: string;
}

/** A finished game at one table. */
export interface TableResult {
  /** North–South's and East–West's final scores. */
  scores: [number, number];
  /** Each hand's score changes, North–South and East–West. */
  hands: [number, number][];
  /** Each of those hands' bid (absent in tournaments saved before bids were kept). */
  bids?: HandBid[];
}

/** A hand's bid: the declarer's seat at the table (0 South, 1 West, 2 North, 3 East), "8♥", made. */
export interface HandBid {
  seat: number;
  label: string;
  made: boolean;
}

/** One table in a round: participant ids by seat (South, West, North, East). */
export interface Seating {
  seats: [number, number, number, number];
  seed: number;
  result: TableResult | null;
}

export interface Round {
  tables: Seating[];
}

/** Who the other 11 are: random styles, all experts, or all gamblers. */
export type TournamentKind = "random" | "expert" | "chaos";

export const KINDS: { kind: TournamentKind; icon: string; title: string; text: string }[] = [
  { kind: "random", icon: "🎲", title: "Random", text: "Every other player gets a random playing style." },
  { kind: "expert", icon: "🎓", title: "Expert", text: "Every other player is an expert: The Counter, The Shark, The Oracle or The Natural." },
  { kind: "chaos", icon: "🌪️", title: "Chaos", text: "Every other player is The Gambler: big bids, all round." },
];

export interface Tournament {
  id: string;
  /** Absent in tournaments started before there were kinds: those were all random. */
  kind?: TournamentKind;
  /** You've had help in one of its games: a hint, an undo, Finish hand, or a helper (grading, the
   * trump counter, the card tracker, out-of-suit marks) on. Absent in tournaments started before
   * this was kept, which can't count as unaided. */
  assisted?: boolean;
  participants: Participant[];
  rounds: Round[];
  /** The round being played (0-based); ROUNDS once all are done. */
  current: number;
  /** The final results have been revealed. */
  finished: boolean;
  /** Your own game in each round, kept for your recap (absent in tournaments from before). */
  mine?: (YourGame | null)[];
}

/** Your game in one round, as your recap needs it. */
export interface YourGame {
  /** The game's save, so the coach can go over your decisions after it's over. */
  save: string;
  hands: YourHand[];
  /** The coach's grades of your decisions: those made during play (grading on), then the rest,
   * added in the background once the game is over. */
  grades: RecapGrade[];
  /** Every decision has been graded. */
  graded: boolean;
}

/** One hand of your game, from your side. */
export interface YourHand {
  /** Who declared it: 0 you, 1 West, 2 your partner, 3 East. */
  declarer: number;
  label: string;
  made: boolean;
  /** You bid in its auction (not just a pass, mention or 6-call). */
  youBid: boolean;
  /** Your side's score change. */
  points: number;
}

/** One of your decisions, graded by the coach. */
export interface RecapGrade {
  /** Its place in the game's action log. */
  action: number;
  phase: "bidding" | "discarding" | "playing";
  verdict: "best" | "good" | "inaccuracy" | "mistake" | "blunder";
  /** Points given up against the coach's choice. */
  loss: number;
  deal: number;
  trick: number | null;
  chosen: string;
  best: string;
  /** The coach's first reason, kept for the costliest. */
  reason: string | null;
  /** You asked for a hint on it, so it isn't counted. */
  hinted?: boolean;
}

/** What your recap shows: how you bid, where your points came from, and how you played. */
export interface Recap {
  games: number;
  hands: number;
  /** Hands you bid in, won the bid on, and made. */
  bid: number;
  declared: number;
  made: number;
  partnerDeclared: number;
  partnerMade: number;
  /** The opponents' bids, and how many of them your side beat. */
  defended: number;
  beaten: number;
  /** Your side's points from your bids, your partners' bids, and tricks on the opponents' bids. */
  points: { yours: number; partner: number; defence: number };
  /** Every graded decision, with its round; and the games not yet fully graded. */
  grades: (RecapGrade & { round: number })[];
  ungraded: number;
}

/** A coach grade as the recap keeps it. */
export function recapGrade(g: GradeView): RecapGrade {
  return {
    action: g.actionNumber,
    phase: g.phase === "bidding" || g.phase === "discarding" ? g.phase : "playing",
    verdict: g.verdict,
    loss: g.loss,
    deal: g.deal,
    trick: g.trick,
    chosen: g.chosenLabel,
    best: g.bestLabel,
    reason: g.reasons[0] ?? null,
    ...(g.hinted ? { hinted: true } : {}),
  };
}

/** What grading a finished game needs of the coach. */
export interface GameCoach {
  decisions(save: string): Promise<{ number: number; discards: boolean }[]>;
  grade(save: string, number: number): Promise<GradeView | null>;
  gradeDiscards(save: string, number: number): Promise<GradeView | null>;
}

/** Grades the decisions of `game` it doesn't have yet, one at a time, handing each to `keep`
 * (with how far along it is) as it comes. */
export async function gradeGame(coach: GameCoach, game: YourGame, keep: (g: RecapGrade | null, done: number, total: number) => void): Promise<void> {
  const list = await coach.decisions(game.save);
  const have = new Set(game.grades.map((g) => g.action));
  const todo = list.filter((d) => !have.has(d.number));
  for (const [i, d] of todo.entries()) {
    const g = await (d.discards ? coach.gradeDiscards(game.save, d.number) : coach.grade(game.save, d.number)).catch(() => null);
    keep(g && recapGrade(g), i + 1, todo.length);
  }
}

export function recap(t: Tournament): Recap {
  const r: Recap = {
    games: 0, hands: 0, bid: 0, declared: 0, made: 0, partnerDeclared: 0, partnerMade: 0, defended: 0, beaten: 0,
    points: { yours: 0, partner: 0, defence: 0 }, grades: [], ungraded: 0,
  };
  (t.mine ?? []).forEach((game, round) => {
    if (!game) return;
    r.games++;
    if (!game.graded) r.ungraded++;
    for (const h of game.hands) {
      r.hands++;
      if (h.youBid) r.bid++;
      if (h.declarer === 0) {
        r.declared++;
        if (h.made) r.made++;
        r.points.yours += h.points;
      } else if (h.declarer === 2) {
        r.partnerDeclared++;
        if (h.made) r.partnerMade++;
        r.points.partner += h.points;
      } else {
        r.defended++;
        if (!h.made) r.beaten++;
        r.points.defence += h.points;
      }
    }
    for (const g of game.grades) r.grades.push({ ...g, round });
  });
  return r;
}


const NAMES = ["Ada", "Ben", "Cleo", "Dev", "Elena", "Finn", "Gus", "Hana", "Ivan", "June", "Kai", "Lena", "Milo", "Nora", "Otto", "Pia", "Quinn", "Rosa", "Sol", "Tess"];
const STYLES = ["steady", "rookie", "rock", "gambler", "counter", "shark", "oracle", "natural"];
/** The strongest bots (an Expert tournament's players; Giant killer's opponents). */
export const EXPERT_STYLES = ["counter", "shark", "oracle", "natural"];
const KEY = "fh500.tournament";

/** A small seeded random number generator (mulberry32), so a tournament is reproducible. */
function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function shuffle<T>(items: T[], random: () => number): T[] {
  const out = [...items];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(random() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

/** Partner pairs for every round: the circle method gives 11 rounds in which every pair of the 12
 * players partners exactly once; 8 of them, in random order, with the players randomly placed. */
export function schedulePairs(random: () => number): [number, number][][] {
  const order = shuffle([...Array(PLAYERS).keys()], random);
  const rounds: [number, number][][] = [];
  for (let r = 0; r < PLAYERS - 1; r++) {
    const pairs: [number, number][] = [[order[PLAYERS - 1], order[r]]];
    for (let k = 1; k < PLAYERS / 2; k++) pairs.push([order[(r + k) % (PLAYERS - 1)], order[(r - k + PLAYERS - 1) % (PLAYERS - 1)]]);
    rounds.push(pairs);
  }
  return shuffle(rounds, random).slice(0, ROUNDS);
}

export function tournamentKind(t: Tournament): TournamentKind {
  return t.kind ?? "random";
}

/** A new tournament: you and 11 bots with random names, styles for the kind, and the whole schedule. */
export function newTournament(seed: number, kind: TournamentKind = "random"): Tournament {
  const random = rng(seed);
  const names = shuffle(NAMES, random);
  const styles = kind === "expert" ? EXPERT_STYLES : kind === "chaos" ? ["gambler"] : STYLES;
  const participants: Participant[] = [{ id: 0, name: "You", persona: "human" }];
  for (let i = 1; i < PLAYERS; i++) participants.push({ id: i, name: names[i - 1], persona: styles[Math.floor(random() * styles.length)] });
  const rounds: Round[] = schedulePairs(random).map((pairs) => {
    const grouped = shuffle(pairs, random);
    const tables: Seating[] = [];
    for (let t = 0; t < 3; t++) {
      let [a, b] = [grouped[2 * t], grouped[2 * t + 1]];
      // You sit South, your partner North.
      if (b.includes(0)) [a, b] = [b, a];
      if (a[1] === 0) a = [a[1], a[0]];
      tables.push({ seats: [a[0], b[0], a[1], b[1]], seed: Math.floor(random() * 2 ** 31), result: null });
    }
    return { tables };
  });
  return { id: `t${seed}`, kind, assisted: false, participants, rounds, current: 0, finished: false };
}

/** The table you're at in round `r`. */
export function yourTable(t: Tournament, r: number): number {
  return t.rounds[r].tables.findIndex((table) => table.seats.includes(0));
}

/** Each participant's points from one round (their side's final score), if their table has finished. */
export function roundPoints(t: Tournament, r: number): Map<number, number> {
  const points = new Map<number, number>();
  for (const table of t.rounds[r].tables) {
    if (!table.result) continue;
    table.seats.forEach((id, seat) => points.set(id, table.result!.scores[seat % 2]));
  }
  return points;
}

export interface Standing {
  participant: Participant;
  total: number;
  /** Points from the latest round with results. */
  last: number | null;
  /** 1-based; players with equal totals share a place. */
  place: number;
}

/** Everyone's totals over the rounds played so far, best first. */
export function standings(t: Tournament): Standing[] {
  const totals = new Map<number, number>(t.participants.map((p) => [p.id, 0]));
  let lastRound: Map<number, number> | null = null;
  t.rounds.forEach((_, r) => {
    const points = roundPoints(t, r);
    if (points.size === 0) return;
    for (const [id, p] of points) totals.set(id, (totals.get(id) ?? 0) + p);
    lastRound = points;
  });
  const list = t.participants
    .map((participant) => ({ participant, total: totals.get(participant.id) ?? 0, last: (lastRound as Map<number, number> | null)?.get(participant.id) ?? null, place: 0 }))
    .sort((a, b) => b.total - a.total || a.participant.id - b.participant.id);
  list.forEach((s, i) => (s.place = i > 0 && s.total === list[i - 1].total ? list[i - 1].place : i + 1));
  return list;
}

/** A best score and the partners who made it. */
export interface Award {
  score: number;
  players: [number, number];
  round: number;
  /** For a high hand: its bid, with the participant who declared it. */
  bid?: { label: string; by: number; made: boolean };
}

/** The High Game (a side's score at the end of a round) and the High Hand (a side's best single
 * hand), over every table so far. */
export function awards(t: Tournament): { highGame: Award | null; highHand: Award | null } {
  let highGame: Award | null = null;
  let highHand: Award | null = null;
  t.rounds.forEach((round, r) => {
    for (const table of round.tables) {
      if (!table.result) continue;
      for (const side of [0, 1]) {
        const players: [number, number] = [table.seats[side], table.seats[side + 2]];
        const game = table.result.scores[side];
        if (!highGame || game > highGame.score) highGame = { score: game, players, round: r };
        table.result.hands.forEach((hand, i) => {
          if (highHand && hand[side] <= highHand.score) return;
          const bid = table.result!.bids?.[i];
          highHand = { score: hand[side], players, round: r, bid: bid && { label: bid.label, by: table.seats[bid.seat], made: bid.made } };
        });
      }
    }
  });
  return { highGame, highHand };
}

/** Every table of the current round has a result. */
export function roundComplete(t: Tournament): boolean {
  return t.current < ROUNDS && t.rounds[t.current].tables.every((table) => table.result !== null);
}

export function loadTournament(): Tournament | null {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? (JSON.parse(raw) as Tournament) : null;
  } catch {
    return null;
  }
}

export function saveTournament(t: Tournament | null) {
  try {
    if (t) localStorage.setItem(KEY, JSON.stringify(t));
    else localStorage.removeItem(KEY);
  } catch {
    // Without storage the tournament lasts as long as the page.
  }
}

export function participantName(t: Tournament, id: number): string {
  return t.participants[id].name;
}

/** One player's game in one round: who they partnered and faced, and the final scores. */
export interface GameLine {
  partner: number;
  opponents: [number, number];
  /** Their side's final score, and the other side's. */
  score: number;
  against: number;
  /** Each hand's bid and what it scored their side (empty in tournaments saved before bids were kept). */
  hands: HandLine[];
}

/** One hand of a game, from one player's side: the bid, who declared it, and their side's points. */
export interface HandLine {
  label: string;
  by: number;
  /** Declared by their side. */
  ours: boolean;
  made: boolean;
  points: number;
}

/** Each player's game in every round, by participant id (null where their table hasn't finished). */
export function gamesByPlayer(t: Tournament): Map<number, (GameLine | null)[]> {
  const games = new Map<number, (GameLine | null)[]>(t.participants.map((p) => [p.id, t.rounds.map(() => null)]));
  t.rounds.forEach((round, r) => {
    for (const table of round.tables) {
      if (!table.result) continue;
      table.seats.forEach((id, seat) => {
        const side = seat % 2;
        games.get(id)![r] = {
          partner: table.seats[(seat + 2) % 4],
          opponents: [table.seats[(seat + 1) % 4], table.seats[(seat + 3) % 4]],
          score: table.result!.scores[side],
          against: table.result!.scores[1 - side],
          hands: (table.result!.bids ?? []).map((bid, i) => ({
            label: bid.label,
            by: table.seats[bid.seat],
            ours: bid.seat % 2 === side,
            made: bid.made,
            points: table.result!.hands[i]?.[side] ?? 0,
          })),
        };
      });
    }
  });
  return games;
}
