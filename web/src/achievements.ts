// Achievements: challenges to try, with progress toward each. Most are measured from what's already
// kept (records, bid stats, drill scores, lessons), so play from before they existed counts; the
// rest come from moments noted as they happen (a slam, a comeback, a clean game). Once done, an
// achievement stays done. Kept in this browser only; storage may be unavailable, so every access is
// guarded. A newly done one is announced with a "fh500:achievements" window event (its titles).

import type { GradeView, HandView, View } from "./engine";
import { LESSONS } from "./learn/lessons";
import { solvedMistakes } from "./mistakes";
import { loadLessonsDone, loadProgress } from "./progress";
import { loadRecords, tournamentsWon } from "./records";
import { awards, EXPERT_STYLES, standings, type Tournament } from "./tournament";

export type Group = "Bidding" | "Play and defence" | "Games" | "Tournaments" | "Learning";

export interface Achievement {
  id: string;
  group: Group;
  icon: string;
  title: string;
  /** What to do. */
  text: string;
  /** How many it takes (1 for a one-off). */
  target: number;
}

export const ACHIEVEMENTS: Achievement[] = [
  { id: "first-bid", group: "Bidding", icon: "✋", title: "Off the mark", text: "Win an auction and make your bid.", target: 1 },
  { id: "every-strain", group: "Bidding", icon: "🌈", title: "Full set", text: "Make a bid of your own in every suit and in No Trumps.", target: 5 },
  { id: "no-trumps", group: "Bidding", icon: "🚫", title: "No trumps needed", text: "Make a No Trumps bid of your own.", target: 1 },
  { id: "ten-tricks", group: "Bidding", icon: "🔟", title: "All in", text: "Make a 10 bid of your own.", target: 1 },
  { id: "ten-nt", group: "Bidding", icon: "👑", title: "The big one", text: "Make 10 No Trumps yourself: 520 points.", target: 1 },
  { id: "workhorse", group: "Bidding", icon: "🐎", title: "Workhorse", text: "Make 50 bids of your own.", target: 50 },
  { id: "slam", group: "Play and defence", icon: "💥", title: "Slam", text: "Take all 10 tricks on your side's bid.", target: 1 },
  { id: "brick-wall", group: "Play and defence", icon: "🧱", title: "Brick wall", text: "Set the other side's bid of 8 or more.", target: 1 },
  { id: "defender", group: "Play and defence", icon: "🛡️", title: "Defender", text: "Set the other side's bid 25 times.", target: 25 },
  { id: "first-win", group: "Games", icon: "🥇", title: "First win", text: "Win a game.", target: 1 },
  { id: "seasoned", group: "Games", icon: "🎖️", title: "Seasoned", text: "Win 25 games.", target: 25 },
  { id: "hot-streak", group: "Games", icon: "🔥", title: "Hot streak", text: "Win 3 games in a row.", target: 3 },
  { id: "comeback", group: "Games", icon: "↩️", title: "Comeback", text: "Win a game after trailing by 200 points or more.", target: 1 },
  { id: "shutout", group: "Games", icon: "🧊", title: "Shutout", text: "Win a game with the other side below zero.", target: 1 },
  {
    id: "giant-killer",
    group: "Games",
    icon: "🗡️",
    title: "Giant killer",
    text: "Win a game against two expert opponents: The Counter, The Shark, The Oracle or The Natural.",
    target: 1,
  },
  {
    id: "clean-sheet",
    group: "Games",
    icon: "✨",
    title: "Clean sheet",
    text: "Finish a game with at least 15 graded decisions, no mistakes or blunders, and no hints.",
    target: 1,
  },
  { id: "tour-finish", group: "Tournaments", icon: "🎟️", title: "Tournament player", text: "Play all 8 rounds of a tournament.", target: 1 },
  { id: "podium", group: "Tournaments", icon: "🥉", title: "On the podium", text: "Finish a tournament in the top 3.", target: 1 },
  { id: "champion", group: "Tournaments", icon: "🏆", title: "Champion", text: "Win a tournament.", target: 1 },
  { id: "headliner", group: "Tournaments", icon: "📣", title: "Headliner", text: "Take a tournament's High Game or High Hand with your partner.", target: 1 },
  { id: "dynasty", group: "Tournaments", icon: "🏰", title: "Dynasty", text: "Win 3 tournaments.", target: 3 },
  { id: "random-champion", group: "Tournaments", icon: "🎲", title: "Luck of the draw", text: "Win a Random tournament.", target: 1 },
  {
    id: "expert-champion",
    group: "Tournaments",
    icon: "🎓",
    title: "Masterclass",
    text: "Win an Expert tournament, where every other player is The Counter, The Shark, The Oracle or The Natural.",
    target: 1,
  },
  { id: "chaos-champion", group: "Tournaments", icon: "🌪️", title: "Eye of the storm", text: "Win a Chaos tournament, where every other player is The Gambler.", target: 1 },
  {
    id: "unaided",
    group: "Tournaments",
    icon: "🙌",
    title: "Unaided",
    text: "Win a tournament with no hints, no undos, no Finish hand, and grading, the trump counter, the card tracker and out-of-suit marks all off.",
    target: 1,
  },
  { id: "scholar", group: "Learning", icon: "📚", title: "Scholar", text: "Finish every lesson.", target: LESSONS.length },
  { id: "card-counter", group: "Learning", icon: "🧮", title: "Card counter", text: "Answer 25 counting-quiz questions right.", target: 25 },
  { id: "auctioneer", group: "Learning", icon: "🔨", title: "Auctioneer", text: "Get 50 right in the bidding trainer.", target: 50 },
  { id: "puzzler", group: "Learning", icon: "🧩", title: "Puzzler", text: "Solve 25 play puzzles.", target: 25 },
  { id: "second-chance", group: "Learning", icon: "🔁", title: "Second chance", text: "Solve 10 of your own mistakes in Your mistakes.", target: 10 },
];

