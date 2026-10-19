import { useState } from "react";
import type { GradeView } from "../engine";
import { Dialog } from "./Dialog";
import { DecisionList } from "./HandReview";

interface Props {
  /** Every graded decision this game. */
  grades: GradeView[];
  /** The deal being played (1-based, counting passed-out deals). */
  deal: number;
  /** Grades still being worked out by the coach. */
  pending: number;
  onClose: () => void;
}

/** The coach's grades for your decisions so far, during the hand: this deal or the whole game. */
export function DecisionsDialog({ grades, deal, pending, onClose }: Props) {
  const thisDeal = grades.filter((g) => g.deal === deal);
  const [scope, setScope] = useState<"deal" | "game">(thisDeal.length > 0 || grades.length === 0 ? "deal" : "game");
  const deals = [...new Set(grades.map((g) => g.deal))].sort((a, b) => b - a);
  return (
    <Dialog title="Your decisions" wide onClose={onClose}>
      <div className="tabs" role="tablist">
        <button type="button" role="tab" aria-selected={scope === "deal"} className={scope === "deal" ? "active" : ""} onClick={() => setScope("deal")}>
          This hand ({thisDeal.length})
        </button>
        <button type="button" role="tab" aria-selected={scope === "game"} className={scope === "game" ? "active" : ""} onClick={() => setScope("game")}>
          Whole game ({grades.length})
        </button>
      </div>
      {pending > 0 && <p className="note">The coach is still grading your last {pending === 1 ? "decision" : `${pending} decisions`}…</p>}
      {scope === "deal" ? (
        <DecisionList grades={thisDeal} />
      ) : grades.length === 0 ? (
        <DecisionList grades={[]} />
      ) : (
        deals.map((d) => (
          <section key={d} className="decision-deal">
            <h3>{d === deal ? "This hand" : `Deal ${d}`}</h3>
            <DecisionList grades={grades.filter((g) => g.deal === d)} />
          </section>
        ))
      )}
      <div className="actions">
        <button type="button" className="primary" onClick={onClose} autoFocus>
          Back to the game
        </button>
      </div>
    </Dialog>
  );
}
