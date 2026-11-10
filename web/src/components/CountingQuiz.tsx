import { useMemo, useState } from "react";
import type { View } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { recordScore } from "../progress";
import { checkAchievements } from "../achievements";

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

export function makeQuestion(view: View): Question {
  const rand = seeded(view.actionCount * 2654435761);
  const trump = view.contract?.strain && view.contract.strain !== "NT" ? view.contract.strain : null;
  const kind = Math.floor(rand() * 3);
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

/** Whether to quiz on this turn: in play, from the third trick, on about one turn in three. */
export function quizDue(view: View): boolean {
  if (view.phase !== "playing" || !view.humanToAct || view.played.length < 8) return false;
  return seeded(view.actionCount * 40503)() < 0.34;
}