export const GROUPS: Group[] = ["Bidding", "Play and defence", "Games", "Tournaments", "Learning"];

/** The strongest bots, for Giant killer. */
const EXPERTS = new Set(EXPERT_STYLES);

/** Moments noted as they happen (what the records don't keep). */
interface Moments {
  slams: number;
  /** The other side's bids your side set, and how many of those were 8 or more. */
  sets: number;
  bigSets: number;
  /** Games won in a row now, and the most ever. */
  streak: number;
  bestStreak: number;
  comebacks: number;
  shutouts: number;
  giantKills: number;
  cleanGames: number;
  tourAwards: number;
  /** Tournaments won with no help at all (`Tournament.assisted` false). */
  unaidedWins: number;
}

interface Stored {
  moments: Moments;
  /** Done achievements, with the date each was done. */
  done: Record<string, string>;
  /** Hands, games and tournaments already noted, so a reload or undo can't note one twice. */
  counted: string[];
}

const KEY = "fh500.achievements";
const EMPTY: Stored = {
  moments: { slams: 0, sets: 0, bigSets: 0, streak: 0, bestStreak: 0, comebacks: 0, shutouts: 0, giantKills: 0, cleanGames: 0, tourAwards: 0, unaidedWins: 0 },
  done: {},
  counted: [],
};

function load(): Stored {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return structuredClone(EMPTY);
    const stored = JSON.parse(raw) as Partial<Stored>;
    return { ...structuredClone(EMPTY), ...stored, moments: { ...EMPTY.moments, ...stored.moments } };
  } catch {
    return structuredClone(EMPTY);
  }
}

function save(stored: Stored) {
  try {
    localStorage.setItem(KEY, JSON.stringify(stored));
  } catch {
    // Achievements are a convenience; ignore failures.
  }
}

export function resetAchievements() {
  save(structuredClone(EMPTY));
}

/** Notes once per key; returns false if already noted. */
function once(stored: Stored, key: string): boolean {
  if (stored.counted.includes(key)) return false;
  stored.counted = [...stored.counted, key].slice(-500);
  return true;
}

/** A hand just scored (seat 0 is you, team 0 your side). */
export function noteHand(game: string, hand: HandView) {
  const stored = load();
  if (!hand.contract || !once(stored, `${game}:${hand.number}`)) return;
  const m = stored.moments;
  const ours = hand.contract.declarer % 2 === 0;
  if (ours && hand.tricksWon[0] === 10) m.slams++;
  if (!ours && !hand.made) {
    m.sets++;
    if (hand.contract.level >= 8) m.bigSets++;
  }
  save(stored);
}

/** A game just finished. */
export function noteGame(game: string, view: View) {
  const stored = load();
  if (!once(stored, `${game}:end`)) return;
  const m = stored.moments;
  const won = view.winner === 0;
  m.streak = won ? m.streak + 1 : 0;
  m.bestStreak = Math.max(m.bestStreak, m.streak);
  if (won) {
    const behind = Math.max(0, ...view.history.filter((h) => !h.passedOut).map((h) => h.scoresAfter[1] - h.scoresAfter[0]));
    if (behind >= 200) m.comebacks++;
    if (view.scores[1] < 0) m.shutouts++;
    if (EXPERTS.has(view.players[1].persona) && EXPERTS.has(view.players[3].persona)) m.giantKills++;
  }
  save(stored);
}

