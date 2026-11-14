import { useMemo, useState } from "react";
import type { View } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { recordScore } from "../progress";
import { checkAchievements } from "../achievements";
import type { QuizRate } from "../settings";

const SUITS = ["S", "C", "D", "H"] as const;

interface Question {
  text: string;
  options: string[];
  answer: string;
  explain: string;
}

/** A small seeded generator, so the question for a position doesn't change on re-render. */
function seeded(seed: number) {
  let x = seed >>> 0 || 1;
  return () => {
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    return (x >>> 0) / 4294967296;
  };
}

function numberOptions(answer: number): string[] {
  const set = new Set([answer]);
  for (const d of [1, -1, 2, -2, 3]) {
    if (set.size >= 4) break;
    if (answer + d >= 0) set.add(answer + d);
  }
  return [...set].sort((a, b) => a - b).map(String);
}

const SAME_COLOR: Record<string, string> = { S: "C", C: "S", D: "H", H: "D" };
const RANKS = ["A", "K", "Q", "J", "10", "9", "8", "7", "6", "5", "4"];

/** Every card of `suit` (its effective suit), strongest first: with trumps, the Joker and both
 * bowers on top; a side suit without its jack when that jack is the left bower. Black suits start
 * at 5, red at 4. Names are "Q♠", "Joker". */
function suitOrder(suit: string, trump: string | null): string[] {
  const low = suit === "D" || suit === "H" ? "4" : "5";
  const plain = RANKS.slice(0, RANKS.indexOf(low) + 1);
  const name = (rank: string, s = suit) => `${rank}${SUIT_SYMBOL[s as keyof typeof SUIT_SYMBOL]}`;
  if (trump === suit) return ["Joker", name("J"), name("J", SAME_COLOR[suit]), ...plain.filter((r) => r !== "J").map((r) => name(r))];
  return plain.filter((r) => !(r === "J" && trump && SAME_COLOR[suit] === trump)).map((r) => name(r));
}

function cardName(c: { rank: string; suit: string | null }): string {
  return c.suit ? `${c.rank}${SUIT_SYMBOL[c.suit as keyof typeof SUIT_SYMBOL]}` : "Joker";
}

/** "Which is the highest ♠ not yet played?", asked of a suit where a higher card has gone, so it
 * takes remembering what fell. Null when no suit makes a fair question. */
function highestQuestion(view: View, trump: string | null, rand: () => number): Question | null {
  const gone = new Set(view.played.map(cardName));
  const declarer = view.contract?.declarer === 0;
  // Your own discards are out of play; the kitty's other cards you can't know, so they count as not played.
  const buried = new Set(declarer ? (view.discards ?? []).map(cardName) : []);
  const mine = new Set(view.hand.map(cardName));
  const fair = SUITS.map((suit) => {
    const order = suitOrder(suit, trump);
    const top = order.findIndex((c) => !gone.has(c) && !buried.has(c));
    return { suit, order, top };
  }).filter((x) => x.top > 0 && x.order.slice(0, x.top).some((c) => gone.has(c)));
  if (fair.length === 0) return null;
  const { suit, order, top } = fair[Math.floor(rand() * fair.length)];
  const answer = order[top];
  const start = Math.max(0, top - 2);
  const options = order.slice(start, start + 4);
  const label = trump === suit ? `trump (${SUIT_SYMBOL[suit]})` : SUIT_SYMBOL[suit];
  const above = order.slice(0, top);
  const fell = above.filter((c) => gone.has(c));
  const out = above.filter((c) => buried.has(c));
  const parts = [`${listed(fell)} ${fell.length > 1 ? "have" : "has"} been played`];
  if (out.length) parts.push(`${listed(out)} ${out.length > 1 ? "are" : "is"} among your discards`);
  return {
    text: `Which is the highest ${label} not yet played?`,
    options,
    answer,
    explain: `${answer}: ${parts.join(", and ")}.${mine.has(answer) ? " It's in your hand: a sure winner in that suit." : ""}`,
  };
}

function listed(items: string[]): string {
  return items.length < 2 ? items.join("") : `${items.slice(0, -1).join(", ")} and ${items.at(-1)}`;
}

