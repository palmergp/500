import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ActionView, CardView, GradeView, HandView, HintView, Persona, Snapshot, TrickView, View } from "./engine";
import { CoachClient, Engine, SUIT_SYMBOL } from "./engine";
import { BiddingBox } from "./components/BiddingBox";
import { Card } from "./components/Card";
import { CardTracker, GradeBar, HintPanel } from "./components/Coach";
import { CoachDialog } from "./components/CoachDialog";
import { Dialog } from "./components/Dialog";
import { HandReview } from "./components/HandReview";
import { NewGameDialog } from "./components/NewGameDialog";
import { ScorePad } from "./components/ScorePad";
import { TableView } from "./components/TableView";
import { loadSettings, savedGame, savedGrades, saveSettings, SPEED_FACTOR, type Settings } from "./settings";

const DELAY = { bidding: 700, discarding: 300, playing: 650, gameOver: 0 };
const TRICK_HOLD = 1300;

function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

function prompt(view: View): string {
  if (!view.humanToAct) return "";
  switch (view.phase) {
    case "bidding":
      return view.mustBid ? "Your call: as dealer on the third deal, you must bid." : "Your call.";
    case "discarding":
      return `You won the bid at ${view.contract?.label}. Choose ${3 - (view.discards?.length ?? 0)} card(s) to discard.`;
    case "playing":
      return view.trick && view.trick.plays.length > 0 ? "Your play." : "Your lead.";
    default:
      return "";
  }
}

