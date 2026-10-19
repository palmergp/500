import { useEffect, useState } from "react";
import type { Coach, ExpertView, GradeView, HandView, PlayerView, TraceView } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { Card } from "./Card";
import { VerdictChip } from "./Coach";

interface Props {
  hand: HandView;
  players: PlayerView[];
  /** The coach's grades for your decisions on this deal. */
  grades: GradeView[];
  coach: Coach | null;
  /** The game's save, for the coach to rebuild positions. */
  save: string | null;
}

type Tab = "overview" | "decisions" | "tricks" | "luck";

/** A played deal, in four views: what happened, your decisions, every trick with hindsight, and
 * how an expert would have done with your cards. */
export function HandReview({ hand, players, grades, coach, save }: Props) {
  const [tab, setTab] = useState<Tab>("overview");
  const tabs: [Tab, string][] = [
    ["overview", "Overview"],
    ["decisions", `Your decisions${grades.length ? ` (${grades.length})` : ""}`],
    ["tricks", "Trick by trick"],
    ["luck", "Luck vs skill"],
  ];
  return (
    <div className="review">
      <div className="tabs" role="tablist">
        {tabs.map(([key, label]) => (
          <button key={key} type="button" role="tab" aria-selected={tab === key} className={tab === key ? "active" : ""} onClick={() => setTab(key)}>
            {label}
          </button>
        ))}
      </div>
      {tab === "overview" && <Overview hand={hand} players={players} />}
      {tab === "decisions" && <DecisionList grades={grades} />}
      {tab === "tricks" && <Tricks hand={hand} coach={coach} save={save} />}
      {tab === "luck" && <Luck hand={hand} coach={coach} save={save} />}
    </div>
  );
}

function Overview({ hand, players }: { hand: HandView; players: PlayerView[] }) {
  const c = hand.contract!;
  return (
    <>
      <p>
        {players[c.declarer].name} ({SEAT_NAMES[c.declarer]}) played <strong>{c.label}</strong> for {c.value} and{" "}
        {hand.made ? "made it" : "went down"}, taking {hand.tricksWon[c.declarer % 2]} tricks.
      </p>
      <h3>Auction</h3>
      <p className="auction-line">
        {hand.calls.map((call, i) => (
          <span key={i}>
            {SEAT_NAMES[call.seat]}: <strong>{call.label}</strong>
          </span>
        ))}
      </p>
      <h3>Hands as dealt</h3>
      <div className="review-hands">
        {[2, 1, 3, 0].map((seat) => (
          <div key={seat} className="review-hand">
            <div className="who">
              {SEAT_NAMES[seat]} · {players[seat].name}
            </div>
            <div className="mini-hand">
              {hand.dealt[seat].map((card) => (
                <Card key={card.id} card={card} small />
              ))}
            </div>
          </div>
        ))}
      </div>
      <div className="review-hands">
        <div className="review-hand">
          <div className="who">Kitty</div>
          <div className="mini-hand">
            {hand.kitty.map((card) => (
              <Card key={card.id} card={card} small />
            ))}
          </div>
        </div>
        <div className="review-hand">
          <div className="who">Discarded by {SEAT_NAMES[c.declarer]}</div>
          <div className="mini-hand">
            {hand.discards.map((card) => (
              <Card key={card.id} card={card} small />
            ))}
          </div>
        </div>
      </div>
    </>
  );
}

/** When a decision was made: the bidding, the kitty, or which trick. */
export function decisionLabel(g: GradeView): string {
  switch (g.phase) {
    case "bidding":
      return "Bidding";
    case "discarding":
      return "Kitty";
    case "playing":
      return g.trick ? `Trick ${g.trick}` : "Play";
    default:
      return "";
  }
}

/** Graded decisions, each with when it was made and (opened for slips) the coach's reasons. */
export function DecisionList({ grades }: { grades: GradeView[] }) {
  if (grades.length === 0) {
    return <p>No graded decisions yet. Turn on “Grade my decisions” in Settings to see them.</p>;
  }
  const lost = grades.reduce((s, g) => s + g.loss, 0);
  const slips = grades.filter((g) => g.verdict === "inaccuracy" || g.verdict === "mistake" || g.verdict === "blunder");
  return (
    <>
      <p>
        {grades.length} decision{grades.length > 1 ? "s" : ""} graded; {slips.length === 0 ? "no slips" : `${slips.length} slip${slips.length > 1 ? "s" : ""}`}, about{" "}
        {Math.round(lost)} expected points given up in all. Grades use only what you could see at the time.
      </p>
      <ol className="decision-list">
        {grades.map((g) => {
          const slip = slips.includes(g);
          return (
            <li key={g.actionNumber}>
              <details open={slip}>
                <summary>
                  <span className="decision-when">{decisionLabel(g)}</span> <VerdictChip verdict={g.verdict} />{" "}
                  <strong>{g.phase === "discarding" && g.chosenLabel.includes(" ") ? `Put back ${g.chosenLabel}` : g.chosenLabel}</strong>
                  {g.bestLabel !== g.chosenLabel && g.verdict !== "best" && <> · better: {g.bestLabel}</>}
                  {g.loss >= 1 && <span className="loss"> (−{Math.round(g.loss)})</span>}
                </summary>
                {g.reasons.length > 0 ? (
                  <ul>
                    {g.reasons.map((r) => (
                      <li key={r}>{r}</li>
                    ))}
                  </ul>
                ) : (
                  <p className="note">No further notes.</p>
                )}
              </details>
            </li>
          );
        })}
      </ol>
    </>
  );
}