/** A finished game's grades, once the coach has graded every decision in it. */
export function noteGrades(game: string, grades: GradeView[]) {
  const slips = grades.some((g) => g.verdict === "mistake" || g.verdict === "blunder");
  if (grades.length < 15 || slips || grades.some((g) => g.hinted)) return;
  const stored = load();
  if (!once(stored, `${game}:graded`)) return;
  stored.moments.cleanGames++;
  save(stored);
}

/** A tournament just finished. */
export function noteTournament(t: Tournament) {
  const stored = load();
  if (!once(stored, `tour:${t.id}`)) return;
  const { highGame, highHand } = awards(t);
  if (highGame?.players.includes(0) || highHand?.players.includes(0)) stored.moments.tourAwards++;
  const won = standings(t).find((s) => s.participant.id === 0)?.place === 1;
  if (won && t.assisted === false) stored.moments.unaidedWins++;
  save(stored);
}

export interface Status {
  achievement: Achievement;
  /** Progress so far (at most the target). */
  value: number;
  /** The date it was done (YYYY-MM-DD), or null. */
  done: string | null;
}

/** How far along each achievement is, measured from everything kept. */
function measure(m: Moments): Record<string, number> {
  const records = loadRecords();
  const progress = loadProgress();
  const made = Object.entries(records.bids).filter(([, b]) => b.made > 0);
  const madeTotal = made.reduce((s, [, b]) => s + b.made, 0);
  // Labels are "8♦", "10NT": the strain is what follows the number.
  const strains = new Set(made.map(([label]) => label.replace(/^\d+/, "")));
  const t = records.tournaments;
  const lessons = new Set(loadLessonsDone());
  return {
    "first-bid": madeTotal,
    "every-strain": strains.size,
    "no-trumps": strains.has("NT") ? 1 : 0,
    "ten-tricks": made.some(([label]) => label.startsWith("10")) ? 1 : 0,
    "ten-nt": (records.bids["10NT"]?.made ?? 0) > 0 ? 1 : 0,
    workhorse: madeTotal,
    slam: m.slams,
    "brick-wall": m.bigSets,
    defender: m.sets,
    "first-win": records.gamesWon,
    seasoned: records.gamesWon,
    "hot-streak": m.bestStreak,
    comeback: m.comebacks,
    shutout: m.shutouts,
    "giant-killer": m.giantKills,
    "clean-sheet": m.cleanGames,
    "tour-finish": t.played,
    podium: t.podiums,
    champion: t.won,
    headliner: m.tourAwards,
    dynasty: t.won,
    "random-champion": tournamentsWon(records, "random"),
    "expert-champion": tournamentsWon(records, "expert"),
    "chaos-champion": tournamentsWon(records, "chaos"),
    unaided: m.unaidedWins,
    scholar: LESSONS.filter((l) => lessons.has(l.id)).length,
    "card-counter": progress.quiz.correct,
    auctioneer: progress.bidding.correct,
    puzzler: progress.puzzles.correct,
    "second-chance": solvedMistakes(),
  };
}

/** Every achievement's progress, marking newly reached ones done. Unless `quiet`, newly done ones
 * are announced (the "fh500:achievements" event, with their titles). */
export function achievements(quiet = false): Status[] {
  const stored = load();
  const values = measure(stored.moments);
  // Your local date (en-CA writes it YYYY-MM-DD).
  const today = new Date().toLocaleDateString("en-CA");
  const fresh: string[] = [];
  const list = ACHIEVEMENTS.map((achievement) => {
    const value = Math.min(values[achievement.id] ?? 0, achievement.target);
    if (value >= achievement.target && !stored.done[achievement.id]) {
      stored.done[achievement.id] = today;
      fresh.push(achievement.title);
    }
    return { achievement, value, done: stored.done[achievement.id] ?? null };
  });
  if (fresh.length) {
    save(stored);
    if (!quiet) window.dispatchEvent(new CustomEvent("fh500:achievements", { detail: fresh }));
  }
  return list;
}

/** A done date for people: "Nov 9, 2026". */
export function doneDate(date: string): string {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
}

/** Checks for newly done achievements and announces them. */
export function checkAchievements() {
  achievements();
}
