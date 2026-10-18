import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ActionView, CardView, Coach, GradeView, HandView, HintView, Persona, Snapshot, TrickView, View } from "./engine";
import { CoachClient, Engine, SUIT_SYMBOL } from "./engine";
import { detectHost, NetClient, savedName, type HostInfo, type Lobby as LobbyState } from "./net";
import { Lobby } from "./components/Lobby";
import { BiddingBox } from "./components/BiddingBox";
import { Card } from "./components/Card";
import { CardTracker, GradeBar, HintPanel } from "./components/Coach";
import { Celebration } from "./components/Celebration";
import { CountingQuiz, quizDue } from "./components/CountingQuiz";
import { Learn } from "./components/Learn";
import { recordGrades } from "./progress";
import { audio } from "./audio";
import { CoachDialog } from "./components/CoachDialog";
import { Dialog } from "./components/Dialog";
import { HandReview } from "./components/HandReview";
import { NewGameDialog } from "./components/NewGameDialog";
import { ScorePad } from "./components/ScorePad";
import { TableView } from "./components/TableView";
import { loadSettings, savedGame, savedGrades, saveSettings, SPEED_FACTOR, type Settings } from "./settings";

const DELAY = { bidding: 700, discarding: 300, playing: 650, gameOver: 0 };
const TRICK_HOLD = 1300;

/** Identifies a game for the progress record: its seed. */
function gameId(save: string): string {
  try {
    return String((JSON.parse(save) as { seed: number }).seed);
  } catch {
    return "unknown";
  }
}

/** On a LAN host, whether you chose to play solo instead (remembered for this tab). */
const soloOnHost = {
  get: () => {
    try {
      return sessionStorage.getItem("fh500.soloOnHost") === "1";
    } catch {
      return false;
    }
  },
  set: (on: boolean) => {
    try {
      if (on) sessionStorage.setItem("fh500.soloOnHost", "1");
      else sessionStorage.removeItem("fh500.soloOnHost");
    } catch {
      // Not remembered.
    }
  },
};

