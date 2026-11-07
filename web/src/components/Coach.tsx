import type { CardView, GradeView, HintView, Verdict } from "../engine";
import { lossText, SUIT_SYMBOL } from "../engine";

const VERDICT_TEXT: Record<Verdict, string> = {
  best: "Best",
  good: "Good",
  inaccuracy: "Inaccuracy",
  mistake: "Mistake",
  blunder: "Blunder",
};

export function VerdictChip({ verdict }: { verdict: Verdict }) {
  return <span className={`verdict ${verdict}`}>{VERDICT_TEXT[verdict]}</span>;
}

function pct(x: number | null) {
  return x === null ? "" : `${Math.round(100 * x)}%`;
}

/** The coach's suggestion for the decision in front of you. */
export function HintPanel({ hint, onClose }: { hint: HintView; onClose: () => void }) {
  const showMakes = hint.options.some((o) => o.makes !== null || o.note);
  return (
    <div className="coach-panel" role="note" aria-label="Coach's hint">
      <div className="coach-head">
        <strong>
          Coach suggests: {hint.bestLabel}
          {hint.close.length > 0 && (
            <span className="close-call" title="The likely deals can't separate these plays: any of the highlighted cards is fine">
              close call
            </span>
          )}
        </strong>
        <button type="button" className="icon" aria-label="Close hint" onClick={onClose}>
          ×
        </button>
      </div>
      <ul>
        {hint.reasons.map((r) => (
          <li key={r}>{r}</li>
        ))}
      </ul>
      {showMakes && hint.options.length > 1 && (
        <p className="coach-options">
          {hint.options.map((o) => (
            <span key={o.index}>
              {o.label}: {o.makes !== null ? `makes ${pct(o.makes)}` : (o.note ?? "")}
            </span>
          ))}
        </p>
      )}
      {hint.worlds > 0 && <p className="coach-foot">Based on {hint.worlds} likely layouts of the hidden cards.</p>}
    </div>
  );
}

/** The grade for your last decision, with the reasons and a way back. */
export function GradeBar({ grade, onUndo, onClose }: { grade: GradeView; onUndo?: () => void; onClose: () => void }) {
  const fine = grade.verdict === "best" || grade.verdict === "good";
  // The three kitty discards are graded together, as one decision.
  const kitty = grade.phase === "discarding" && grade.chosenLabel.includes(" ");
  return (
    <div className={`coach-panel grade ${grade.verdict}`} role="status" aria-label="Coach's grade">
      <div className="coach-head">
        <span>
          <VerdictChip verdict={grade.verdict} /> {kitty && "Put back "}
          {grade.chosenLabel}
          {grade.verdict !== "best" && grade.bestLabel !== grade.chosenLabel && (
            <>
              {" "}
              · best: <strong>{grade.bestLabel}</strong>
              {grade.loss >= 1 && <span className="loss"> (−{lossText(grade)})</span>}
            </>
          )}
        </span>
        <span className="coach-actions">
          {onUndo && !fine && (
            <button type="button" onClick={onUndo}>
              Undo
            </button>
          )}
          <button type="button" className="icon" aria-label="Close grade" onClick={onClose}>
            ×
          </button>
        </span>
      </div>
      {grade.reasons.length > 0 && (
        <ul>
          {grade.reasons.map((r) => (
            <li key={r}>{r}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

const TRACK_ORDER = ["S", "H", "C", "D"] as const;
const SUIT_INDEX = { S: 0, C: 1, D: 2, H: 3 } as const;

/** Every card played this hand, by suit, and how many of each suit you haven't seen: a memory aid
 * you can switch off. Suits are effective suits, so a bower counts as a trump. */
export function CardTracker({ played, unseen, jokerUnseen, trump }: { played: CardView[]; unseen: number[]; jokerUnseen: boolean; trump: string | null }) {
  return (
    <div className="tracker" aria-label="Cards played">
      {TRACK_ORDER.map((suit) => (
        <span key={suit} className={suit === "H" || suit === "D" ? "red" : ""}>
          {SUIT_SYMBOL[suit]}
          {trump === suit ? " (trumps)" : ""} {played.filter((c) => (trump === suit ? c.trump : c.suit === suit && !c.trump)).map((c) => (c.suit === null ? "Jk" : c.rank)).join(" ") || "—"}
          <em> · {unseen[SUIT_INDEX[suit]]} out</em>
        </span>
      ))}
      {trump === null && <span>Joker {jokerUnseen ? "out" : "seen"}</span>}
    </div>
  );
}
