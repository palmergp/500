import { useEffect, useRef, useState } from "react";
import type { ActionView, CardView, CoachClient, DrillView, GradeView } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { LESSONS, type Lesson } from "../learn/lessons";
import { leaks, loadProgress, recordScore, resetProgress, type Progress } from "../progress";
import { loadRecords, resetRecords, type Records } from "../records";
import { PATTERNS } from "../learn/patterns";
import { loadMistakes, solveMistake, type Mistake } from "../mistakes";
import { BiddingBox } from "./BiddingBox";
import { Card } from "./Card";
import { VerdictChip } from "./Coach";
import { Dialog } from "./Dialog";
import { TableView } from "./TableView";

type Tab = "lessons" | "bidding" | "puzzles" | "mistakes" | "progress";

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
  /** A lesson to open, from a pattern on the Progress tab. */
  const [lesson, setLesson] = useState<string | null>(null);
  const tabs: [Tab, string][] = [
    ["lessons", "Lessons"],
    ["bidding", "Bidding trainer"],
    ["puzzles", "Play puzzles"],
    ["mistakes", "Your mistakes"],
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
      {tab === "lessons" && <Lessons key={lesson ?? ""} initial={lesson} />}
      {tab === "bidding" && <BiddingTrainer coach={coach} />}
      {tab === "puzzles" && <Puzzles coach={coach} />}
      {tab === "mistakes" && <MyMistakes coach={coach} />}
      {tab === "progress" && (
        <ProgressView
          onLesson={(id) => {
            setLesson(id);
            setTab("lessons");
          }}
        />
      )}
    </Dialog>
  );
}

// ---- Lessons --------------------------------------------------------------------------------

