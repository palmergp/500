import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ActionView, CardView, EventView, HandView, Persona, TrickView, View } from "./engine";
import { loadEngine, personas as listPersonas, SUIT_SYMBOL, Table } from "./engine";
import { BiddingBox } from "./components/BiddingBox";
import { Card } from "./components/Card";
import { Dialog } from "./components/Dialog";
import { HandReview } from "./components/HandReview";
import { NewGameDialog } from "./components/NewGameDialog";
import { ScorePad } from "./components/ScorePad";
import { TableView } from "./components/TableView";
import { loadSettings, savedGame, saveSettings, SPEED_FACTOR, type Settings } from "./settings";

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
  const tableRef = useRef<Table | null>(null);
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

  const factor = SPEED_FACTOR[settings.speed];

  const handle = useCallback((events: EventView[]) => {
    const table = tableRef.current!;
    const next = table.view();
    setView(next);
    savedGame.store(table.save());
    const shown = events.filter((e) => e.kind !== "discarded" && e.kind !== "dealt");
    if (shown.length > 0) setMessage(shown[shown.length - 1].text);
    if (events.some((e) => e.kind === "trickWon")) {
      setHeld(next.lastTrick ?? next.history.at(-1)?.tricks.at(-1) ?? null);
    }
    if (events.some((e) => e.kind === "handScored")) {
      setResult(next.history.filter((h) => !h.passedOut).at(-1) ?? null);
    }
  }, []);

  const startGame = useCallback((chosen: Settings) => {
    saveSettings(chosen);
    setSettings(chosen);
    const table = Table.create(randomSeed(), chosen);
    tableRef.current = table;
    savedGame.store(table.save());
    setView(table.view());
    setMessage("New game. First to 500 wins.");
    setHeld(null);
    setResult(null);
    setSelected([]);
    setJokerLeads(null);
    setShowNewGame(false);
  }, []);

  // Load the engine, then resume the saved game or deal a new one.
  useEffect(() => {
    if (tableRef.current) return;
    loadEngine()
      .then(() => {
        setPersonas(listPersonas());
        const save = savedGame.load();
        if (save) {
          try {
            tableRef.current = Table.load(save);
            setView(tableRef.current.view());
            setMessage("Welcome back: your game was restored.");
            return;
          } catch {
            savedGame.clear();
          }
        }
        setShowNewGame(true);
      })
      .catch((e: unknown) => setError(String(e)));
  }, []);

  // Show a finished trick for a moment before play continues.
  useEffect(() => {
    if (!held) return;
    const timer = setTimeout(() => setHeld(null), TRICK_HOLD * factor);
    return () => clearTimeout(timer);
  }, [held, factor]);

  const paused = !!held || !!result || !!review || showScores || showNewGame;

  // Let the bots take their turns, one at a time, at a readable pace.
  useEffect(() => {
    if (!view || paused || view.humanToAct || view.phase === "gameOver") return;
    const timer = setTimeout(() => {
      const events = tableRef.current?.stepBot();
      if (events) handle(events);
    }, DELAY[view.phase] * factor);
    return () => clearTimeout(timer);
  }, [view, paused, factor, handle]);

  const act = (index: number) => {
    setJokerLeads(null);
    handle(tableRef.current!.act(index));
  };

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
    const table = tableRef.current!;
    for (const id of selected) {
      const action = table.view().legal.find((a) => a.kind === "discard" && a.card === id);
      if (action) handle(table.act(action.index));
    }
    setSelected([]);
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
          <button type="button" onClick={() => setShowScores(true)} disabled={!view}>
            Score pad
          </button>
          <button type="button" onClick={() => setShowNewGame(true)} disabled={personas.length === 0}>
            New game
          </button>
        </nav>
      </header>

      {view ? (
        <TableView view={view} trick={trick}>
          <div className="status" role="status">
            {prompt(view) || message}
          </div>
          {view.humanToAct && !paused && view.phase === "bidding" && (
            <BiddingBox legal={view.legal} mustBid={view.mustBid} onAct={act} />
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
                onClick={legalCards.has(card.id) ? () => clickCard(card) : undefined}
              />
            ))}
          </div>
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
          <HandReview hand={review} players={view.players} />
        </Dialog>
      )}
      {showScores && view && (
        <Dialog title="Score pad" wide onClose={() => setShowScores(false)}>
          <ScorePad history={view.history} players={view.players} onReview={(h) => { setShowScores(false); setReview(h); }} />
        </Dialog>
      )}
      {showNewGame && personas.length > 0 && (
        <NewGameDialog personas={personas} initial={settings} onStart={startGame} onCancel={tableRef.current ? () => setShowNewGame(false) : undefined} />
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
