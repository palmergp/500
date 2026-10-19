import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
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
import { MainMenu } from "./components/MainMenu";
import { recordGrades } from "./progress";
import { audio } from "./audio";
import { SettingsDialog } from "./components/SettingsDialog";
import { Dialog } from "./components/Dialog";
import { HandReview } from "./components/HandReview";
import { NewGameDialog } from "./components/NewGameDialog";
import { ScorePad } from "./components/ScorePad";
import { InfoPanel, TableView } from "./components/TableView";
import { TricksDialog } from "./components/TricksDialog";
import { DecisionsDialog } from "./components/DecisionsDialog";
import { keepMistake } from "./mistakes";
import { loadSettings, savedGame, savedGrades, saveSettings, SPEED_FACTOR, type Settings } from "./settings";

const DELAY = { bidding: 700, discarding: 300, playing: 650, gameOver: 0 };
const TRICK_HOLD = 1300;
/** Wide enough for a side rail beside the table (the auction, the bid and the coach). */
const WIDE = "(min-width: 1100px)";

/** Identifies a game for the progress record: its seed. */
function gameId(save: string): string {
  try {
    return String((JSON.parse(save) as { seed: number }).seed);
  } catch {
    return "unknown";
  }
}

/** Which game this tab was in, so a reload goes straight back to it instead of the menu. */
const tabGame = {
  get: (): "solo" | "lan" | null => {
    try {
      const value = sessionStorage.getItem("fh500.playing");
      return value === "solo" || value === "lan" ? value : null;
    } catch {
      return null;
    }
  },
  set: (value: "solo" | "lan" | null) => {
    try {
      if (value) sessionStorage.setItem("fh500.playing", value);
      else sessionStorage.removeItem("fh500.playing");
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

function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => window.matchMedia(query).matches);
  useEffect(() => {
    const list = window.matchMedia(query);
    const update = () => setMatches(list.matches);
    list.addEventListener("change", update);
    update();
    return () => list.removeEventListener("change", update);
  }, [query]);
  return matches;
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

/** A top-bar button: an icon, and a label that hides on small screens. The tooltip says what it does. */
function NavButton({ icon, label, tip, onClick, disabled, className }: { icon: string; label: string; tip: string; onClick: () => void; disabled?: boolean; className?: string }) {
  return (
    <button type="button" className={`nav-button ${className ?? ""}`} title={tip} aria-label={label} onClick={onClick} disabled={disabled}>
      <span aria-hidden="true">{icon}</span>
      <span className="nav-label">{label}</span>
    </button>
  );
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
  const [showSettings, setShowSettings] = useState(false);
  const [sweeping, setSweeping] = useState(false);
  const [quizzed, setQuizzed] = useState<number | null>(null);
  const [showLearn, setShowLearn] = useState(false);
  const [showTricks, setShowTricks] = useState<{ focus?: number } | null>(null);
  const [showDecisions, setShowDecisions] = useState(false);
  /** Grades the coach is still working out. */
  const [grading, setGrading] = useState(0);
  /** Action numbers you asked a hint for: their grades don't count toward your patterns. */
  const hintedRef = useRef(new Set<number>());
  const [announcement, setAnnouncement] = useState("");
  /** The main menu, or the table. */
  const [screen, setScreen] = useState<"starting" | "menu" | "table">("starting");
  // Playing at a table on a LAN host (crates/host) rather than against bots on this device.
  const [mode, setMode] = useState<"none" | "solo" | "lan">("none");
  const modeRef = useRef(mode);
  const netRef = useRef<NetClient | null>(null);
  const gameRef = useRef<string | null>(null);
  const gradesRef = useRef(grades);
  const [host, setHost] = useState<HostInfo | null>(null);
  const [lobby, setLobby] = useState<LobbyState | null>(null);
  const [connected, setConnected] = useState(true);
  const [name, setName] = useState(savedName);
  const [showLobby, setShowLobby] = useState(false);
  const wide = useMediaQuery(WIDE);
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

  const clearTable = useCallback(() => {
    setView(null);
    setHeld(null);
    setResult(null);
    setReview(null);
    setHint(null);
    setLastGrade(null);
    setSelected([]);
    setJokerLeads(null);
    setShowTricks(null);
    setShowDecisions(false);
    setMessage("");
  }, []);

  /** Leaves the host's table (your seat waits for you) and plays against bots on this device. */
  const toSolo = useCallback(() => {
    netRef.current?.close();
    netRef.current = null;
    setLobby(null);
    setShowLobby(false);
    if (modeRef.current !== "solo") clearTable();
    modeRef.current = "solo";
    setMode("solo");
    tabGame.set("solo");
  }, [clearTable]);

  const startGame = useCallback(
    (chosen: Settings) => {
      saveSettings(chosen);
      setSettings(chosen);
      toSolo();
      savedGrades.clear();
      setGrades([]);
      hintedRef.current.clear();
      clearTable();
      setShowNewGame(false);
      setScreen("table");
      // "any" seats are filled at random by the engine.
      void request((engine) => engine.create(randomSeed(), chosen)).then(() =>
        setMessage(chosen.hands ? `New game: ${chosen.hands} hands, highest score wins.` : "New game. First to 500 wins."),
      );
    },
    [request, toSolo, clearTable],
  );

  /** Picks up the game saved on this device, or offers a new one. */
  const continueSolo = useCallback(async () => {
    setScreen("table");
    if (modeRef.current === "solo" && view) return;
    toSolo();
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
  }, [handle, toSolo, view]);

  /** At the LAN host's table. */
  const joinHost = useCallback(() => {
    setScreen("table");
    tabGame.set("lan");
    if (modeRef.current === "lan" && netRef.current) return;
    modeRef.current = "lan";
    setMode("lan");
    clearTable();
    setGrades([]);
    setShowNewGame(false);
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

  // Start the engine, then go back to the game this tab was in (after a reload), or the menu.
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
        const last = tabGame.get();
        if (last === "lan" && info) joinHost();
        else if (last === "solo") await continueSolo();
        else setScreen("menu");
      })
      .catch((e: unknown) => setError(String(e)));
    // Once, on start.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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

  const atTable = screen === "table";
  const lobbyOpen = atTable && lan && (showLobby || !lobby?.playing || lobby.you === null || !view);
  // At a LAN table, nobody moves until everyone has seen the last hand's result.
  const waitingOnOthers = lan && !!lobby && lobby.waiting.length > 0;
  const paused =
    !atTable || !!held || !!result || !!review || showScores || showNewGame || showSettings || showLearn || !!showTricks || showDecisions || lobbyOpen || waitingOnOthers;

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

  const keepGrade = (save: string, found: GradeView | null, numbers: number[]) => {
    if (!found) return;
    const g: GradeView = { ...found, hinted: numbers.some((n) => hintedRef.current.has(n)) };
    // Your mistakes become puzzles (solo games only: a LAN host keeps the save to itself).
    if (modeRef.current === "solo") keepMistake(g, save);
    setGrades((all) => {
      const next = [...all.filter((x) => x.actionNumber !== g.actionNumber), g].sort((a, b) => a.actionNumber - b.actionNumber);
      if (modeRef.current === "solo") savedGrades.store(next);
      recordGrades(gameRef.current ?? gameId(save), next);
      return next;
    });
    setLastGrade(g);
  };

  /** Asks the coach to grade your actions `numbers` (positions in the game's action log). */
  /** Runs one grading request, counting it while it's out. */
  const grade = (run: () => Promise<GradeView | null>, save: string, numbers: number[]) => {
    setGrading((n) => n + 1);
    run()
      .then((g) => keepGrade(save, g, numbers))
      .catch(() => {})
      .finally(() => setGrading((n) => n - 1));
  };

  const gradeLater = (save: string, numbers: number[]) => {
    if (!coach || !settings.grade) return;
    for (const number of numbers) grade(() => coach.grade(save, number), save, [number]);
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
        hintedRef.current.add(forAction);
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
    if (next.music !== settings.music) {
      audio.unlock();
      audio.setMusic(next.music);
    }
    setSettings(next);
    saveSettings(next);
  };

  const quizActive = !!view && atTable && settings.quiz && !paused && quizzed !== view.actionCount && quizDue(view);
  const currentHint = hint && view && hint.forAction === view.actionCount && view.humanToAct ? hint.hint : null;
  const hintedAction = currentHint ? view?.legal[currentHint.best] : undefined;
  /** Cards the coach rates about as good as its choice (a close call). */
  const alsoGood = new Set(currentHint && view ? currentHint.close.map((i) => view.legal[i]?.card).filter((c) => c != null) : []);

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
    if (!view) return;
    const chosen = selected;
    const numbers: number[] = [];
    const wholeKitty = (view.discards?.length ?? 0) === 0;
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
    }).then((snapshot) => {
      if (!snapshot || !coach || !settings.grade) return;
      // The three cards are graded together, as one decision.
      if (wholeKitty && numbers.length === 3) grade(() => coach.gradeDiscards(snapshot.save, numbers[0]), snapshot.save, numbers);
      else gradeLater(snapshot.save, numbers);
    });
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

  const trick = held ?? view?.trick ?? null;
  const openTricks = (focus?: number) => setShowTricks({ focus });
  const deal = view ? view.history.length + 1 : 0;
  const dealGrades = grades.filter((g) => g.deal === deal).length;
  const openDecisions = settings.grade ? () => setShowDecisions(true) : undefined;
  const trumpsUnseen = view && settings.trumpCounter && !quizActive ? view.trumpsUnseen : null;

  // Coach panels: in the side rail on wide screens, above the middle of the table otherwise.
  let coachPanels: ReactNode = null;
  if (view && atTable) {
    const tracker = settings.tracker && !quizActive && view.phase === "playing";
    const grade = lastGrade && !currentHint;
    if (quizActive || currentHint || grade || tracker) {
      coachPanels = (
        <>
          {quizActive && <CountingQuiz key={view.actionCount} view={view} onDone={() => setQuizzed(view.actionCount)} />}
          {currentHint && <HintPanel hint={currentHint} onClose={() => setHint(null)} />}
          {grade && <GradeBar grade={lastGrade} onUndo={lan ? undefined : () => void undo(lastGrade)} onClose={() => setLastGrade(null)} />}
          {tracker && (
            <CardTracker
              played={view.played}
              unseen={view.unseenBySuit}
              jokerUnseen={view.jokerUnseen}
              trump={view.contract && view.contract.strain !== "NT" ? view.contract.strain : null}
            />
          )}
        </>
      );
    }
  }

  // In the middle of the table, in place of the trick: whatever you need to act.
  let middle: ReactNode = undefined;
  if (view && view.humanToAct && !paused && view.phase === "bidding") {
    middle = <BiddingBox legal={view.legal} mustBid={view.mustBid} onAct={act} hinted={hintedAction?.index} />;
  } else if (view && jokerLeads) {
    middle = (
      <div className="joker-choice">
        Lead the Joker and name a suit:
        <div>
          {jokerLeads.map((a) => (
            <button key={a.index} type="button" className={`call ${a.namedSuit === "D" || a.namedSuit === "H" ? "red" : ""}`} onClick={() => act(a.index)}>
              {SUIT_SYMBOL[a.namedSuit!]}
            </button>
          ))}
          <button type="button" onClick={() => setJokerLeads(null)}>
            Cancel
          </button>
        </div>
      </div>
    );
  } else if (view && view.phase === "discarding" && view.humanToAct) {
    middle = (
      <div className="discard-bar">
        <p>Pick {discardsLeft} cards from your hand to put back.</p>
        <button type="button" className="primary" disabled={selected.length !== discardsLeft} onClick={confirmDiscards}>
          Discard {selected.length}/{discardsLeft}
        </button>
      </div>
    );
  }

  return (
    <div className={`app${settings.fourColor ? " four-color" : ""}${settings.bigCards ? " big-cards" : ""}${atTable ? " at-table" : ""}`}>
      <div className="visually-hidden" aria-live="polite" aria-atomic="true">
        {announcement}
      </div>

      {screen === "menu" && (
        <MainMenu
          canContinue={(mode === "solo" && !!view) || !!savedGame.load()}
          hostAvailable={!!host}
          atTable={lan}
          onContinue={() => void continueSolo()}
          onSolo={() => setShowNewGame(true)}
          onFriends={joinHost}
          onLearn={() => setShowLearn(true)}
          onSettings={() => setShowSettings(true)}
        />
      )}
      {screen === "starting" && <div className="loading">Shuffling…</div>}

      {atTable && (
        <>
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
              <NavButton icon="☰" label="Menu" tip="Main menu: play solo or with friends, learn, settings. Your game is kept." onClick={() => setScreen("menu")} />
              <NavButton
                icon="🎓"
                label="Learn"
                className="learn-button"
                tip="Learn: lessons on bidding, play and counting, a bidding trainer and play puzzles. Your game waits."
                onClick={() => setShowLearn(true)}
              />
              <NavButton icon="📋" label="Score pad" tip="Score pad: every hand so far, and a review of each one" onClick={() => setShowScores(true)} disabled={!view} />
              {lan && <NavButton icon="👥" label="Table" tip="Who's sitting where; leave or end the game" onClick={() => setShowLobby(true)} />}
              <NavButton icon="⚙" label="Settings" tip="Settings: coach, counting aids, sound, speed and display" onClick={() => setShowSettings(true)} />
              <button
                type="button"
                className="icon-toggle"
                aria-pressed={settings.sound}
                title={settings.sound ? "Sound on (click to mute)" : "Sound off (click to turn on)"}
                aria-label="Sound effects"
                onClick={() => changeSettings({ ...settings, sound: !settings.sound })}
              >
                {settings.sound ? "🔊" : "🔇"}
              </button>
              <button
                type="button"
                className={`icon-toggle${settings.music ? "" : " off"}`}
                aria-pressed={settings.music}
                title={settings.music ? "Music on (click to stop)" : "Music off (click to play)"}
                aria-label="Music"
                onClick={() => changeSettings({ ...settings, music: !settings.music })}
              >
                🎵
              </button>
            </nav>
          </header>

          <main className="play-area">
            {view ? (
              <>
                <TableView
                  view={view}
                  trick={trick}
                  sweepTo={held && sweeping ? held.winner : null}
                  trumpsUnseen={trumpsUnseen}
                  showVoids={settings.voids && !quizActive}
                  holding={!!held}
                  onTricks={openTricks}
                  onDecisions={openDecisions}
                  decisions={dealGrades}
                  infoInCorner={!wide}
                  middle={middle}
                  dock={wide ? undefined : (coachPanels ?? undefined)}
                  status={
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
                        <button
                          type="button"
                          className="hint-button"
                          onClick={askHint}
                          disabled={hintLoading || !!currentHint}
                          title="Ask the coach what it would do here, and why"
                        >
                          {hintLoading ? "Thinking…" : "Hint"}
                        </button>
                      )}
                    </div>
                  }
                >
                  <div key={`deal-${view.history.length}`} className={`hand${view.hand.length > 10 ? " crowded" : ""}`} aria-label="Your hand">
                    {view.hand.map((card, i) => (
                      <Card
                        key={card.id}
                        fan={{ index: i, count: view.hand.length }}
                        card={card}
                        playable={legalCards.has(card.id)}
                        selected={selected.includes(card.id)}
                        fromKitty={kittyIds.has(card.id)}
                        hinted={hintedAction?.card === card.id || (alsoGood.has(card.id) ? "close" : false)}
                        onClick={legalCards.has(card.id) ? () => clickCard(card) : undefined}
                      />
                    ))}
                  </div>
                </TableView>
                {wide && (
                  <aside className="rail" aria-label="Bidding and coach">
                    <InfoPanel view={view} trumpsUnseen={trumpsUnseen} onTricks={() => openTricks()} decisions={dealGrades} onDecisions={openDecisions} />
                    {coachPanels}
                  </aside>
                )}
              </>
            ) : (
              <div className="loading">{lan ? "At the table…" : "Shuffling…"}</div>
            )}
          </main>
        </>
      )}

      {result && view && atTable && (
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
      {showTricks && view && (
        <TricksDialog tricks={view.tricks} players={view.players} focus={showTricks.focus} onClose={() => setShowTricks(null)} />
      )}
      {showDecisions && view && <DecisionsDialog grades={grades} deal={deal} pending={grading} onClose={() => setShowDecisions(false)} />}
      {showScores && view && (
        <Dialog title="Score pad" wide onClose={() => setShowScores(false)}>
          <ScorePad history={view.history} players={view.players} onReview={(h) => { setShowScores(false); setReview(h); }} />
        </Dialog>
      )}
      {showLearn && <Learn coach={coachRef.current} onClose={() => setShowLearn(false)} />}
      {showSettings && <SettingsDialog settings={settings} onChange={changeSettings} onClose={() => setShowSettings(false)} />}
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
            toSolo();
            setScreen("menu");
            tabGame.set(null);
          }}
          onClose={lobby?.playing && lobby.you !== null && view ? () => setShowLobby(false) : undefined}
        />
      )}
      {showNewGame && personas.length > 0 && (
        <NewGameDialog
          personas={personas}
          initial={settings}
          onStart={startGame}
          onCancel={() => {
            setShowNewGame(false);
            if (!view || mode !== "solo") setScreen("menu");
          }}
        />
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
  const short = view.handLimit !== null;
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
      {short && won === null && (
        <p className="note">
          {view.handsPlayed < view.handLimit!
            ? `${view.handLimit! - view.handsPlayed} of ${view.handLimit} hands to go.`
            : "Tied after the last hand: one more hand to settle it."}
        </p>
      )}
      {won !== null && (
        <p className="game-over">
          {won === 0 ? "You and your partner win the game!" : `${view.players[1].name} and ${view.players[3].name} win the game.`}
          {short && ` (${view.handsPlayed} hands)`}
        </p>
      )}
    </div>
  );
}