/** Loads something from the coach once, while the tab is shown. */
function useCoach<T>(load: (() => Promise<T>) | null): { value: T | null; error: string | null } {
  const [value, setValue] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    if (!load) return;
    let live = true;
    load()
      .then((v) => live && setValue(v))
      .catch((e: unknown) => live && setError(String(e)));
    return () => {
      live = false;
    };
  }, []);
  return { value, error };
}

function Tricks({ hand, coach, save }: { hand: HandView; coach: Coach | null; save: string | null }) {
  const { value: trace, error } = useCoach<TraceView | null>(coach && save ? () => coach.trace(save, hand.number) : null);
  const c = hand.contract!;
  if (error) return <p>Could not analyze this hand: {error}</p>;
  if (!trace) return <p className="waiting-text">Analyzing every card with all hands visible…</p>;
  const tricks = hand.tricks.map((t, i) => ({ trick: t, plays: trace.plays.slice(i * 4, i * 4 + 4) }));
  return (
    <>
      <p>
        With every card visible and perfect play by everyone, {SEAT_NAMES[c.declarer]} takes <strong>{trace.par}</strong> tricks in {c.label}; they
        took {hand.tricksWon[c.declarer % 2]}. Marked cards cost their side a trick <em>in hindsight</em>, which no player can know at the table:
        use it to spot patterns, not as a grade.
      </p>
      <ol className="trick-list">
        {tricks.map(({ trick, plays }, i) => (
          <li key={i}>
            {plays.map((p) => (
              <span key={p.seat} className={`${p.seat === trick.winner ? "winner" : ""} ${p.cost > 0 ? "costly" : ""}`}>
                {SEAT_NAMES[p.seat][0]} {p.card.label}
                {p.card.suit === null && trick.namedSuit ? ` (naming ${SUIT_SYMBOL[trick.namedSuit]})` : ""}
                {p.cost > 0 && <sup> −{p.cost}</sup>}
              </span>
            ))}
            <span className="par">par {plays[3]?.parAfter ?? ""}</span>
          </li>
        ))}
      </ol>
    </>
  );
}

function Luck({ hand, coach, save }: { hand: HandView; coach: Coach | null; save: string | null }) {
  const [started, setStarted] = useState(false);
  const [result, setResult] = useState<ExpertView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const run = () => {
    if (!coach || !save) return;
    setStarted(true);
    coach.expert(save, hand.number).then(setResult, (e: unknown) => setError(String(e)));
  };
  const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);
  return (
    <>
      <p>
        Was this deal decided by the cards or by the play? The coach can replay it with <strong>The Counter</strong> in your seat (same cards,
        same opponents and partner) and compare.
      </p>
      {!started && (
        <button type="button" className="primary" onClick={run} disabled={!coach || !save}>
          Replay with The Counter in my seat
        </button>
      )}
      {started && !result && !error && <p className="waiting-text">Replaying the deal…</p>}
      {error && <p>Could not replay: {error}</p>}
      {result && (
        <div className="luck">
          <p>
            Your side scored <strong>{signed(result.actual)}</strong> on this deal (your points minus theirs).
            <br />
            With The Counter in your seat: <strong>{signed(result.expert)}</strong>
            {result.expertContract && (
              <>
                {" "}
                ({SEAT_NAMES[result.expertDeclarer!]} played {result.expertContract} and {result.expertMade ? "made it" : "went down"})
              </>
            )}
            .
          </p>
          <p>{verdictText(result.expert - result.actual)}</p>
          <p className="coach-foot">One replay of a single deal is noisy: look for patterns across many hands.</p>
        </div>
      )}
    </>
  );
}

function verdictText(gap: number): string {
  if (Math.abs(gap) <= 20) return "About the same: this result was mostly down to the cards.";
  if (gap > 0) return `The expert did about ${gap} points better with your cards: worth a look at your decisions.`;
  return `You did about ${-gap} points better than the expert replay. Nicely played.`;
}