export function App() {
  const engineRef = useRef<Engine | null>(null);
  const coachRef = useRef<CoachClient | null>(null);
  const saveRef = useRef<string | null>(null);
  const busy = useRef(false);
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [settings, setSettings] = useState<Settings>(loadSettings);
  const [message, setMessage] = useState("");
  const [held, setHeld] = useState<TrickView | null>(null);
  const [result, setResult] = useState<HandView | null>(null);
  const [review, setReview] = useState<HandView | null>(null);
  const [showScores, setShowScores] = useState(false);
  const [showNewGame, setShowNewGame] = useState(false);
  const [selected, setSelected] = useState<number[]>([]);
  const [jokerLeads, setJokerLeads] = useState<ActionView[] | null>(null);
  const [hint, setHint] = useState<{ forAction: number; hint: HintView } | null>(null);
  const [hintLoading, setHintLoading] = useState(false);
  const [grades, setGrades] = useState<GradeView[]>(() => savedGrades.load<GradeView>());
  const [lastGrade, setLastGrade] = useState<GradeView | null>(null);
  const [showCoach, setShowCoach] = useState(false);

  const factor = SPEED_FACTOR[settings.speed];

  const handle = useCallback((snapshot: Snapshot) => {
    const next = snapshot.view;
    if (snapshot.ms > 50) console.debug(`engine: ${snapshot.ms.toFixed(0)} ms (${snapshot.events.map((e) => e.text).join("; ")})`);
    setView(next);
    saveRef.current = snapshot.save;
    savedGame.store(snapshot.save);
    const shown = snapshot.events.filter((e) => e.kind !== "discarded" && e.kind !== "dealt");
    if (shown.length > 0) setMessage(shown[shown.length - 1].text);
    if (snapshot.events.some((e) => e.kind === "trickWon")) {
      setHeld(next.lastTrick ?? next.history.at(-1)?.tricks.at(-1) ?? null);
    }
    if (snapshot.events.some((e) => e.kind === "handScored")) {
      setResult(next.history.filter((h) => !h.passedOut).at(-1) ?? null);
    }
  }, []);

  /** Runs one engine request at a time, so moves can't interleave. */
  const request = useCallback(async (run: (engine: Engine) => Promise<Snapshot | null>) => {
    const engine = engineRef.current;
    if (!engine || busy.current) return null;
    busy.current = true;
    try {
      const snapshot = await run(engine);
      if (snapshot) handle(snapshot);
      return snapshot;
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      busy.current = false;
    }
  }, [handle]);

  const startGame = useCallback(
    (chosen: Settings) => {
      saveSettings(chosen);
      setSettings(chosen);
      savedGrades.clear();
      setGrades([]);
      setLastGrade(null);
      setHint(null);
      setHeld(null);
      setResult(null);
      setSelected([]);
      setJokerLeads(null);
      setShowNewGame(false);
      void request((engine) => engine.create(randomSeed(), chosen)).then(() => setMessage("New game. First to 500 wins."));
    },
    [request],
  );

  // Start the engine, then resume the saved game or ask for a new one.
  useEffect(() => {
    if (engineRef.current) return;
    const engine = new Engine();
    engineRef.current = engine;
    coachRef.current = new CoachClient();
    engine
      .personas()
      .then(async (list) => {
        setPersonas(list);
        const save = savedGame.load();
        if (save) {
          try {
            handle(await engine.load(save));
            setMessage("Welcome back: your game was restored.");
            return;
          } catch {
            savedGame.clear();
          }
        }
        setShowNewGame(true);
      })
      .catch((e: unknown) => setError(String(e)));
  }, [handle]);

  // Show a finished trick for a moment before play continues.
  useEffect(() => {
    if (!held) return;
    const timer = setTimeout(() => setHeld(null), TRICK_HOLD * factor);
    return () => clearTimeout(timer);
  }, [held, factor]);

  const paused = !!held || !!result || !!review || showScores || showNewGame || showCoach;

  // Let the bots take their turns one at a time. Thinking time counts toward the pause between
  // moves, so a slow search doesn't add to it.
  useEffect(() => {
    if (!view || paused || view.humanToAct || view.phase === "gameOver") return;
    const pause = new Promise((r) => setTimeout(r, DELAY[view.phase] * factor));
    void request(async (engine) => {
      const [snapshot] = await Promise.all([engine.stepBot(), pause]);
      return snapshot;
    });
  }, [view, paused, factor, request]);

  /** Asks the coach to grade your actions `numbers` (positions in the game's action log). */
  const gradeLater = (save: string, numbers: number[]) => {
    const coach = coachRef.current;
    if (!coach || !settings.grade) return;
    for (const number of numbers) {
      coach
        .grade(save, number)
        .then((g) => {
          if (!g) return;
          setGrades((all) => {
            const next = [...all.filter((x) => x.actionNumber !== g.actionNumber), g].sort((a, b) => a.actionNumber - b.actionNumber);
            savedGrades.store(next);
            return next;
          });
          setLastGrade(g);
        })
        .catch(() => {});
    }
  };

  const act = (index: number) => {
    if (!view) return;
    const number = view.actionCount;
    setJokerLeads(null);
    setHint(null);
    void request((engine) => engine.act(index)).then((snapshot) => snapshot && gradeLater(snapshot.save, [number]));
  };

  const askHint = () => {
    const coach = coachRef.current;
    const save = saveRef.current;
    if (!coach || !save || !view) return;
    const forAction = view.actionCount;
    setHintLoading(true);
    coach
      .hint(save)
      .then((h) => setHint({ forAction, hint: h }))
      .catch(() => {})
      .finally(() => setHintLoading(false));
  };

  /** Takes back the graded action and everything after it. */
  const undo = async (grade: GradeView) => {
    const save = saveRef.current;
    if (!save) return;
    while (busy.current) await new Promise((r) => setTimeout(r, 50));
    const data = JSON.parse(save) as { log: number[] };
    data.log = data.log.slice(0, grade.actionNumber);
    setHeld(null);
    setResult(null);
    setHint(null);
    setLastGrade(null);
    const snapshot = await request((engine) => engine.load(JSON.stringify(data)));
    if (snapshot) {
      setGrades((all) => {
        const next = all.filter((g) => g.actionNumber < grade.actionNumber);
        savedGrades.store(next);
        return next;
      });
      setMessage("Taken back: try again.");
    }
  };

  const changeSettings = (next: Settings) => {
    setSettings(next);
    saveSettings(next);
  };

  const currentHint = hint && view && hint.forAction === view.actionCount && view.humanToAct ? hint.hint : null;
  const hintedAction = currentHint ? view?.legal[currentHint.best] : undefined;

  const legalCards = useMemo(() => {
    if (!view?.humanToAct || paused) return new Set<number>();
    return new Set(view.legal.filter((a) => a.card !== null).map((a) => a.card!));
  }, [view, paused]);

  const kittyIds = useMemo(() => new Set(view?.phase === "discarding" ? view.kitty?.map((c) => c.id) : []), [view]);
  const discardsLeft = view ? 3 - (view.discards?.length ?? 0) : 0;

  const clickCard = (card: CardView) => {
    if (!view || !legalCards.has(card.id)) return;
    if (view.phase === "discarding") {
      setSelected((s) => (s.includes(card.id) ? s.filter((id) => id !== card.id) : s.length < discardsLeft ? [...s, card.id] : s));
      return;
    }
    const plays = view.legal.filter((a) => a.kind === "play" && a.card === card.id);
    if (plays.length === 1) act(plays[0].index);
    else setJokerLeads(plays); // leading the Joker in no trumps: name a suit
  };

  const confirmDiscards = () => {
    const chosen = selected;
    const numbers: number[] = [];
    setSelected([]);
    setHint(null);
    void request(async (engine) => {
      let snapshot: Snapshot | null = null;
      for (const id of chosen) {
        const current = (snapshot ?? { view }).view!;
        const action = current.legal.find((a) => a.kind === "discard" && a.card === id);
        if (action) {
          numbers.push(current.actionCount);
          snapshot = await engine.act(action.index);
        }
      }
      return snapshot;
    }).then((snapshot) => snapshot && gradeLater(snapshot.save, numbers));
  };

  if (error) return <div className="fatal">Could not start the game engine: {error}</div>;

  const trick = held ?? view?.trick ?? null;
  return (
    <div className="app">
      <header className="topbar">
        <h1>500</h1>
        {view && (
          <div className="scores" aria-label="Scores">
            <span className={view.scores[0] >= view.scores[1] ? "lead" : ""}>
              <span className="long">You &amp; {view.players[2].name}</span>
              <span className="short">Us</span> <b>{view.scores[0]}</b>
            </span>
            <span className={view.scores[1] > view.scores[0] ? "lead" : ""}>
              <span className="long">
                {view.players[1].name} &amp; {view.players[3].name}
              </span>
              <span className="short">Them</span> <b>{view.scores[1]}</b>
            </span>
          </div>
        )}
        <nav>
          <button type="button" onClick={() => setShowCoach(true)}>
            Coach
          </button>
          <button type="button" onClick={() => setShowScores(true)} disabled={!view}>
            Score pad
          </button>
          <button type="button" onClick={() => setShowNewGame(true)} disabled={personas.length === 0}>
            New game
          </button>
        </nav>
      </header>

      {view ? (
        <TableView view={view} trick={trick} trumpsUnseen={settings.trumpCounter ? view.trumpsUnseen : null}>
          <div className="status-row">
            <div className="status" role="status">
              {prompt(view) || message}
            </div>
            {view.humanToAct && !paused && view.phase !== "gameOver" && (
              <button type="button" className="hint-button" onClick={askHint} disabled={hintLoading || !!currentHint}>
                {hintLoading ? "Thinking…" : "Hint"}
              </button>
            )}
          </div>
          {currentHint && <HintPanel hint={currentHint} onClose={() => setHint(null)} />}
          {lastGrade && !currentHint && <GradeBar grade={lastGrade} onUndo={() => void undo(lastGrade)} onClose={() => setLastGrade(null)} />}
          {view.humanToAct && !paused && view.phase === "bidding" && (
            <BiddingBox legal={view.legal} mustBid={view.mustBid} onAct={act} hinted={hintedAction?.index} />
          )}
          {jokerLeads && (
            <div className="joker-choice">
              Lead the Joker and name a suit:
              {jokerLeads.map((a) => (
                <button key={a.index} type="button" className={`call ${a.namedSuit === "D" || a.namedSuit === "H" ? "red" : ""}`} onClick={() => act(a.index)}>
                  {SUIT_SYMBOL[a.namedSuit!]}
                </button>
              ))}
              <button type="button" onClick={() => setJokerLeads(null)}>
                Cancel
              </button>
            </div>
          )}
          {view.phase === "discarding" && view.humanToAct && (
            <div className="discard-bar">
              <button type="button" className="primary" disabled={selected.length !== discardsLeft} onClick={confirmDiscards}>
                Discard {selected.length}/{discardsLeft}
              </button>
            </div>
          )}
          <div className={`hand${view.hand.length > 10 ? " crowded" : ""}`} aria-label="Your hand">
            {view.hand.map((card) => (
              <Card
                key={card.id}
                card={card}
                playable={legalCards.has(card.id)}
                selected={selected.includes(card.id)}
                fromKitty={kittyIds.has(card.id)}
                hinted={hintedAction?.card === card.id}
                onClick={legalCards.has(card.id) ? () => clickCard(card) : undefined}
              />
            ))}
          </div>
          {settings.tracker && view.phase === "playing" && <CardTracker played={view.played} />}
        </TableView>
      ) : (
        <div className="loading">Shuffling…</div>
      )}

      {result && view && (
        <Dialog title={`Hand ${view.history.filter((h) => !h.passedOut).length}`}>
          <HandResult hand={result} view={view} />
          <div className="actions">
            <button type="button" onClick={() => setReview(result)}>
              Review hand
            </button>
            {view.phase === "gameOver" ? (
              <button type="button" className="primary" onClick={() => { setResult(null); setShowNewGame(true); }}>
                New game
              </button>
            ) : (
              <button type="button" className="primary" onClick={() => setResult(null)} autoFocus>
                Next hand
              </button>
            )}
          </div>
        </Dialog>
      )}
      {review && view && (
        <Dialog title="Hand review" wide onClose={() => setReview(null)}>
          <HandReview
            hand={review}
            players={view.players}
            grades={grades.filter((g) => g.deal === review.number)}
            coach={coachRef.current}
            save={saveRef.current}
          />
        </Dialog>
      )}
      {showScores && view && (
        <Dialog title="Score pad" wide onClose={() => setShowScores(false)}>
          <ScorePad history={view.history} players={view.players} onReview={(h) => { setShowScores(false); setReview(h); }} />
        </Dialog>
      )}
      {showCoach && <CoachDialog settings={settings} onChange={changeSettings} onClose={() => setShowCoach(false)} />}
      {showNewGame && personas.length > 0 && (
        <NewGameDialog personas={personas} initial={settings} onStart={startGame} onCancel={view ? () => setShowNewGame(false) : undefined} />
      )}
    </div>
  );
}