function Lessons({ initial }: { initial?: string | null }) {
  const [done, setDone] = useState<string[]>(loadDone);
  const [open, setOpen] = useState<Lesson | null>(() => LESSONS.find((l) => l.id === initial) ?? null);
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
  // A batch of deals occasionally holds no clear puzzle; quietly try fresh ones before giving up.
  const fetchOne = (tries = 3): Promise<DrillView> => load(randomSeed()).catch((e: unknown) => (tries > 1 ? fetchOne(tries - 1) : Promise.reject(e)));
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

// ---- Your own mistakes, replayed ------------------------------------------------------------

/** Positions from your games where you made a mistake or a blunder, to try again. A good answer
 * clears the puzzle; a miss keeps it for later. */
function MyMistakes({ coach }: { coach: CoachClient | null }) {
  const [list, setList] = useState<Mistake[]>(() => loadMistakes().reverse());
  const [index, setIndex] = useState(0);
  const [drill, setDrill] = useState<DrillView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [grade, setGrade] = useState<GradeView | null>(null);
  const [jokerLeads, setJokerLeads] = useState<ActionView[] | null>(null);
  const current = list[index] ?? null;
  useEffect(() => {
    setDrill(null);
    setError(null);
    if (!coach || !current) return;
    coach.position(current.save).then(setDrill, (e: unknown) => setError(String(e)));
  }, [coach, current]);
  if (!coach) return <p>The coach is still starting.</p>;
  if (list.length === 0) {
    return (
      <p>
        No mistakes saved yet. When the coach grades one of your bids or cards in a game on this device a mistake or a blunder (and you hadn't
        asked for a hint), the position is kept here so you can try it again.
      </p>
    );
  }
  if (!current) {
    return (
      <div>
        <p>That's all of them for now.</p>
        <button type="button" className="primary" onClick={() => { setList(loadMistakes().reverse()); setIndex(0); }}>
          Start again with the ones left ({loadMistakes().length})
        </button>
      </div>
    );
  }
  if (error) return <p>Couldn't rebuild that position: {error}</p>;
  if (!drill) return <p className="waiting-text">Setting up the position…</p>;
  const view = drill.view;
  const choose = (i: number) => {
    if (grade) return;
    setJokerLeads(null);
    gradeDrill(coach, drill, i).then((g) => {
      if (!g) return;
      setGrade(g);
      recordScore("puzzles", g.verdict === "best" || g.verdict === "good");
      if (g.verdict === "best" || g.verdict === "good") solveMistake(current.save);
    });
  };
  const next = () => {
    setGrade(null);
    setIndex((i) => i + 1);
  };
  const clickCard = (card: CardView) => {
    if (grade) return;
    const plays = view.legal.filter((a) => a.card === card.id);
    if (plays.length === 1) choose(plays[0].index);
    else if (plays.length > 1) setJokerLeads(plays);
  };
  const legal = new Set(view.legal.map((a) => a.card));
  return (
    <div className={`drill${current.phase === "playing" ? " puzzle" : ""}`}>
      <p>
        From your game on {current.date} ({index + 1} of {list.length}). You played <strong>{current.chosenLabel}</strong>, graded a{" "}
        {current.verdict}. What's the better choice?
      </p>
      {current.phase === "bidding" ? (
        <>
          <p className="auction-line">
            {view.calls.length === 0 ? <span>You speak first.</span> : view.calls.map((c, i) => <span key={i}>{SEAT_NAMES[c.seat]}: <strong>{c.label}</strong></span>)}
          </p>
          <HandRow cards={view.hand} label="Your hand" />
          {grade ? (
            <DrillResult grade={grade} nextLabel="Next mistake" onNext={next} />
          ) : (
            <BiddingBox legal={view.legal} mustBid={view.mustBid} onAct={choose} />
          )}
        </>
      ) : (
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
          {grade && <DrillResult grade={grade} nextLabel="Next mistake" onNext={next} />}
          <div className="hand" aria-label="Your hand">
            {view.hand.map((card) => (
              <Card key={card.id} card={card} playable={!grade && legal.has(card.id)} onClick={!grade && legal.has(card.id) ? () => clickCard(card) : undefined} />
            ))}
          </div>
        </TableView>
      )}
    </div>
  );
}

// ---- Progress -------------------------------------------------------------------------------

const VERDICTS = ["best", "good", "inaccuracy", "mistake", "blunder"] as const;

/** Your records, and each bid you've declared with how often you made it. */
function RecordsView({ records }: { records: Records }) {
  const bids = Object.entries(records.bids).sort((a, b) => b[1].bids - a[1].bids || a[0].localeCompare(b[0]));
  const declared = bids.reduce((s, [, b]) => s + b.bids, 0);
  const made = bids.reduce((s, [, b]) => s + b.made, 0);
  return (
    <>
      <h4>Records</h4>
      <div className="stat-tiles">
        <div className="stat">
          <span className="stat-value">{records.highGame ?? "—"}</span>
          <span className="stat-label">high game (your side's final score)</span>
        </div>
        <div className="stat">
          <span className="stat-value">{records.bestHand ? `+${records.bestHand.points}` : "—"}</span>
          <span className="stat-label">best hand{records.bestHand ? ` (${records.bestHand.label})` : ""}</span>
        </div>
        <div className="stat">
          <span className="stat-value">{records.highBid?.label ?? "—"}</span>
          <span className="stat-label">highest bid you've made</span>
        </div>
        <div className="stat">
          <span className="stat-value">{records.gamesPlayed ? `${records.gamesWon} of ${records.gamesPlayed}` : "—"}</span>
          <span className="stat-label">games won</span>
        </div>
      </div>
      <h4>Your bids</h4>
      {declared === 0 ? (
        <p className="note">Win an auction and play the hand: each bid you declare, and whether you made it, shows here.</p>
      ) : (
        <>
          <p className="note">
            You've declared {declared} {declared === 1 ? "bid" : "bids"} and made {Math.round((100 * made) / declared)}% of them. Most often:{" "}
            <strong>{bids[0][0]}</strong> ({bids[0][1].bids} times).
          </p>
          <table className="score-pad bid-stats">
            <thead>
              <tr>
                <th>Bid</th>
                <th>Times</th>
                <th>Made</th>
              </tr>
            </thead>
            <tbody>
              {bids.map(([label, b]) => (
                <tr key={label}>
                  <td>{label}</td>
                  <td>{b.bids}</td>
                  <td>
                    {Math.round((100 * b.made) / b.bids)}% ({b.made})
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
    </>
  );
}

function ProgressView({ onLesson }: { onLesson: (id: string) => void }) {
  const [progress, setProgress] = useState<Progress>(loadProgress);
  const [records, setRecords] = useState<Records>(loadRecords);
  const myLeaks = leaks(progress).slice(0, 3);
  const hinted = progress.games.reduce((s, g) => s + (g.hinted ?? 0), 0);
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

      <RecordsView records={records} />

      <h4>Your patterns</h4>
      {myLeaks.length === 0 ? (
        <p className="note">
          Play a few more graded hands: your most costly recurring slips will show here, with what to practice. Decisions made after you ask for a
          hint don't count, since the hint shaped them.
        </p>
      ) : (
        <>
          <p className="note">
            Your costliest recurring slips over your last 20 games{hinted ? `, leaving out ${hinted} decisions made after a hint` : ""}.
          </p>
          <ol className="leaks">
            {myLeaks.map((l) => {
              const p = PATTERNS[l.pattern];
              const lessonTitle = LESSONS.find((x) => x.id === p?.lesson)?.title;
              return (
                <li key={l.pattern}>
                  <div className="leak-head">
                    <strong>{p?.title ?? l.pattern}</strong>
                    <span className="loss">
                      {l.slips} times, about {Math.round(l.loss)} points in all
                    </span>
                  </div>
                  {p && <p>{p.advice}</p>}
                  {p && lessonTitle && (
                    <button type="button" onClick={() => onLesson(p.lesson)}>
                      Lesson: {lessonTitle}
                    </button>
                  )}
                </li>
              );
            })}
          </ol>
        </>
      )}

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
            if (confirm("Clear all progress, records and bid stats?")) {
              resetProgress();
              resetRecords();
              setProgress(loadProgress());
              setRecords(loadRecords());
            }
          }}
        >
          Reset progress
        </button>
      </div>
    </div>
  );
}
