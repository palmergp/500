import { useEffect, useState } from "react";
import type { Coach, ExpertView, GradeView, HandView, PlayerView, TraceView } from "../engine";
import { lossText, SEAT_NAMES, SUIT_SYMBOL } from "../engine";
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
  /** Play the hand again from action `at` (solo games): it doesn't count toward the game. */
  onReplay?: (at: number, label: string) => void;
  /** A hand code for the game as it stood after action `at`. */
  onCode?: (at: number) => void;
  /** Where this hand ends in the game's log, for its code. */
  end?: number;
}

/** The first action of trick `k` (1-based) of `hand`: after the calls and the three discards. */
export function trickStart(hand: HandView, k: number): number {
  return hand.firstAction + hand.calls.length + 3 + 4 * (k - 1);
}

/** "Replay from here" and "Code" for a moment of a hand. */
export function MomentActions({ at, label, onReplay, onCode }: { at: number; label: string; onReplay?: (at: number, label: string) => void; onCode?: (at: number) => void }) {
  if (!onReplay && !onCode) return null;
  return (
    <span className="moment-actions">
      {onReplay && (
        <button type="button" onClick={() => onReplay(at, label)} title="Play the hand again from this point: try something else. It doesn't count toward the game.">
          Replay from here
        </button>
      )}
      {onCode && (
        <button type="button" onClick={() => onCode(at)} title="A code for this exact moment, to ask about it later">
          Code
        </button>
      )}
    </span>
  );
}

type Tab = "overview" | "decisions" | "tricks" | "luck";

/** A played deal, in four views: what happened, your decisions, every trick with hindsight, and
 * how an expert would have done with your cards. */
export function HandReview({ hand, players, grades, coach, save, onReplay, onCode, end }: Props) {
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
      {tab === "overview" && (
        <>
          <Overview hand={hand} players={players} />
          {(onReplay || onCode) && (
            <div className="actions">
              {onReplay && (
                <button type="button" onClick={() => onReplay(hand.firstAction, "the start of the hand")} title="Bid and play this deal again. It doesn't count toward the game.">
                  Replay this hand
                </button>
              )}
              {onCode && end !== undefined && (
                <button type="button" onClick={() => onCode(end)} title="A code for this hand, to ask about it later">
                  Hand code
                </button>
              )}
            </div>
          )}
        </>
      )}
      {tab === "decisions" && <DecisionList grades={grades} onReplay={onReplay} onCode={onCode} />}
      {tab === "tricks" && <Tricks hand={hand} coach={coach} save={save} onReplay={onReplay} />}
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
      {hand.bids && hand.bids.length > 0 && (
        <>
          <h3>The bids, as their bidders saw them</h3>
          <ul className="bid-notes">
            {hand.bids.map((b, i) => (
              <li key={i}>
                {SEAT_NAMES[b.seat]} ({players[b.seat].name}) bid <strong>{b.label}</strong>: a count of {b.count.toFixed(1)}
                {b.makes !== null && <>, and the coach gave it {Math.round(100 * b.makes)}% to make</>}.
              </li>
            ))}
          </ul>
        </>
      )}
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
export function DecisionList({ grades, onReplay, onCode }: { grades: GradeView[]; onReplay?: (at: number, label: string) => void; onCode?: (at: number) => void }) {
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
                  {g.bestLabel !== g.chosenLabel && g.verdict !== "best" && <> · best: {g.bestLabel}</>}
                  {g.loss >= 1 && <span className="loss"> (−{lossText(g)})</span>}
                  {g.hinted && (
                    <span className="hinted-tag" title="You asked for a hint on this decision, so it doesn't count toward your patterns">
                      after a hint
                    </span>
                  )}
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
                <MomentActions at={g.actionNumber} label={`your ${decisionLabel(g).toLowerCase()} decision`} onReplay={onReplay} onCode={onCode} />
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

function Tricks({ hand, coach, save, onReplay }: { hand: HandView; coach: Coach | null; save: string | null; onReplay?: (at: number, label: string) => void }) {
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
      <Contributions hand={hand} trace={trace} />
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
            {onReplay && (
              <button type="button" className="replay-trick" onClick={() => onReplay(trickStart(hand, i + 1), `trick ${i + 1}`)} title={`Play the hand again from trick ${i + 1}`}>
                Replay
              </button>
            )}
          </li>
        ))}
      </ol>
    </>
  );
}

/** What partner and the kitty added to the declarer's own cards, against the count's constant. */
function Contributions({ hand, trace }: { hand: HandView; trace: TraceView }) {
  const c = hand.contract!;
  // Real play runs about half a trick below the solver; the count estimates real play.
  const real = trace.par - 0.5;
  const added = real - trace.ownCount;
  const kitty = trace.par - trace.parWithoutKitty;
  const partner = added - kitty;
  const n = (x: number) => (x >= 0 ? `+${x.toFixed(1)}` : x.toFixed(1));
  return (
    <div className="contributions">
      <h3>What partner and the kitty added</h3>
      <p>
        {SEAT_NAMES[c.declarer]}'s own ten cards count <strong>{trace.ownCount.toFixed(1)}</strong> in {c.label.replace(/^\d+/, "")}. With every card
        visible the side takes {trace.par} (about {real.toFixed(1)} in real play), so partner and the kitty added <strong>{n(added)}</strong> tricks,
        against the <strong>{trace.constant === 3 ? "3" : "3⅓"}</strong> the count assumes.
      </p>
      <ul>
        <li>
          The kitty: <strong>{n(kitty)}</strong> ({trace.par} tricks with it, {trace.parWithoutKitty} had the declarer kept the cards they were dealt).
        </li>
        <li>
          Partner (and anything the count misjudged): <strong>{n(partner)}</strong>.
        </li>
      </ul>
    </div>
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