function names(list: string[]): string {
  return list.length <= 1 ? list.join("") : `${list.slice(0, -1).join(", ")} and ${list.at(-1)}`;
}

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
  const [sweeping, setSweeping] = useState(false);
  const [quizzed, setQuizzed] = useState<number | null>(null);
  const [showLearn, setShowLearn] = useState(false);
  const [announcement, setAnnouncement] = useState("");
  // Playing at a table on a LAN host (crates/host) rather than against bots on this device.
  const [mode, setMode] = useState<"starting" | "solo" | "lan">("starting");
  const modeRef = useRef(mode);
  const netRef = useRef<NetClient | null>(null);
  const gameRef = useRef<string | null>(null);
  const gradesRef = useRef(grades);
  const [host, setHost] = useState<HostInfo | null>(null);
  const [lobby, setLobby] = useState<LobbyState | null>(null);
  const [connected, setConnected] = useState(true);
  const [name, setName] = useState(savedName);
  const [showLobby, setShowLobby] = useState(false);
  const lan = mode === "lan";
  useEffect(() => {
    gradesRef.current = grades;
  }, [grades]);

  const factor = SPEED_FACTOR[lan && lobby ? lobby.pace : settings.speed];
  /** Answers hints and reviews: the host for a LAN game (it holds the cards), else this device. */
  const coach: Coach | null = lan ? netRef.current : coachRef.current;
  /** The coach rebuilds positions from the save; a LAN host already has the game. */
  const coachSave = () => (modeRef.current === "lan" ? "lan" : saveRef.current);

  const handle = useCallback((snapshot: Snapshot) => {
    const next = snapshot.view;
    if (snapshot.ms > 50) console.debug(`engine: ${snapshot.ms.toFixed(0)} ms (${snapshot.events.map((e) => e.text).join("; ")})`);
    setView(next);
    saveRef.current = snapshot.save;
    gameRef.current = snapshot.game ?? gameId(snapshot.save);
    if (modeRef.current === "solo") savedGame.store(snapshot.save);
    for (const e of snapshot.events) {
      if (e.kind === "played") audio.play("card");
      else if (e.kind === "dealt") audio.play("deal");
      else if (e.kind === "called") audio.play(e.text.endsWith("Pass") ? "pass" : "bid");
      else if (e.kind === "contractWon") audio.play("contract");
      else if (e.kind === "trickWon") audio.play(e.seat !== null && e.seat % 2 === 0 ? "trickUs" : "trickThem");
      else if (e.kind === "handScored") {
        const hand = next.history.filter((h) => !h.passedOut).at(-1);
        const ours = hand?.contract ? hand.contract.declarer % 2 === 0 : false;
        audio.play(hand && ours === !!hand.made ? "made" : "set");
      } else if (e.kind === "gameWon") {
        audio.play(next.winner === 0 ? "win" : "lose");
        recordGrades(gameRef.current, gradesRef.current, true, next.winner === 0);
      }
    }
    // Screen readers hear every call, card, trick and score as it happens.
    const spoken = snapshot.events.filter((e) => e.kind !== "discarded").map((e) => e.text);
    if (spoken.length > 0) setAnnouncement(spoken.join(". "));
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
      if (modeRef.current === "lan") setMessage(e instanceof Error ? e.message : String(e));
      else setError(String(e));
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

  const clearTable = useCallback(() => {
    setView(null);
    setHeld(null);
    setResult(null);
    setReview(null);
    setHint(null);
    setLastGrade(null);
    setSelected([]);
    setJokerLeads(null);
    setMessage("");
  }, []);

  /** Against bots on this device: resume the saved game or ask for a new one. */
  const startSolo = useCallback(async () => {
    netRef.current?.close();
    netRef.current = null;
    modeRef.current = "solo";
    setMode("solo");
    clearTable();
    setGrades(savedGrades.load<GradeView>());
    const save = savedGame.load();
    if (save && engineRef.current) {
      try {
        handle(await engineRef.current.load(save));
        setMessage("Welcome back: your game was restored.");
        return;
      } catch {
        savedGame.clear();
      }
    }
    setShowNewGame(true);
  }, [handle, clearTable]);

  /** At the LAN host's table. */
  const joinHost = useCallback(() => {
    modeRef.current = "lan";
    setMode("lan");
    clearTable();
    setGrades([]);
    setShowNewGame(false);
    netRef.current?.close();
    netRef.current = new NetClient(
      {
        lobby: setLobby,
        snapshot: handle,
        ended: () => {
          clearTable();
          setGrades([]);
        },
        notice: setMessage,
        connection: setConnected,
      },
      savedName() || "Player",
    );
  }, [handle, clearTable]);

  // Start the engine; join the table if the app came from a LAN host, else play solo.
  useEffect(() => {
    if (engineRef.current) return;
    const engine = new Engine();
    engineRef.current = engine;
    coachRef.current = new CoachClient();
    engine
      .personas()
      .then(async (list) => {
        setPersonas(list);
        const info = await detectHost();
        setHost(info);
        if (info && !soloOnHost.get()) joinHost();
        else await startSolo();
      })
      .catch((e: unknown) => setError(String(e)));
  }, [startSolo, joinHost]);

  // Show a finished trick for a moment before play continues.
  useEffect(() => {
    if (!held) return;
    setSweeping(false);
    const sweep = setTimeout(() => setSweeping(true), TRICK_HOLD * factor * 0.65);
    const timer = setTimeout(() => setHeld(null), TRICK_HOLD * factor);
    return () => {
      clearTimeout(sweep);
      clearTimeout(timer);
    };
  }, [held, factor]);

  // Audio may only start after a user gesture.
  useEffect(() => {
    audio.sfxOn = settings.sound;
    const unlock = () => {
      audio.unlock();
      audio.setMusic(settings.music);
    };
    window.addEventListener("pointerdown", unlock);
    window.addEventListener("keydown", unlock);
    return () => {
      window.removeEventListener("pointerdown", unlock);
      window.removeEventListener("keydown", unlock);
    };
  }, [settings.sound, settings.music]);

  const lobbyOpen = lan && (showLobby || !lobby?.playing || lobby.you === null || !view);
  // At a LAN table, nobody moves until everyone has seen the last hand's result.
  const waitingOnOthers = lan && !!lobby && lobby.waiting.length > 0;
  const paused = !!held || !!result || !!review || showScores || showNewGame || showCoach || showLearn || lobbyOpen || waitingOnOthers;

  // Let the bots take their turns one at a time. Thinking time counts toward the pause between
  // moves, so a slow search doesn't add to it.
  useEffect(() => {
    if (mode !== "solo" || !view || paused || view.humanToAct || view.phase === "gameOver") return;
    const pause = new Promise((r) => setTimeout(r, DELAY[view.phase] * factor));
    void request(async (engine) => {
      const [snapshot] = await Promise.all([engine.stepBot(), pause]);
      return snapshot;
    });
  }, [mode, view, paused, factor, request]);

  /** Asks the coach to grade your actions `numbers` (positions in the game's action log). */
  const gradeLater = (save: string, numbers: number[]) => {
    if (!coach || !settings.grade) return;
    for (const number of numbers) {
      coach
        .grade(save, number)
        .then((g) => {
          if (!g) return;
          setGrades((all) => {
            const next = [...all.filter((x) => x.actionNumber !== g.actionNumber), g].sort((a, b) => a.actionNumber - b.actionNumber);
            if (modeRef.current === "solo") savedGrades.store(next);
            recordGrades(gameRef.current ?? gameId(save), next);
            return next;
          });
          setLastGrade(g);
        })
        .catch(() => {});
    }
  };

  /** Your action: to this device's engine, or to the host (`expect` guards against a stale click). */
  const send = (engine: Engine, index: number, expect: number) =>
    modeRef.current === "lan" && netRef.current ? netRef.current.act(index, expect) : engine.act(index);

  const act = (index: number) => {
    if (!view) return;
    const number = view.actionCount;
    setJokerLeads(null);
    setHint(null);
    void request((engine) => send(engine, index, number)).then((snapshot) => snapshot && gradeLater(snapshot.save, [number]));
  };

  const askHint = () => {
    const save = coachSave();
    if (!coach || !save || !view) return;
    const forAction = view.actionCount;
    setHintLoading(true);
    coach
      .hint(save)
      .then((h) => {
        setHint({ forAction, hint: h });
        audio.play("hint");
      })
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

  const quizActive = !!view && settings.quiz && !paused && quizzed !== view.actionCount && quizDue(view);
  const currentHint = hint && view && hint.forAction === view.actionCount && view.humanToAct ? hint.hint : null;
  const hintedAction = currentHint ? view?.legal[currentHint.best] : undefined;

  const legalCards = useMemo(() => {
    if (!view?.humanToAct || paused || (settings.quiz && view && quizzed !== view.actionCount && quizDue(view))) return new Set<number>();
    return new Set(view.legal.filter((a) => a.card !== null).map((a) => a.card!));
  }, [view, paused, settings.quiz, quizzed]);

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
          snapshot = await send(engine, action.index, current.actionCount);
        }
      }
      return snapshot;
    }).then((snapshot) => snapshot && gradeLater(snapshot.save, numbers));
  };

  if (error) return <div className="fatal">Could not start the game engine: {error}</div>;

  /** On a LAN table: who or what the game is waiting on, when it isn't you. */
  const lanStatus = (() => {
    if (!lan || !lobby || lobby.you === null || !view) return null;
    if (!connected) return { text: "Reconnecting to the host…" };
    if (lobby.waiting.length > 0 && !result) return { text: `Waiting for ${names(lobby.waiting)} to see the result…` };
    if (view.toAct !== null && !view.humanToAct && view.phase !== "gameOver") {
      const seat = (view.toAct + lobby.you) % 4;
      const person = lobby.seats[seat].person;
      if (person && !person.connected) return { text: `Waiting for ${person.name} to reconnect…`, seat };
    }
    return null;
  })();
  const leaveSolo = () => {
    soloOnHost.set(false);
    joinHost();
  };

  const trick = held ?? view?.trick ?? null;
  return (
    <div className={`app${settings.fourColor ? " four-color" : ""}${settings.bigCards ? " big-cards" : ""}`}>
      <div className="visually-hidden" aria-live="polite" aria-atomic="true">
        {announcement}
      </div>
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
          <button
            type="button"
            className="icon-toggle"
            aria-pressed={settings.sound}
            title={settings.sound ? "Sound on" : "Sound off"}
            aria-label="Sound effects"
            onClick={() => changeSettings({ ...settings, sound: !settings.sound })}
          >
            {settings.sound ? "🔊" : "🔇"}
          </button>
          <button
            type="button"
            className={`icon-toggle${settings.music ? "" : " off"}`}
            aria-pressed={settings.music}
            title={settings.music ? "Music on" : "Music off"}
            aria-label="Music"
            onClick={() => {
              audio.unlock();
              audio.setMusic(!settings.music);
              changeSettings({ ...settings, music: !settings.music });
            }}
          >
            🎵
          </button>
          <button type="button" className="learn-button" onClick={() => setShowLearn(true)}>
            Learn
          </button>
          <button type="button" onClick={() => setShowCoach(true)}>
            Coach
          </button>
          <button type="button" onClick={() => setShowScores(true)} disabled={!view}>
            Score pad
          </button>
          {lan ? (
            <button type="button" onClick={() => setShowLobby(true)}>
              Table
            </button>
          ) : (
            <>
              {host && (
                <button type="button" onClick={leaveSolo}>
                  Join table
                </button>
              )}
              <button type="button" onClick={() => setShowNewGame(true)} disabled={personas.length === 0}>
                New game
              </button>
            </>
          )}
        </nav>
      </header>

      {view ? (
        <TableView
          view={view}
          trick={trick}
          sweepTo={held && sweeping ? held.winner : null}
          trumpsUnseen={settings.trumpCounter && !quizActive ? view.trumpsUnseen : null}
          showVoids={settings.voids && !quizActive}
        >
          <div className="status-row">
            <div className="status" role="status">
              {lanStatus?.text || prompt(view) || message}
            </div>
            {lanStatus?.seat !== undefined && lobby && (
              <button type="button" onClick={() => netRef.current?.setBot(lanStatus.seat!, lobby.seats[lanStatus.seat!].bot)}>
                Let a bot play for them
              </button>
            )}
            {view.humanToAct && !paused && view.phase !== "gameOver" && (
              <button type="button" className="hint-button" onClick={askHint} disabled={hintLoading || !!currentHint}>
                {hintLoading ? "Thinking…" : "Hint"}
              </button>
            )}
          </div>
          {quizActive && <CountingQuiz key={view.actionCount} view={view} onDone={() => setQuizzed(view.actionCount)} />}
          {currentHint && <HintPanel hint={currentHint} onClose={() => setHint(null)} />}
          {lastGrade && !currentHint && (
            <GradeBar grade={lastGrade} onUndo={lan ? undefined : () => void undo(lastGrade)} onClose={() => setLastGrade(null)} />
          )}
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
          <div key={`deal-${view.history.length}`} className={`hand${view.hand.length > 10 ? " crowded" : ""}`} aria-label="Your hand">
            {view.hand.map((card, i) => (
              <Card
                key={card.id}
                fan={{ index: i, count: view.hand.length }}
                card={card}
                playable={legalCards.has(card.id)}
                selected={selected.includes(card.id)}
                fromKitty={kittyIds.has(card.id)}
                hinted={hintedAction?.card === card.id}
                onClick={legalCards.has(card.id) ? () => clickCard(card) : undefined}
              />
            ))}
          </div>
          {settings.tracker && !quizActive && view.phase === "playing" && (
            <CardTracker
              played={view.played}
              unseen={view.unseenBySuit}
              jokerUnseen={view.jokerUnseen}
              trump={view.contract && view.contract.strain !== "NT" ? view.contract.strain : null}
            />
          )}
        </TableView>
      ) : (
        <div className="loading">{lan ? "At the table…" : "Shuffling…"}</div>
      )}

      {result && view && (
        <Dialog title={`Hand ${view.history.filter((h) => !h.passedOut).length}`}>
          <Celebration hand={result} gameWinner={view.winner} />
          <HandResult hand={result} view={view} />
          <div className="actions">
            <button type="button" onClick={() => setReview(result)}>
              Review hand
            </button>
            {view.phase === "gameOver" ? (
              <button
                type="button"
                className="primary"
                onClick={() => {
                  setResult(null);
                  if (lan) netRef.current?.endGame();
                  else setShowNewGame(true);
                }}
              >
                New game
              </button>
            ) : (
              <button
                type="button"
                className="primary"
                onClick={() => {
                  setResult(null);
                  if (lan) netRef.current?.continue();
                }}
                autoFocus
              >
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
            coach={coach}
            save={coachSave()}
          />
        </Dialog>
      )}
      {showScores && view && (
        <Dialog title="Score pad" wide onClose={() => setShowScores(false)}>
          <ScorePad history={view.history} players={view.players} onReview={(h) => { setShowScores(false); setReview(h); }} />
        </Dialog>
      )}
      {showLearn && <Learn coach={coachRef.current} onClose={() => setShowLearn(false)} />}
      {showCoach && <CoachDialog settings={settings} onChange={changeSettings} onClose={() => setShowCoach(false)} />}
      {lobbyOpen && host && netRef.current && (
        <Lobby
          host={host}
          lobby={lobby}
          net={netRef.current}
          connected={connected}
          name={name}
          onName={(n) => {
            setName(n);
            netRef.current?.rename(n);
          }}
          onSolo={() => {
            soloOnHost.set(true);
            setShowLobby(false);
            setLobby(null);
            void startSolo();
          }}
          onClose={lobby?.playing && lobby.you !== null && view ? () => setShowLobby(false) : undefined}
        />
      )}
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