export function makeQuestion(view: View): Question {
  const rand = seeded(view.actionCount * 2654435761);
  const trump = view.contract?.strain && view.contract.strain !== "NT" ? view.contract.strain : null;
  const kind = Math.floor(rand() * 4);
  if (kind === 3) {
    const highest = highestQuestion(view, trump, rand);
    if (highest) return highest;
  }
  if (kind === 0 && trump) {
    const n = view.trumpsUnseen ?? 0;
    return {
      text: `How many trumps (${SUIT_SYMBOL[trump]}, counting the Joker and both bowers) haven't you seen yet?`,
      options: numberOptions(n),
      answer: String(n),
      explain: `${n} trump${n === 1 ? "" : "s"} you haven't seen: not in your hand, not played${view.discards ? ", not among your discards" : ""}.`,
    };
  }
  if (kind === 1) {
    const suit = SUITS[Math.floor(rand() * 4)];
    const n = view.unseenBySuit[SUITS.indexOf(suit)];
    const label = trump === suit ? `${SUIT_SYMBOL[suit]} (trumps)` : SUIT_SYMBOL[suit];
    return {
      text: `How many ${label} cards haven't you seen yet?`,
      options: numberOptions(n),
      answer: String(n),
      explain:
        trump && suit === trump
          ? `${n}. Remember the Joker and the left bower count as trumps.`
          : trump && suit !== trump && ((suit === "D" && trump === "H") || (suit === "H" && trump === "D") || (suit === "S" && trump === "C") || (suit === "C" && trump === "S"))
            ? `${n}. This suit's jack is the left bower, a trump, so it doesn't count here.`
            : `${n}.`,
    };
  }
  const seat = 1 + Math.floor(rand() * 3);
  const suit = SUITS[Math.floor(rand() * 4)];
  const out = view.shownOut[seat][SUITS.indexOf(suit)];
  return {
    text: `Has ${view.players[seat].name} (${SEAT_NAMES[seat]}) shown out of ${SUIT_SYMBOL[suit]}?`,
    options: ["Yes", "No"],
    answer: out ? "Yes" : "No",
    explain: out
      ? `Yes: they failed to follow when ${SUIT_SYMBOL[suit]} was led, so they hold none for the rest of the hand.`
      : `No: every time ${SUIT_SYMBOL[suit]} was led they followed (or it hasn't been led yet).`,
  };
}

/** A counting question that has to be answered before you play. */
export function CountingQuiz({ view, onDone }: { view: View; onDone: () => void }) {
  const question = useMemo(() => makeQuestion(view), [view]);
  const [picked, setPicked] = useState<string | null>(null);
  const answer = (option: string) => {
    if (picked) return;
    setPicked(option);
    recordScore("quiz", option === question.answer);
    checkAchievements();
  };
  return (
    <div className="coach-panel quiz" role="dialog" aria-label="Counting quiz">
      <div className="coach-head">
        <strong>Count check</strong>
      </div>
      <p>{question.text}</p>
      <div className="quiz-options">
        {question.options.map((o) => (
          <button
            key={o}
            type="button"
            className={picked ? (o === question.answer ? "right" : o === picked ? "wrong" : "") : ""}
            onClick={() => answer(o)}
            disabled={!!picked && o !== picked && o !== question.answer}
          >
            {o}
          </button>
        ))}
      </div>
      {picked && (
        <>
          <p className={picked === question.answer ? "made" : "down"}>
            {picked === question.answer ? "Right! " : "Not quite. "}
            {question.explain}
          </p>
          <div className="actions">
            <button type="button" className="primary" onClick={onDone} autoFocus>
              Continue
            </button>
          </div>
        </>
      )}
    </div>
  );
}

/** How often a turn is quizzed, by the setting. */
export const QUIZ_RATE: Record<QuizRate, number> = { low: 0.15, medium: 0.34, high: 0.6 };

/** Whether to quiz on this turn: in play, from the third trick to the ninth (the last card is
 * forced, so nothing hangs on it), on a share of turns set by `rate`. */
export function quizDue(view: View, rate: QuizRate = "medium"): boolean {
  if (view.phase !== "playing" || !view.humanToAct || view.played.length < 8 || view.hand.length <= 1) return false;
  return seeded(view.actionCount * 40503)() < QUIZ_RATE[rate];
}