function HandResult({ hand, view }: { hand: HandView; view: View }) {
  const c = hand.contract!;
  const team = c.declarer % 2;
  const ours = team === 0;
  const who = ours ? (c.declarer === 0 ? "You" : `${view.players[c.declarer].name} (your partner)`) : view.players[c.declarer].name;
  const won = view.winner;
  return (
    <div className="hand-result">
      <p className={hand.made ? "made" : "down"}>
        {who} {hand.made ? "made" : "went down in"} <strong>{c.label}</strong> with {hand.tricksWon[team]} tricks.
      </p>
      <table>
        <tbody>
          <tr>
            <td>You &amp; {view.players[2].name}</td>
            <td>{hand.scoreChange[0] > 0 ? "+" : ""}{hand.scoreChange[0]}</td>
            <td><b>{hand.scoresAfter[0]}</b></td>
          </tr>
          <tr>
            <td>{view.players[1].name} &amp; {view.players[3].name}</td>
            <td>{hand.scoreChange[1] > 0 ? "+" : ""}{hand.scoreChange[1]}</td>
            <td><b>{hand.scoresAfter[1]}</b></td>
          </tr>
        </tbody>
      </table>
      {won !== null && <p className="game-over">{won === 0 ? "You and your partner win the game!" : `${view.players[1].name} and ${view.players[3].name} win the game.`}</p>}
    </div>
  );
}
