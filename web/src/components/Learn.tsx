import { useEffect, useRef, useState } from "react";
import type { ActionView, CardView, CoachClient, DrillView, GradeView } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { LESSONS, type Lesson } from "../learn/lessons";
import { loadProgress, recordScore, resetProgress, type Progress } from "../progress";
import { BiddingBox } from "./BiddingBox";
import { Card } from "./Card";
import { VerdictChip } from "./Coach";
import { Dialog } from "./Dialog";
import { TableView } from "./TableView";

type Tab = "lessons" | "bidding" | "puzzles" | "progress";

const DONE_KEY = "fh500.lessonsDone";

function loadDone(): string[] {
  try {
    return JSON.parse(localStorage.getItem(DONE_KEY) ?? "[]") as string[];
  } catch {
    return [];
  }
}

function saveDone(done: string[]) {
  try {
    localStorage.setItem(DONE_KEY, JSON.stringify(done));
  } catch {
    // Optional.
  }
}

function randomSeed() {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** Lessons, drills and progress. */
export function Learn({ coach, onClose }: { coach: CoachClient | null; onClose: () => void }) {
  const [tab, setTab] = useState<Tab>("lessons");
  const tabs: [Tab, string][] = [
    ["lessons", "Lessons"],
    ["bidding", "Bidding trainer"],
    ["puzzles", "Play puzzles"],
    ["progress", "Progress"],
  ];
  return (
    <Dialog title="Learn" wide onClose={onClose}>
      <div className="tabs" role="tablist">
        {tabs.map(([key, label]) => (
          <button key={key} type="button" role="tab" aria-selected={tab === key} className={tab === key ? "active" : ""} onClick={() => setTab(key)}>
            {label}
          </button>
        ))}
      </div>
      {tab === "lessons" && <Lessons />}
      {tab === "bidding" && <BiddingTrainer coach={coach} />}
      {tab === "puzzles" && <Puzzles coach={coach} />}
      {tab === "progress" && <ProgressView />}
    </Dialog>
  );
}

// ---- Lessons --------------------------------------------------------------------------------

function Lessons() {
  const [done, setDone] = useState<string[]>(loadDone);
  const [open, setOpen] = useState<Lesson | null>(null);
  if (open) {
    return (
      <LessonView
        lesson={open}
        onBack={() => setOpen(null)}
        onComplete={() => {
          const next = [...new Set([...done, open.id])];
          setDone(next);
          saveDone(next);
        }}
        done={done.includes(open.id)}
      />
    );
  }
  return (
    <>
      <p>
        {done.length} of {LESSONS.length} lessons done. The numbers in these lessons were measured: from solved deals, duplicate matches between
        bots, and the strongest bots' habits (see <code>docs/rules-of-thumb.md</code>).
      </p>
      <ol className="lesson-list">
        {LESSONS.map((lesson) => (
          <li key={lesson.id}>
            <button type="button" className="lesson-item" onClick={() => setOpen(lesson)}>
              <span className="lesson-title">
                {done.includes(lesson.id) ? "✓ " : ""}
                {lesson.title}
              </span>
              <small>{lesson.summary}</small>
            </button>
          </li>
        ))}
      </ol>
    </>
  );
}

function LessonView({ lesson, onBack, onComplete, done }: { lesson: Lesson; onBack: () => void; onComplete: () => void; done: boolean }) {
  const [answers, setAnswers] = useState<(number | null)[]>(() => lesson.quiz.map(() => null));
  const allAnswered = answers.every((a) => a !== null);
  const right = answers.filter((a, i) => a === lesson.quiz[i].answer).length;
  useEffect(() => {
    if (allAnswered && right === lesson.quiz.length && !done) onComplete();
  }, [allAnswered, right, lesson.quiz.length, done, onComplete]);
  return (
    <div className="lesson">
      <button type="button" className="link" onClick={onBack}>
        ← All lessons
      </button>
      <h3>{lesson.title}</h3>
      <LessonBody lines={lesson.body} />
      <h3>Check yourself</h3>
      {lesson.quiz.map((q, i) => (
        <div key={i} className="quiz-question">
          <p>{q.q}</p>
          <div className="quiz-options">
            {q.options.map((o, j) => {
              const picked = answers[i];
              const cls = picked === null ? "" : j === q.answer ? "right" : j === picked ? "wrong" : "";
              return (
                <button
                  key={o}
                  type="button"
                  className={cls}
                  disabled={picked !== null && j !== picked && j !== q.answer}
                  onClick={() => picked === null && setAnswers(answers.map((a, k) => (k === i ? j : a)))}
                >
                  {o}
                </button>
              );
            })}
          </div>
          {answers[i] !== null && <p className={answers[i] === q.answer ? "made" : "down"}>{q.explain}</p>}
        </div>
      ))}
      {allAnswered && (
        <p>
          <strong>
            {right} of {lesson.quiz.length} right.
          </strong>{" "}
          {right === lesson.quiz.length ? "Lesson complete." : "Read it again and try the questions once more later."}
        </p>
      )}
    </div>
  );
}

function LessonBody({ lines }: { lines: string[] }) {
  const out: React.ReactNode[] = [];
  let bullets: string[] = [];
  const flush = () => {
    if (bullets.length) {
      out.push(
        <ul key={out.length}>
          {bullets.map((b) => (
            <li key={b}>{b}</li>
          ))}
        </ul>,
      );
      bullets = [];
    }
  };
  for (const line of lines) {
    if (line.startsWith("• ")) {
      bullets.push(line.slice(2));
      continue;
    }
    flush();
    if (line.startsWith("## ")) out.push(<h4 key={out.length}>{line.slice(3)}</h4>);
    else out.push(<p key={out.length}>{line}</p>);
  }
  flush();
  return <>{out}</>;
}

// ---- Drills ---------------------------------------------------------------------------------

/** Loads drills one at a time, fetching the next while you work on the current one. */
function useDrills(load: (seed: number) => Promise<DrillView>) {
  const [current, setCurrent] = useState<DrillView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const next = useRef<Promise<DrillView> | null>(null);
  const fetchOne = () => load(randomSeed());
  const advance = () => {
    setCurrent(null);
    setError(null);
    const pending = next.current ?? fetchOne();
    next.current = fetchOne();
    pending.then(setCurrent, (e: unknown) => setError(String(e)));
  };
  useEffect(() => {
    advance();
  }, []); // once, on mount
  return { current, error, advance };
}

/** Grades the action at `index` taken from the drill's position. */
async function gradeDrill(coach: CoachClient, drill: DrillView, index: number): Promise<GradeView | null> {
  const save = JSON.parse(drill.save) as { log: number[] };
  const number = save.log.length;
  save.log.push(index);
  return coach.grade(JSON.stringify(save), number);
}

function DrillResult({ grade, onNext, nextLabel }: { grade: GradeView; onNext: () => void; nextLabel: string }) {
  return (
    <div className={`coach-panel grade ${grade.verdict}`}>
      <div className="coach-head">
        <span>
          <VerdictChip verdict={grade.verdict} /> {grade.chosenLabel}
          {grade.verdict !== "best" && grade.bestLabel !== grade.chosenLabel && (
            <>
              {" "}
              · best: <strong>{grade.bestLabel}</strong>
            </>
          )}
        </span>
        <button type="button" className="primary" onClick={onNext} autoFocus>
          {nextLabel}
        </button>
      </div>
      <ul>
        {grade.reasons.map((r) => (
          <li key={r}>{r}</li>
        ))}
      </ul>
    </div>
  );
}

function HandRow({ cards, label }: { cards: CardView[]; label: string }) {
  return (
    <div className="drill-hand" aria-label={label}>
      {cards.map((c) => (
        <Card key={c.id} card={c} />
      ))}
    </div>
  );
}

function BiddingTrainer({ coach }: { coach: CoachClient | null }) {
  const { current, error, advance } = useDrills((seed) => coach!.drillBidding(seed));
  const [grade, setGrade] = useState<GradeView | null>(null);
  const [busy, setBusy] = useState(false);
  if (!coach) return <p>The coach is still starting.</p>;
  if (error) return <p>Couldn't deal a hand: {error}</p>;
  if (!current) return <p className="waiting-text">Dealing…</p>;
  const view = current.view;
  const pick = (index: number) => {
    if (busy || grade) return;
    setBusy(true);
    gradeDrill(coach, current, index)
      .then((g) => {
        if (g) {
          setGrade(g);
          recordScore("bidding", g.verdict === "best" || g.verdict === "good");
        }
      })
      .finally(() => setBusy(false));
  };
  return (
    <div className="drill">
      <p>
        You are South; {SEAT_NAMES[view.dealer]} dealt (deal {view.dealNumber} of 3). What do you call?
      </p>
      <p className="auction-line">
        {view.calls.length === 0 ? <span>You speak first.</span> : view.calls.map((c, i) => <span key={i}>{SEAT_NAMES[c.seat]}: <strong>{c.label}</strong></span>)}
      </p>
      <HandRow cards={view.hand} label="Your hand" />
      {grade ? (
        <DrillResult grade={grade} nextLabel="Next hand" onNext={() => { setGrade(null); advance(); }} />
      ) : (
        <BiddingBox legal={view.legal} mustBid={view.mustBid} onAct={pick} />
      )}
    </div>
  );
}

function Puzzles({ coach }: { coach: CoachClient | null }) {
  const { current, error, advance } = useDrills((seed) => coach!.drillPlay(seed));
  const [grade, setGrade] = useState<GradeView | null>(null);
  const [jokerLeads, setJokerLeads] = useState<ActionView[] | null>(null);
  if (!coach) return <p>The coach is still starting.</p>;
  if (error) return <p>{error}</p>;
  if (!current) return <p className="waiting-text">Finding a position with one clearly best play…</p>;
  const view = current.view;
  const choose = (index: number) => {
    setJokerLeads(null);
    gradeDrill(coach, current, index).then((g) => {
      if (g) {
        setGrade(g);
        recordScore("puzzles", g.verdict === "best" || g.verdict === "good");
      }
    });
  };
  const clickCard = (card: CardView) => {
    if (grade) return;
    const plays = view.legal.filter((a) => a.card === card.id);
    if (plays.length === 1) choose(plays[0].index);
    else if (plays.length > 1) setJokerLeads(plays);
  };
  const legal = new Set(view.legal.map((a) => a.card));
  return (
    <div className="drill puzzle">
      <p>
        Find the best card. {view.contract ? `${SEAT_NAMES[view.contract.declarer]} is playing ${view.contract.label}; ` : ""}
        you are South, trick {view.tricksWon[0] + view.tricksWon[1] + 1}. One play here is clearly better than the rest.
      </p>
      <TableView view={view} trick={view.trick} trumpsUnseen={view.trumpsUnseen} showVoids>
        {jokerLeads && (
          <div className="joker-choice">
            Lead the Joker and name a suit:
            {jokerLeads.map((a) => (
              <button key={a.index} type="button" className="call" onClick={() => choose(a.index)}>
                {SUIT_SYMBOL[a.namedSuit!]}
              </button>
            ))}
          </div>
        )}
        {grade && <DrillResult grade={grade} nextLabel="Next puzzle" onNext={() => { setGrade(null); advance(); }} />}
        <div className="hand" aria-label="Your hand">
          {view.hand.map((card) => (
            <Card key={card.id} card={card} playable={!grade && legal.has(card.id)} onClick={!grade && legal.has(card.id) ? () => clickCard(card) : undefined} />
          ))}
        </div>
      </TableView>
    </div>
  );
}

// ---- Progress -------------------------------------------------------------------------------

const VERDICTS = ["best", "good", "inaccuracy", "mistake", "blunder"] as const;

function ProgressView() {
  const [progress, setProgress] = useState<Progress>(loadProgress);
  const games = progress.games.filter((g) => g.decisions > 0);
  const decisions = games.reduce((s, g) => s + g.decisions, 0);
  const totals = VERDICTS.map((v) => games.reduce((s, g) => s + g.verdicts[v], 0));
  const lossPer = (g: { loss: number; decisions: number }) => (g.decisions ? g.loss / g.decisions : 0);
  const recent = games.slice(-12);
  const maxLoss = Math.max(1, ...recent.map(lossPer));
  const pct = (s: { asked: number; correct: number }) => (s.asked ? `${Math.round((100 * s.correct) / s.asked)}%` : "—");
  const [tip, setTip] = useState<string | null>(null);
  return (
    <div className="progress">
      <div className="stat-tiles">
        <div className="stat">
          <span className="stat-value">{decisions}</span>
          <span className="stat-label">decisions graded in {games.length} game{games.length === 1 ? "" : "s"}</span>
        </div>
        <div className="stat">
          <span className="stat-value">{decisions ? (games.reduce((s, g) => s + g.loss, 0) / decisions).toFixed(1) : "—"}</span>
          <span className="stat-label">expected points given up per decision</span>
        </div>
        <div className="stat">
          <span className="stat-value">{pct(progress.quiz)}</span>
          <span className="stat-label">counting quiz ({progress.quiz.asked})</span>
        </div>
        <div className="stat">
          <span className="stat-value">{pct(progress.bidding)}</span>
          <span className="stat-label">bidding trainer ({progress.bidding.asked})</span>
        </div>
        <div className="stat">
          <span className="stat-value">{pct(progress.puzzles)}</span>
          <span className="stat-label">play puzzles ({progress.puzzles.asked})</span>
        </div>
      </div>

      {decisions > 0 && (
        <>
          <h4>How your decisions were graded</h4>
          <div className="verdict-bar" role="img" aria-label={VERDICTS.map((v, i) => `${v} ${totals[i]}`).join(", ")}>
            {VERDICTS.map((v, i) =>
              totals[i] ? <span key={v} className={`seg ${v}`} style={{ flexGrow: totals[i] }} title={`${v}: ${totals[i]}`} /> : null,
            )}
          </div>
          <div className="verdict-legend">
            {VERDICTS.map((v, i) => (
              <span key={v}>
                <VerdictChip verdict={v} /> {totals[i]} ({Math.round((100 * totals[i]) / decisions)}%)
              </span>
            ))}
          </div>

          <h4>Points given up per decision, recent games</h4>
          <div className="trend" onMouseLeave={() => setTip(null)}>
            {recent.map((g, i) => (
              <div
                key={g.id}
                className="trend-col"
                onMouseEnter={() => setTip(`${g.date}: ${lossPer(g).toFixed(1)} points per decision over ${g.decisions} decisions${g.finished ? (g.won ? ", won" : ", lost") : ""}`)}
              >
                <span className="trend-bar" style={{ height: `${(100 * lossPer(g)) / maxLoss}%` }} />
                <span className="trend-label">{i + 1}</span>
              </div>
            ))}
          </div>
          <p className="coach-foot">{tip ?? "Lower is better. Hover a bar for that game."}</p>
          <table className="score-pad">
            <thead>
              <tr>
                <th>Game</th>
                <th>Date</th>
                <th>Decisions</th>
                <th>Per decision</th>
                <th>Slips (inaccuracy or worse)</th>
                <th>Result</th>
              </tr>
            </thead>
            <tbody>
              {recent.map((g, i) => (
                <tr key={g.id}>
                  <td>{i + 1}</td>
                  <td>{g.date}</td>
                  <td>{g.decisions}</td>
                  <td>{lossPer(g).toFixed(1)}</td>
                  <td>{g.verdicts.inaccuracy + g.verdicts.mistake + g.verdicts.blunder}</td>
                  <td>{g.finished ? (g.won ? "won" : "lost") : "in progress"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
      {decisions === 0 && <p>Play with “Grade my decisions” on (Coach menu) and your progress will appear here.</p>}
      <div className="actions">
        <button
          type="button"
          onClick={() => {
            if (confirm("Clear all progress records?")) {
              resetProgress();
              setProgress(loadProgress());
            }
          }}
        >
          Reset progress
        </button>
      </div>
    </div>
  );
}
