import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { ActionView, CardView, Coach, GradeView, HandView, HintView, Persona, Snapshot, TrickView, View } from "./engine";
import { CoachClient, Engine, setHostEngine, SUIT_SYMBOL, Simulator } from "./engine";
import { detectHost, NetClient, savedName, type HostInfo, type Lobby as LobbyState } from "./net";
import { Lobby } from "./components/Lobby";
import { BiddingBox } from "./components/BiddingBox";
import { Card } from "./components/Card";
import { CardTracker, GradeBar, HintPanel } from "./components/Coach";
import { Celebration } from "./components/Celebration";
import { CountingQuiz, quizDue } from "./components/CountingQuiz";
import { Learn, type LearnTab } from "./components/Learn";
import { MainMenu } from "./components/MainMenu";
import { recordGrades } from "./progress";
import { audio } from "./audio";
import { SettingsDialog } from "./components/SettingsDialog";
import { Dialog } from "./components/Dialog";
import { HandReview } from "./components/HandReview";
import { NewGameDialog } from "./components/NewGameDialog";
import { ScorePad } from "./components/ScorePad";
import { InfoPanel, TableView } from "./components/TableView";
import { BidText } from "./components/BidText";
import { TricksDialog } from "./components/TricksDialog";
import { DecisionsDialog } from "./components/DecisionsDialog";
import { keepMistake } from "./mistakes";
import { recordGame, recordHand, recordTournament } from "./records";
import { achievements, checkAchievements, noteGame, noteGrades, noteHand, noteTournament } from "./achievements";
import { handCode, saveAt } from "./handcode";
import { FinalReveal, NewTournamentDialog, RecapDialog, TournamentDialog } from "./components/Tournament";
import { HAND_PUZZLES, loadHandPuzzles, noteHandPuzzle, type HandPuzzle } from "./learn/handPuzzles";
import { TouchTips } from "./components/TouchTips";
import { MoreItem, MoreMenu } from "./components/MoreMenu";
import { gradeGame, HANDS, loadTournament, newTournament, recapGrade, ROUNDS, saveTournament, standings, yourTable, type Tournament, type TournamentKind } from "./tournament";
import { banterFor, fill, loadCareer, nextChapter, saveCareer, type CareerState } from "./career/career";
import { CHAPTERS, CHARACTERS, goal, type Step } from "./career/story";
import { CareerDialog } from "./components/Career";
import { Cutscene, Portrait } from "./components/Cutscene";
import { addPlayTime, IDLE_MS, type PlayMode } from "./playtime";
import { assistHelpers, loadSettings, savedGame, savedGrades, saveSettings, SPEED_FACTOR, type Settings, type Speed } from "./settings";

const DELAY = { bidding: 700, discarding: 300, playing: 650, gameOver: 0 };
const TRICK_HOLD = 1300;
/** Pace while finishing a hand: every pause cut to a tenth. */
const FINISH_FACTOR = 0.1;
const SPEEDS: Speed[] = ["slow", "normal", "fast"];
const SPEED_ICON: Record<Speed, string> = { slow: "🐢", normal: "▶", fast: "⏩" };
/** Wide enough for a side rail beside the table (the auction, the bid and the coach). */
const WIDE = "(min-width: 1100px)";
/** Screens where the top bar's labels hide (styles.css): too narrow for every button, so the less
 * used ones go in the "More" menu. */
const COMPACT = "(max-width: 760px), (max-height: 500px)";

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

/** The seed a save was dealt from. */
function seedOf(save: string | null): number | null {
  try {
    return save ? (JSON.parse(save) as { seed: number }).seed : null;
  } catch {
    return null;
  }
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

/**
 * The table as the hand just played ended: every card played, the last trick on the table. The
 * engine deals the next hand in the same step that scores this one, so until you move on from the
 * result the table shows the finished hand, not the new cards.
 */
function finishedHand(view: View, hand: HandView): View {
  return {
    ...view,
    phase: "playing",
    toAct: null,
    humanToAct: false,
    dealer: hand.dealer,
    dealNumber: hand.dealNumber,
    dealerForced: false,
    mustBid: false,
    hand: [],
    handCounts: [0, 0, 0, 0],
    calls: hand.calls,
    contract: hand.contract,
    kitty: null,
    discards: null,
    trick: null,
    lastTrick: hand.tricks.at(-1) ?? null,
    tricks: hand.tricks,
    tricksWon: hand.tricksWon,
    // This hand is still being played, as far as the table shows: it isn't among those played yet.
    handsPlayed: view.history.filter((h) => !h.passedOut && h.number < hand.number).length,
    legal: [],
    played: hand.tricks.flatMap((t) => t.plays.map((p) => p.card)),
    trumpsUnseen: null,
  };
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

const CODE_TIP = "A code for this exact moment of the game, copied for you: give it to Claude with a question";
const SETTINGS_TIP = "Settings: coach, counting aids, sound, speed and display";

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
  /** Playing out the rest of the hand quickly, a bot playing your cards (they aren't graded). */
  const [finishing, setFinishing] = useState(false);
  /** Replaying a hand from a point in its review: the real game's save to come back to. Nothing
   * in a replay counts (no saving, grades, records or progress). */
  const replayRef = useRef<{ resume: string } | null>(null);
  const [replaying, setReplaying] = useState<string | null>(null);
  /** A full-hand puzzle (Learn → Full hands) on the table, and what to go back to afterwards: the
   * real game's save (if it was on the table), its grades and hints, and where you were. Nothing
   * in a puzzle is saved or counted, but your decisions are graded (shown once it's over). */
  const puzzleRef = useRef<{ puzzle: HandPuzzle; resume: string | null; grades: GradeView[]; hinted: Set<number>; inTour: boolean; screen: "menu" | "table" } | null>(null);
  const [puzzle, setPuzzle] = useState<HandPuzzle | null>(null);
  /** The Learn tab to open on. */
  const [learnTab, setLearnTab] = useState<LearnTab>("lessons");
  /** The tournament (saved in this browser), whether the game on the table is its current round,
   * its dialog (`after`: the round just played) and the final reveal. */
  const [tour, setTour] = useState<Tournament | null>(loadTournament);
  const tourRef = useRef(tour);
  const [inTour, setInTour] = useState(false);
  const [tourDialog, setTourDialog] = useState<{ after?: number } | null>(null);
  const [showFinal, setShowFinal] = useState(false);
  /** The deal (by the number of deals before it) whose finished auction you've continued from. */
  const [seenAuction, setSeenAuction] = useState<number | null>(null);
  /** Choosing a new tournament's kind. */
  const [showNewTour, setShowNewTour] = useState(false);
  const simRef = useRef<Simulator | null>(null);
  const simsRunning = useRef(new Set<string>());
  /** A hand code being shown (and copied). */
  const [codeShown, setCodeShown] = useState<string | null>(null);
  /** A record you just set ("highest bid made: 9♥"), shown for a moment at the top. */
  const [recordToast, setRecordToast] = useState<string | null>(null);
  /** Achievements just done ("Slam · Hot streak"), shown the same way, wherever you are. */
  const [achievementToast, setAchievementToast] = useState<string | null>(null);
  const [quizzed, setQuizzed] = useState<number | null>(null);
  const [showLearn, setShowLearn] = useState(false);
  const [showTricks, setShowTricks] = useState<{ focus?: number } | null>(null);
  const [showDecisions, setShowDecisions] = useState(false);
  /** Grades the coach is still working out. */
  const [grading, setGrading] = useState(0);
  /** Action numbers you asked a hint for: their grades don't count toward your patterns. */
  const hintedRef = useRef(new Set<number>());
  const [announcement, setAnnouncement] = useState("");
  /** Career mode: your progress, whether the game on the table is a chapter's level, the chapter
   * list, and a story scene playing (with what follows it). */
  const [career, setCareer] = useState<CareerState>(loadCareer);
  const careerRef = useRef(career);
  const [careerGame, setCareerGame] = useState(false);
  const [showCareer, setShowCareer] = useState(false);
  const [scene, setScene] = useState<{ id: number; steps: Step[]; label: string; then: () => void } | null>(null);
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
  const compact = useMediaQuery(COMPACT);
  const lan = mode === "lan";
  useEffect(() => {
    gradesRef.current = grades;
  }, [grades]);

  const factor = finishing ? FINISH_FACTOR : SPEED_FACTOR[lan && lobby ? lobby.pace : settings.speed];
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
    const replay = !!replayRef.current || !!puzzleRef.current;
    if (modeRef.current === "solo" && !replay) savedGame.store(snapshot.save, next.phase === "gameOver");
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
        const broken = hand && gameRef.current && !replay ? recordHand(gameRef.current, hand) : [];
        if (broken.length) setRecordToast(broken.join(" · "));
        if (hand && gameRef.current && !replay) {
          noteHand(gameRef.current, hand);
          checkAchievements();
        }
        const solving = puzzleRef.current;
        if (solving && !replayRef.current && hand?.contract?.declarer === 0 && hand.made) noteHandPuzzle(solving.puzzle.id, hintedRef.current.size > 0);
      } else if (e.kind === "gameWon") {
        audio.play(next.winner === 0 ? "win" : "lose");
        if (!replay) recordGrades(gameRef.current, gradesRef.current, true, next.winner === 0);
        const broken = gameRef.current && !replay ? recordGame(gameRef.current, next.scores[0], next.winner === 0) : [];
        if (broken.length) setRecordToast((t) => [t, ...broken].filter(Boolean).join(" · "));
        if (gameRef.current && !replay) {
          noteGame(gameRef.current, next);
          checkAchievements();
        }
      }
    }
    // Screen readers hear every call, card, trick and score as it happens.
    const spoken = snapshot.events.filter((e) => e.kind !== "discarded").map((e) => e.text);
    if (spoken.length > 0) setAnnouncement(spoken.join(". "));
    const shown = snapshot.events.filter((e) => e.kind !== "discarded" && e.kind !== "dealt");
    if (shown.length > 0) setMessage(shown[shown.length - 1].text);
    const playing = puzzleRef.current?.puzzle;
    if (playing && !replayRef.current && next.phase === "gameOver") {
      const last = next.history.filter((h) => !h.passedOut).at(-1);
      setMessage(last?.made ? `${playing.contract} made.` : `${playing.contract} went down.`);
    }
    if (snapshot.events.some((e) => e.kind === "trickWon")) {
      setHeld(next.lastTrick ?? next.history.at(-1)?.tricks.at(-1) ?? null);
    }
    if (snapshot.events.some((e) => e.kind === "handScored")) {
      setFinishing(false);
      setResult(next.history.filter((h) => !h.passedOut).at(-1) ?? null);
    } else if (modeRef.current === "solo" && next.phase === "gameOver") {
      // A finished game picked up again (a reload, Continue, back from a replay): nothing is left
      // to play, so its last hand's result is the way on (New game, or the tournament's results).
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
    // A new game (or a host's table) ends any replay or puzzle: its game is gone, and so is the way back.
    replayRef.current = null;
    setReplaying(null);
    puzzleRef.current = null;
    setPuzzle(null);
    setFinishing(false);
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
      setInTour(false);
      setCareerGame(false);
      savedGrades.clear();
      setGrades([]);
      hintedRef.current.clear();
      clearTable();
      setShowNewGame(false);
      setScreen("table");
      // "any" seats are filled at random by the engine.
      void request((engine) => engine.create(randomSeed(), chosen)).then(() =>
        setMessage(chosen.hands ? `New game: ${chosen.hands} hands, highest score wins.` : "New game to 500: make a bid that takes you there to win. −500 loses."),
      );
    },
    [request, toSolo, clearTable],
  );

  /** Changes the tournament and keeps it saved. */
  const updateTour = useCallback((change: (t: Tournament) => void) => {
    const t = tourRef.current;
    if (!t) return;
    const next = structuredClone(t);
    change(next);
    tourRef.current = next;
    setTour(next);
    saveTournament(next);
  }, []);

  /** Plays out the other tables, in their own worker, one at a time: the current round's, and any
   * earlier round's still missing (a reload while they played, say after the last round). */
  const runSims = useCallback(() => {
    const t = tourRef.current;
    if (!t) return;
    simRef.current ??= new Simulator();
    t.rounds.slice(0, Math.min(t.current + 1, ROUNDS)).forEach((round, r) => round.tables.forEach((table, i) => {
      const key = `${t.id}:${r}:${i}`;
      if (table.result || table.seats.includes(0) || simsRunning.current.has(key)) return;
      simsRunning.current.add(key);
      const players = table.seats.map((id) => t.participants[id].persona) as [string, string, string, string];
      void simRef.current!.simulate(table.seed, players, HANDS)
        .then((result) => updateTour((next) => {
          if (next.id === t.id) next.rounds[r].tables[i].result = result;
        }))
        .catch(() => {})
        .finally(() => simsRunning.current.delete(key));
    }));
  }, [updateTour]);

  /** Deals your table's game for the tournament's current round (and starts the other tables). */
  const startTourRound = useCallback(() => {
    const t = tourRef.current;
    if (!t || t.current >= ROUNDS) return;
    const table = t.rounds[t.current].tables[yourTable(t, t.current)];
    const [, west, north, east] = table.seats.map((id) => t.participants[id]);
    toSolo();
    savedGrades.clear();
    setGrades([]);
    hintedRef.current.clear();
    clearTable();
    setShowNewGame(false);
    setTourDialog(null);
    setScreen("table");
    setInTour(true);
    setCareerGame(false);
    void request((engine) =>
      engine.create(table.seed, { west: west.persona, north: north.persona, east: east.persona, hands: HANDS, names: [west.name, north.name, east.name] }),
    ).then(() => setMessage(`Tournament round ${t.current + 1} of ${ROUNDS}: you partner ${north.name}.`));
    runSims();
  }, [request, toSolo, clearTable, runSims]);

  /** Your round is over: record your table's result, and show the tournament. */
  const finishTourRound = () => {
    const t = tourRef.current;
    if (!t || !view || t.current >= ROUNDS) return;
    const r = t.current;
    updateTour((next) => {
      next.rounds[r].tables[yourTable(next, r)].result = {
        scores: [view.scores[0], view.scores[1]],
        hands: view.history.filter((h) => !h.passedOut).map((h) => h.scoreChange),
        // You sit South, so the view's seats are the table's.
        bids: view.history
          .filter((h) => !h.passedOut && h.contract)
          .map((h) => ({ seat: h.contract!.declarer, label: h.contract!.label, made: !!h.made })),
      };
      // Your game, for your recap: the coach goes over whatever it didn't grade during play.
      const mine = next.mine ?? next.rounds.map(() => null);
      mine[r] = {
        save: saveRef.current ?? "",
        hands: view.history
          .filter((h) => !h.passedOut && h.contract)
          .map((h) => ({
            declarer: h.contract!.declarer,
            label: h.contract!.label,
            made: !!h.made,
            youBid: h.calls.some((c) => c.seat === 0 && c.kind === "bid"),
            points: h.scoreChange[0],
          })),
        grades: grades.map(recapGrade),
        graded: false,
      };
      next.mine = mine;
      next.current = r + 1;
    });
    setResult(null);
    setInTour(false);
    // After the last round, straight to the reveal: a recap first would give the results away.
    if (r === ROUNDS - 1) {
      setShowFinal(true);
      runSims();
    }
    else setTourDialog({ after: r });
  };

  /** From the menu: the tournament in progress, or a choice of new one. */
  const openTournament = () => {
    if (!tourRef.current || tourRef.current.finished) setShowNewTour(true);
    else setTourDialog({});
  };

  /** From a round's results to the main menu. The finished game goes: the tournament carries on
   * from the menu (Tournament), and Continue would only find a game with nothing left to play. */
  const leaveTourRound = () => {
    setTourDialog(null);
    savedGame.clear();
    savedGrades.clear();
    setGrades([]);
    clearTable();
    setScreen("menu");
    tabGame.set(null);
  };

  /** Starts a new tournament of `kind` (replacing any other) and shows it. */
  const beginTournament = (kind: TournamentKind) => {
    const t = newTournament(randomSeed(), kind);
    tourRef.current = t;
    setTour(t);
    saveTournament(t);
    setShowNewTour(false);
    setTourDialog({});
  };

  /** Ends the tournament for good, any time before its results. Its game goes too (if it's the one
   * saved or on the table); a solo game saved meanwhile stays. */
  const quitTournament = () => {
    const t = tourRef.current;
    const tourGame = t && t.current < ROUNDS ? t.rounds[t.current].tables[yourTable(t, t.current)].seed : null;
    const saved = savedGame.load();
    const ownSave = saved !== null && tourGame !== null && (JSON.parse(saved) as { seed: number }).seed === tourGame;
    tourRef.current = null;
    setTour(null);
    saveTournament(null);
    setTourDialog(null);
    if (inTour || ownSave) {
      leaveTourRound();
    }
    setInTour(false);
  };

  /** Turns every helper off (for an unaided tournament or story chapter). */
  const turnOffHelp = () => changeSettings({ ...settings, grade: false, trumpCounter: false, tracker: false, voids: false });

  /** You've had help in a tournament game: it can't count as unaided any more. */
  const markAssisted = useCallback(() => {
    if (tourRef.current?.assisted === false && !replayRef.current) {
      updateTour((t) => {
        t.assisted = true;
      });
    }
  }, [updateTour]);
  const helpers = assistHelpers(settings);
  const helped = helpers.length > 0;
  useEffect(() => {
    // Again when a replay ends: helpers turned on during one are still on in the real game.
    if (inTour && helped) markAssisted();
  }, [inTour, helped, replaying, markAssisted]);

  // Your recap: the coach goes over each of your finished tournament games in the background, a
  // game at a time, keeping each grade as it comes (so a reload carries on where it was). It waits
  // while you play a game it's grading live, so your grades there aren't held up.
  const [showRecap, setShowRecap] = useState(false);
  const [recapProgress, setRecapProgress] = useState<{ round: number; done: number; total: number } | null>(null);
  const recapBusy = useRef(false);
  const liveGrading = screen === "table" && mode === "solo" && settings.grade && !!view && view.phase !== "gameOver";
  useEffect(() => {
    const t = tour;
    const coach = coachRef.current;
    if (!t || !coach || recapBusy.current || liveGrading) return;
    const r = (t.mine ?? []).findIndex((g) => g && !g.graded);
    if (r < 0) return;
    recapBusy.current = true;
    const own = (next: Tournament) => (next.id === t.id ? next.mine?.[r] ?? null : null);
    gradeGame(coach, t.mine![r]!, (g, done, total) => {
      if (g) {
        updateTour((next) => {
          const game = own(next);
          if (game && !game.grades.some((x) => x.action === g.action)) game.grades.push(g);
        });
      }
      setRecapProgress({ round: r, done, total });
    })
      .catch(() => {})
      .finally(() => {
        // Done (or the coach couldn't read the game): either way, don't go round again.
        updateTour((next) => {
          const game = own(next);
          if (!game) return;
          game.grades.sort((a, b) => a.action - b.action);
          game.graded = true;
        });
        recapBusy.current = false;
        setRecapProgress(null);
      });
    // `personas` arrives once the coach's worker is up.
  }, [tour, liveGrading, updateTour, personas]);

  /** Play the tournament's current round: pick up its game if it's the one saved, else deal it. */
  const playTourRound = () => {
    const t = tourRef.current;
    if (!t) return;
    if (t.current >= ROUNDS) {
      setTourDialog(null);
      setShowFinal(true);
      runSims();
      return;
    }
    const save = savedGame.load();
    const seed = t.rounds[t.current].tables[yourTable(t, t.current)].seed;
    if (save && (JSON.parse(save) as { seed: number }).seed === seed) {
      setTourDialog(null);
      void continueSolo();
      return;
    }
    startTourRound();
  };

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
        setCareerGame(careerRef.current.playing?.seed === seedOf(save));
        const t = tourRef.current;
        if (t && t.current < ROUNDS && t.rounds[t.current].tables[yourTable(t, t.current)].seed === (JSON.parse(save) as { seed: number }).seed) {
          setInTour(true);
          runSims();
        }
        return;
      } catch {
        savedGame.clear();
      }
    }
    setShowNewGame(true);
  }, [handle, toSolo, view]);

  /** Changes your career and keeps it saved. */
  const updateCareer = useCallback((change: (c: CareerState) => void) => {
    const next = structuredClone(careerRef.current);
    change(next);
    careerRef.current = next;
    setCareer(next);
    saveCareer(next);
  }, []);

  /** You've had help in a story game: this try can't count as unaided. */
  const markCareerAssisted = useCallback(() => {
    const p = careerRef.current.playing;
    if (!p || p.assisted || replayRef.current || puzzleRef.current) return;
    updateCareer((c) => {
      if (c.playing) c.playing.assisted = true;
    });
  }, [updateCareer]);
  useEffect(() => {
    if (careerGame && helped) markCareerAssisted();
  }, [careerGame, helped, replaying, markCareerAssisted]);
  /** A hint, an undo or Finish hand: help, for an unaided tournament or story chapter. */
  const noteHelp = () => {
    if (inTour) markAssisted();
    if (careerGame) markCareerAssisted();
  };

  /** Plays a story scene, then whatever comes next. */
  const playScene = (steps: Step[], label: string, then: () => void) => setScene({ id: Date.now(), steps, label, then });

  /** A chapter's level: its game against the story's players, or its tournament. */
  const startLevel = (chapter: number) => {
    const level = CHAPTERS[chapter].level;
    const seed = randomSeed();
    if (level.kind === "tournament") {
      const cast = level.cast.map((c) => (typeof c === "string" ? { name: CHARACTERS[c].name, persona: CHARACTERS[c].persona } : { name: c[0], persona: c[1] }));
      const t = newTournament(seed, "random", cast, { chapter, title: level.title, goal: goal(level), place: level.place });
      tourRef.current = t;
      setTour(t);
      saveTournament(t);
      updateCareer((c) => {
        if (c.playing) Object.assign(c.playing, { tour: t.id, seed: null });
      });
      setScreen("menu");
      setTourDialog({});
      return;
    }
    updateCareer((c) => {
      if (c.playing) Object.assign(c.playing, { seed, tour: null, assisted: false });
    });
    const [west, north, east] = [level.west, level.partner, level.east].map((id) => CHARACTERS[id]);
    toSolo();
    setInTour(false);
    setCareerGame(true);
    savedGrades.clear();
    setGrades([]);
    hintedRef.current.clear();
    clearTable();
    setShowNewGame(false);
    setScreen("table");
    void request((engine) =>
      engine.create(seed, { west: west.persona, north: north.persona, east: east.persona, hands: level.hands ?? 0, names: [west.name, north.name, east.name] }),
    ).then(() => setMessage(`Chapter ${chapter + 1}: ${goal(level)} You partner ${north.name}.`));
  };

  /** From the chapter list: the chapter's opening scene and then its level (or, trying again, the level). */
  const startChapter = (chapter: number, skipIntro: boolean) => {
    const ch = CHAPTERS[chapter];
    const t = tourRef.current;
    if (ch.level.kind === "tournament" && t && !t.finished && !t.career && !confirm(`This chapter is a tournament: it replaces your tournament in progress (round ${t.current + 1} of ${ROUNDS}). Go on?`)) return;
    const save = savedGame.load();
    const ownSave = seedOf(save) === careerRef.current.playing?.seed;
    if (ch.level.kind === "game" && save && !savedGame.over() && !ownSave && !confirm("This chapter deals a new game: it replaces your game in progress. Go on?")) return;
    updateCareer((c) => {
      c.playing = { chapter, seed: null, tour: null, attempts: c.playing?.chapter === chapter ? c.playing.attempts : 0 };
    });
    setShowCareer(false);
    if (skipIntro) startLevel(chapter);
    else playScene(ch.intro, `Chapter ${chapter + 1}: ${ch.title}`, () => startLevel(chapter));
  };

  /** A chapter's closing scene (how it went), opening where the chapter was played unless it says otherwise. */
  const closingScene = (chapter: number, won: boolean): Step[] => {
    const ch = CHAPTERS[chapter];
    const steps = won ? ch.win : ch.lose;
    const setting = [...ch.intro].reverse().find((s) => "bg" in s);
    return setting && !("bg" in steps[0]) ? [setting, ...steps] : steps;
  };

  /** A level is over (`unaided`: won with no help at all): the scene for how it went, then the
   * chapter list (the next chapter, or another try). */
  const endLevel = (chapter: number, won: boolean, unaided: boolean) => {
    const ch = CHAPTERS[chapter];
    setCareerGame(false);
    updateCareer((c) => {
      if (won) {
        if (!c.done.includes(chapter)) c.done.push(chapter);
        if (unaided && !c.unaided.includes(chapter)) c.unaided.push(chapter);
        if (c.playing?.chapter === chapter) c.playing = null;
      } else {
        const attempts = c.playing?.chapter === chapter ? c.playing.attempts + 1 : 1;
        c.playing = { chapter, seed: null, tour: null, attempts };
      }
    });
    playScene(closingScene(chapter, won), `Chapter ${chapter + 1}: ${ch.title}`, () => {
      setShowCareer(true);
      // After the scene, so its banner doesn't land on top of it.
      checkAchievements();
    });
  };

  /** From the chapter list: a finished chapter's opening or ending scene again. */
  const watchScene = (chapter: number, which: "opening" | "ending") => {
    const ch = CHAPTERS[chapter];
    setShowCareer(false);
    playScene(which === "opening" ? ch.intro : closingScene(chapter, true), `Chapter ${chapter + 1}: ${ch.title}`, () => setShowCareer(true));
  };

  /** "Continue the story" after a chapter's game. */
  const finishCareerGame = () => {
    const p = careerRef.current.playing;
    if (!view || !p) return;
    const won = view.winner === 0;
    const unaided = !p.assisted && assistHelpers(settings).length === 0;
    setResult(null);
    savedGame.clear();
    savedGrades.clear();
    setGrades([]);
    clearTable();
    setScreen("menu");
    tabGame.set(null);
    endLevel(p.chapter, won, unaided);
  };

  /** Back to the chapter under way: its game, or its tournament. */
  const resumeCareer = () => {
    const p = careerRef.current.playing;
    setShowCareer(false);
    if (!p) return;
    if (p.tour) setTourDialog({});
    else void continueSolo();
  };

  /** At the LAN host's table. */
  const joinHost = useCallback(() => {
    setScreen("table");
    tabGame.set("lan");
    if (modeRef.current === "lan" && netRef.current) return;
    modeRef.current = "lan";
    setMode("lan");
    setInTour(false);
    setCareerGame(false);
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
  // Loaded from a host, the host's computer does the engine's thinking (a phone's own is far
  // slower; `?local` keeps it on this device).
  const startedRef = useRef(false);
  useEffect(() => {
    if (startedRef.current) return;
    startedRef.current = true;
    detectHost()
      .then(async (info) => {
        setHostEngine(!!info && !new URLSearchParams(location.search).has("local"));
        const engine = new Engine();
        engineRef.current = engine;
        coachRef.current = new CoachClient();
        setPersonas(await engine.personas());
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

  // A new record shows for a few seconds, as a popover so it stays above the hand's result
  // (a modal dialog): raised again when the result opens.
  const toastRef = useRef<HTMLDivElement>(null);
  const resultOpen = !!result && !held;
  useEffect(() => {
    if (!recordToast) return;
    const timer = setTimeout(() => setRecordToast(null), 6000);
    return () => clearTimeout(timer);
  }, [recordToast]);
  useEffect(() => {
    if (!achievementToast) return;
    const timer = setTimeout(() => setAchievementToast(null), 6000);
    return () => clearTimeout(timer);
  }, [achievementToast]);
  // Achievements are checked wherever they happen (a drill in Learn too): they announce themselves.
  useEffect(() => {
    const announce = (e: Event) => {
      const titles = (e as CustomEvent<string[]>).detail;
      setAchievementToast((t) => [t, ...titles].filter(Boolean).join(" · "));
    };
    window.addEventListener("fh500:achievements", announce);
    // On start, quietly: a tournament finished before it could be counted, and anything already
    // reached (play from before achievements existed counts).
    const t = tourRef.current;
    if (t?.finished) {
      recordTournament(t);
      noteTournament(t);
    }
    achievements(true);
    return () => window.removeEventListener("fh500:achievements", announce);
  }, []);
  // Clean sheet: judged once the coach has graded every decision of a finished game.
  useEffect(() => {
    if (mode !== "solo" || replayRef.current || puzzleRef.current || view?.phase !== "gameOver" || grading > 0 || !gameRef.current) return;
    noteGrades(gameRef.current, grades);
    checkAchievements();
  }, [mode, view, grading, grades]);
  // Records show at the table; achievements anywhere.
  const showRecordToast = !!recordToast && screen === "table";
  useEffect(() => {
    const el = toastRef.current;
    if (!el || (!showRecordToast && !achievementToast)) return;
    try {
      if (el.matches(":popover-open")) el.hidePopover();
      el.showPopover();
    } catch {
      // No popover support: it shows in the page instead.
    }
  }, [recordToast, achievementToast, showRecordToast, resultOpen]);

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
  // "Pause after the bidding": the finished auction waits in the middle until you continue.
  const auctionPause =
    atTable && mode === "solo" && settings.pauseAfterBidding && !replaying && !puzzle && !!view && view.phase === "discarding" && seenAuction !== view.history.length;
  const paused =
    auctionPause || !atTable || !!held || !!result || !!review || showScores || showNewGame || showSettings || showLearn || !!showTricks || showDecisions || lobbyOpen || waitingOnOthers || !!tourDialog || showFinal || !!scene || showCareer;

  // Let the bots take their turns one at a time. Thinking time counts toward the pause between
  // moves, so a slow search doesn't add to it.
  useEffect(() => {
    if (mode !== "solo" || !view || paused || (view.humanToAct && !finishing) || view.phase === "gameOver") return;
    const pause = new Promise((r) => setTimeout(r, DELAY[view.phase] * factor));
    if (view.humanToAct) autoActed.current.add(`${gameRef.current}:${view.actionCount}`);
    void request(async (engine) => {
      const [snapshot] = await Promise.all([view.humanToAct ? engine.autoAct() : engine.stepBot(), pause]);
      return snapshot;
    });
  }, [mode, view, paused, factor, request, finishing]);
  /** Your actions that Finish hand made (The Natural played them), by game: never graded as yours. */
  const autoActed = useRef(new Set<string>());

  // Time played, by mode: counted every 10 seconds while the window is in view and there's been a
  // click, tap or key in the last few minutes (playtime.ts).
  const lastInput = useRef(Date.now());
  useEffect(() => {
    const mark = () => (lastInput.current = Date.now());
    window.addEventListener("pointerdown", mark);
    window.addEventListener("keydown", mark);
    return () => {
      window.removeEventListener("pointerdown", mark);
      window.removeEventListener("keydown", mark);
    };
  }, []);
  const playMode: PlayMode | null = scene
    ? "story"
    : showLearn
      ? "learn"
      : screen === "table" && view
        ? lan
          ? "friends"
          : puzzle
            ? "puzzles"
            : careerGame
              ? "story"
              : inTour
                ? "tournament"
                : "solo"
        : null;
  useEffect(() => {
    if (!playMode) return;
    let last = Date.now();
    const count = () => {
      const now = Date.now();
      if (document.visibilityState === "visible" && now - lastInput.current < IDLE_MS) addPlayTime(playMode, Math.min(now - last, 15_000) / 1000);
      last = now;
    };
    const timer = window.setInterval(count, 10_000);
    return () => {
      window.clearInterval(timer);
      count(); // the time since the last tick, as the mode changes
    };
  }, [playMode]);

  /** Keeps a grade: shown at the table as it comes (`quiet`: kept without showing, for a hand
   * graded after the fact in its review). */
  const keepGrade = (save: string, found: GradeView | null, numbers: number[], quiet = false) => {
    if (!found) return;
    const g: GradeView = { ...found, hinted: numbers.some((n) => hintedRef.current.has(n)) };
    // A puzzle's grades wait for the end (Review hand), and don't count anywhere else. A grade
    // from an earlier try (started again since) is dropped.
    if (puzzleRef.current) {
      const now = saveRef.current ? (JSON.parse(saveRef.current) as { log: number[] }).log : [];
      const then = (JSON.parse(save) as { log: number[] }).log;
      if (then.length <= now.length && then.every((a, i) => a === now[i])) {
        setGrades((all) => [...all.filter((x) => x.actionNumber !== g.actionNumber), g].sort((a, b) => a.actionNumber - b.actionNumber));
      }
      return;
    }
    // Your mistakes become puzzles (solo games only: a LAN host keeps the save to itself).
    if (modeRef.current === "solo") keepMistake(g, save);
    setGrades((all) => {
      const next = [...all.filter((x) => x.actionNumber !== g.actionNumber), g].sort((a, b) => a.actionNumber - b.actionNumber);
      if (modeRef.current === "solo") savedGrades.store(next);
      recordGrades(gameRef.current ?? gameId(save), next);
      return next;
    });
    if (!quiet) setLastGrade(g);
  };

  /** Asks the coach to grade your actions `numbers` (positions in the game's action log). */
  /** Runs one grading request, counting it while it's out. */
  const grade = (run: () => Promise<GradeView | null>, save: string, numbers: number[], quiet = false) => {
    setGrading((n) => n + 1);
    run()
      .then((g) => keepGrade(save, g, numbers, quiet))
      .catch(() => {})
      .finally(() => setGrading((n) => n - 1));
  };

  /** A finished hand's review with grading off: the coach grades your decisions in it now, after
   * the fact (so they don't count as help in an unaided tournament or story chapter). Each hand
   * once; Finish hand's plays are left out (they weren't yours). */
  const gradedAfter = useRef(new Set<string>());
  const gradeHandAfter = (hand: HandView, end: number) => {
    const save = saveRef.current;
    const client = coachRef.current;
    if (!client || !save || modeRef.current !== "solo" || replayRef.current || puzzleRef.current) return;
    const key = `${gameRef.current}:${hand.number}`;
    if (gradedAfter.current.has(key)) return;
    gradedAfter.current.add(key);
    void client
      .decisions(save)
      .then((list) => {
        const have = new Set(gradesRef.current.map((g) => g.actionNumber));
        for (const d of list) {
          if (d.number < hand.firstAction || d.number >= end || have.has(d.number) || autoActed.current.has(`${gameRef.current}:${d.number}`)) continue;
          grade(() => (d.discards ? client.gradeDiscards(save, d.number) : client.grade(save, d.number)), save, [d.number], true);
        }
      })
      .catch(() => gradedAfter.current.delete(key));
  };
  const reviewEnd = review && view ? (view.history.find((h) => h.number === review.number + 1)?.firstAction ?? view.actionCount) : 0;
  useEffect(() => {
    if (review && !settings.grade) gradeHandAfter(review, reviewEnd);
    // Once per review opened; the rest is read when it runs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [review]);

  const gradeLater = (save: string, numbers: number[]) => {
    if (!coach || replayRef.current || (!settings.grade && !puzzleRef.current)) return;
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

  // Card play: start the coach's analysis as soon as it's your turn, so a hint (and the grade after
  // your play, which reuses it) is ready sooner. Only when the coach would do the work anyway:
  // grading is on, or you've asked for a hint.
  const lookAhead = useRef<{ save: string; hint: Promise<HintView> } | null>(null);
  useEffect(() => {
    const save = saveRef.current;
    if (mode !== "solo" || !view?.humanToAct || view.phase !== "playing" || finishing || !save || !coachRef.current) return;
    if (!settings.grade && !puzzleRef.current && hintedRef.current.size === 0) return;
    if (lookAhead.current?.save === save) return;
    const hint = coachRef.current.hint(save);
    hint.catch(() => {});
    lookAhead.current = { save, hint };
  }, [mode, view, settings.grade, finishing]);

  /** A hand code for the game as it stood after action `at`, copied to the clipboard and shown. */
  const showCode = (at: number) => {
    const save = saveRef.current;
    if (!save || modeRef.current !== "solo") return;
    void handCode(save, at, !!host).then((code) => {
      setCodeShown(code);
      navigator.clipboard?.writeText(code).catch(() => {});
    });
  };

  /** Loads a save once the engine is free (a bot may be mid-move): a busy engine drops requests. */
  const loadWhenFree = async (save: string) => {
    while (busy.current) await new Promise((r) => setTimeout(r, 50));
    await request((engine) => engine.load(save));
  };

  /** Plays the hand again from action `at` of the game: a sandbox that doesn't count. */
  const startReplay = (at: number, label: string) => {
    const save = replayRef.current?.resume ?? saveRef.current;
    if (!save || modeRef.current !== "solo") return;
    replayRef.current = { resume: save };
    setReplaying(label);
    setReview(null);
    setResult(null);
    setShowScores(false);
    setShowDecisions(false);
    setHint(null);
    setLastGrade(null);
    setFinishing(false);
    void loadWhenFree(saveAt(save, at));
  };

  /** Plays a full-hand puzzle at the table (from Learn), keeping the way back to what was there. */
  const startPuzzle = (p: HandPuzzle) => {
    const back = puzzleRef.current ?? {
      resume: modeRef.current === "solo" && view ? (replayRef.current?.resume ?? saveRef.current) : null,
      grades: gradesRef.current,
      hinted: new Set(hintedRef.current),
      inTour,
      screen: screen === "table" ? ("table" as const) : ("menu" as const),
    };
    toSolo();
    clearTable();
    setGrades([]);
    hintedRef.current = new Set();
    setInTour(false);
    setShowLearn(false);
    setShowNewGame(false);
    setScreen("table");
    puzzleRef.current = { ...back, puzzle: p };
    setPuzzle(p);
    void loadWhenFree(p.save).then(() => setMessage(`Puzzle ${p.id}: make ${p.contract}. Take the kitty and put three cards back.`));
  };

  /** The next puzzle you haven't solved (after this one, round to the start), or the next one. */
  const nextPuzzle = (p: HandPuzzle): HandPuzzle => {
    const { solved } = loadHandPuzzles();
    const at = HAND_PUZZLES.findIndex((x) => x.id === p.id);
    const after = [...HAND_PUZZLES.slice(at + 1), ...HAND_PUZZLES.slice(0, at + 1)];
    return after.find((x) => !solved.includes(x.id)) ?? after[0];
  };

  /** Back from the puzzles to where you were, with the list open. */
  const leavePuzzle = async () => {
    const back = puzzleRef.current;
    if (!back) return;
    puzzleRef.current = null;
    setPuzzle(null);
    setResult(null);
    setReview(null);
    setHint(null);
    setLastGrade(null);
    setFinishing(false);
    setHeld(null);
    setGrades(back.grades);
    hintedRef.current = back.hinted;
    setInTour(back.inTour);
    if (back.resume) {
      await loadWhenFree(back.resume);
      setScreen(back.screen);
    } else {
      clearTable();
      tabGame.set(null);
      setScreen("menu");
    }
    setLearnTab("hands");
    setShowLearn(true);
  };

  /** Back from a replay to the real game, where you left it. */
  const endReplay = () => {
    const resume = replayRef.current?.resume;
    if (!resume) return;
    replayRef.current = null;
    setReplaying(null);
    setResult(null);
    setHint(null);
    setLastGrade(null);
    setFinishing(false);
    void loadWhenFree(resume);
  };

  const askHint = () => {
    const save = coachSave();
    if (!coach || !save || !view) return;
    const forAction = view.actionCount;
    setHintLoading(true);
    (lookAhead.current?.save === save ? lookAhead.current.hint : coach.hint(save))
      .then((h) => {
        hintedRef.current.add(forAction);
        noteHelp();
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
    noteHelp();
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

  const quizActive = !!view && atTable && settings.quiz && !paused && !finishing && quizzed !== view.actionCount && quizDue(view, settings.quizRate);
  const currentHint = hint && view && hint.forAction === view.actionCount && view.humanToAct ? hint.hint : null;
  const hintedAction = currentHint ? view?.legal[currentHint.best] : undefined;
  /** Cards the coach rates about as good as its choice (a close call). */
  /** After a hint in card play: the coach's numbers for each card, shown when you hover over it. */
  const cardTips = new Map<number, string>();
  const declaring = view?.contract ? view.contract.declarer % 2 === 0 : true;
  // Best first, so the Joker's lead (one option per named suit) shows its best.
  for (const p of [...(currentHint?.plays ?? [])].sort((a, b) => b.ev - a.ev)) {
    const id = view?.legal[p.index]?.card;
    if (id == null || cardTips.has(id)) continue;
    const chance =
      p.makes === null ? null : declaring ? `the bid makes ${Math.round(100 * p.makes)}%` : `their bid goes down ${Math.round(100 * (1 - p.makes))}%`;
    cardTips.set(id, `Coach: ${[chance, p.note].filter(Boolean).join(", ")}`);
  }
  const alsoGood = new Set(currentHint && view ? currentHint.close.map((i) => view.legal[i]?.card).filter((c) => c != null) : []);

  const legalCards = useMemo(() => {
    if (!view?.humanToAct || paused || finishing || (settings.quiz && view && quizzed !== view.actionCount && quizDue(view, settings.quizRate))) return new Set<number>();
    return new Set(view.legal.filter((a) => a.card !== null).map((a) => a.card!));
  }, [view, paused, finishing, settings.quiz, settings.quizRate, quizzed]);

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
  /** What the table shows: the finished hand while its result is up, else the game as it is. */
  const table = view && result ? finishedHand(view, result) : view;
  const openTricks = (focus?: number) => setShowTricks({ focus });
  const deal = view ? view.history.length + 1 : 0;
  const dealGrades = grades.filter((g) => g.deal === deal).length;
  // In a puzzle, your graded decisions wait until the hand is over.
  const openDecisions = (puzzle ? view?.phase === "gameOver" : settings.grade) ? () => setShowDecisions(true) : undefined;
  const trumpsUnseen = view && settings.trumpCounter && !quizActive ? view.trumpsUnseen : null;

  // Coach panels: in the side rail on wide screens, above the middle of the table otherwise.
  let coachPanels: ReactNode = null;
  if (view && atTable) {
    const tracker = settings.tracker && !quizActive && view.phase === "playing" && !result;
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
    const chances = currentHint?.bids ? new Map(currentHint.bids.map((b) => [b.index, b.makes ?? 0])) : undefined;
    middle = <BiddingBox legal={view.legal} mustBid={view.mustBid} onAct={act} hinted={hintedAction?.index} chances={chances} />;
  } else if (view && auctionPause && view.contract) {
    const c = view.contract;
    middle = (
      <div className="auction-summary" role="group" aria-label="The bidding">
        <div className="auction-title">The bidding</div>
        <ol>
          {view.calls.map((call, i) => (
            <li key={i} className="auction-row">
              <span>{view.players[call.seat].name}</span>
              <strong className={call.kind}>
                <BidText label={call.label} />
              </strong>
            </li>
          ))}
        </ol>
        <p>
          {c.declarer === 0 ? "You play" : `${view.players[c.declarer].name} plays`}{" "}
          <strong className="bid-big">
            <BidText label={c.label} />
          </strong>
        </p>
        <button type="button" className="primary" autoFocus onClick={() => setSeenAuction(view.history.length)}>
          Continue
        </button>
      </div>
    );
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

  // Career: the level on the table (for its lines after each hand), and where the story's up to.
  const careerLevel = careerGame && mode === "solo" && !puzzle && !replaying && career.playing ? CHAPTERS[career.playing.chapter].level : null;
  const careerLine = careerLevel?.kind === "game" && result ? banterFor(careerLevel.banter, result) : null;
  const careerAt = career.playing?.chapter ?? nextChapter(career);
  const careerLabel = !career.name
    ? `${CHAPTERS.length} chapters of cards, crime and betrayal`
    : careerAt >= CHAPTERS.length
      ? "The end… for now. Play any chapter again"
      : `Chapter ${careerAt + 1}: ${CHAPTERS[careerAt].title}`;
  const careerUnaided = career.unaided.length > 0 ? ` · 🙌 ${career.unaided.length} of ${CHAPTERS.length} unaided` : "";
  const careerResumable = (() => {
    const p = career.playing;
    if (!p) return false;
    if (p.tour) return !!tour && tour.id === p.tour && !tour.finished;
    return p.seed !== null && seedOf(savedGame.load()) === p.seed;
  })();

  return (
    <div className={`app${settings.fourColor ? " four-color" : ""}${settings.bigCards ? " big-cards" : ""}${atTable ? " at-table" : ""}`}>
      <div className="visually-hidden" aria-live="polite" aria-atomic="true">
        {announcement}
      </div>
      <TouchTips />

      {screen === "menu" && (
        <MainMenu
          canContinue={mode === "solo" && !!view ? view.phase !== "gameOver" : !!savedGame.load() && !savedGame.over()}
          hostAvailable={!!host}
          atTable={lan}
          onContinue={() => void continueSolo()}
          tournament={tour && !tour.finished ? { round: Math.min(tour.current + 1, ROUNDS), chapter: tour.career?.chapter } : null}
          onTournament={openTournament}
          career={careerLabel + careerUnaided}
          onCareer={() => setShowCareer(true)}
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
              {inTour && tour && (
                <NavButton icon="🏆" label="Standings" tip={`Tournament, round ${tour.current + 1} of ${ROUNDS}: everyone's totals so far`} onClick={() => setTourDialog({})} />
              )}
              {lan && <NavButton icon="👥" label="Table" tip="Who's sitting where; leave or end the game" onClick={() => setShowLobby(true)} />}
              {!lan && (
                <button
                  type="button"
                  className="icon-toggle speed-toggle"
                  aria-label={`Speed: ${settings.speed}`}
                  title={`Speed: ${settings.speed} (click for ${SPEEDS[(SPEEDS.indexOf(settings.speed) + 1) % SPEEDS.length]})`}
                  onClick={() => changeSettings({ ...settings, speed: SPEEDS[(SPEEDS.indexOf(settings.speed) + 1) % SPEEDS.length] })}
                >
                  {SPEED_ICON[settings.speed]}
                </button>
              )}
              {compact ? (
                <MoreMenu>
                  {mode === "solo" && view && <MoreItem icon="🔗" label="Code" tip={CODE_TIP} onClick={() => showCode(view.actionCount)} />}
                  <MoreItem icon="⚙" label="Settings" tip={SETTINGS_TIP} onClick={() => setShowSettings(true)} />
                  <MoreItem
                    icon={settings.sound ? "🔊" : "🔇"}
                    label="Sound effects"
                    state={settings.sound ? "on" : "off"}
                    off={!settings.sound}
                    tip={settings.sound ? "Sound on (tap to mute)" : "Sound off (tap to turn on)"}
                    onClick={() => changeSettings({ ...settings, sound: !settings.sound })}
                  />
                  <MoreItem
                    icon="🎵"
                    label="Music"
                    state={settings.music ? "on" : "off"}
                    off={!settings.music}
                    tip={settings.music ? "Music on (tap to stop)" : "Music off (tap to play)"}
                    onClick={() => changeSettings({ ...settings, music: !settings.music })}
                  />
                </MoreMenu>
              ) : (
                <>
                  {mode === "solo" && view && <NavButton icon="🔗" label="Code" tip={CODE_TIP} onClick={() => showCode(view.actionCount)} />}
                  <NavButton icon="⚙" label="Settings" tip={SETTINGS_TIP} onClick={() => setShowSettings(true)} />
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
                </>
              )}
            </nav>
          </header>

          <main className="play-area">
            {view ? (
              <>
                <TableView
                  view={table ?? view}
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
                      {puzzle && !replaying && (
                        <span className="replay-banner puzzle-banner">
                          Puzzle {puzzle.id}: make <strong>{puzzle.contract}</strong>.{" "}
                          <button type="button" onClick={() => startPuzzle(puzzle)}>
                            Start again
                          </button>{" "}
                          <button type="button" onClick={() => void leavePuzzle()}>
                            Leave
                          </button>
                        </span>
                      )}
                      {replaying && (
                        <span className="replay-banner">
                          Replay from {replaying}: it doesn't count.{" "}
                          <button type="button" onClick={endReplay}>
                            Back to the game
                          </button>
                        </span>
                      )}
                      <div className="status" role="status">
                        {lanStatus?.text || prompt(table ?? view) || message}
                      </div>
                      {lanStatus?.seat !== undefined && lobby && (
                        <button type="button" onClick={() => netRef.current?.setBot(lanStatus.seat!, lobby.seats[lanStatus.seat!].bot)}>
                          Let a bot play for them
                        </button>
                      )}
                      {mode === "solo" && !puzzle && view.phase === "playing" && !result && (
                        <button
                          type="button"
                          className="finish-button"
                          onClick={() => {
                            setHint(null);
                            if (!finishing) noteHelp();
                            setFinishing((f) => !f);
                          }}
                          title={
                            finishing
                              ? "Stop, and play the rest of your cards yourself"
                              : "Finish the hand: play it out quickly, with a bot playing your cards (those plays aren't graded)"
                          }
                        >
                          {finishing ? "Stop" : "Finish hand ⏭"}
                        </button>
                      )}
                      {view.humanToAct && !paused && !finishing && view.phase !== "gameOver" && (
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
                  <div key={result ? "finished" : `deal-${view.history.length}`} className={`hand${view.hand.length > 10 ? " crowded" : ""}`} aria-label="Your hand">
                    {(table ?? view).hand.map((card, i) => (
                      <Card
                        key={card.id}
                        fan={{ index: i, count: (table ?? view).hand.length }}
                        card={card}
                        playable={legalCards.has(card.id)}
                        selected={selected.includes(card.id)}
                        fromKitty={kittyIds.has(card.id)}
                        hinted={hintedAction?.card === card.id || (alsoGood.has(card.id) ? "close" : false)}
                        onClick={legalCards.has(card.id) ? () => clickCard(card) : undefined}
                        tip={cardTips.get(card.id)}
                      />
                    ))}
                  </div>
                </TableView>
                {wide && (
                  <aside className="rail" aria-label="Bidding and coach">
                    <InfoPanel view={table ?? view} trumpsUnseen={trumpsUnseen} onTricks={() => openTricks()} decisions={dealGrades} onDecisions={openDecisions} />
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

      {(showRecordToast || achievementToast) && (
        <div ref={toastRef} popover="manual" className="record-toast" role="status">
          {showRecordToast && (
            <span>
              <span aria-hidden="true">🏆</span> New record! {recordToast}
            </span>
          )}
          {achievementToast && (
            <span>
              <span aria-hidden="true">🏅</span> Achievement: {achievementToast}
            </span>
          )}
        </div>
      )}
      {result && !held && view && atTable && (
        <Dialog title={puzzle && !replaying ? `Puzzle ${puzzle.id}` : `Hand ${view.history.filter((h) => !h.passedOut).length}`}>
          <Celebration hand={result} gameWinner={puzzle ? null : view.winner} />
          {careerLine && (
            <div className="career-banter">
              <Portrait look={CHARACTERS[careerLine.who].look} />
              <p>
                <strong>{CHARACTERS[careerLine.who].name}</strong> “{fill(careerLine.text, career.name)}”
              </p>
            </div>
          )}
          {puzzle && !replaying && (
            <p className={`puzzle-verdict ${result.made ? "made" : "down"}`}>
              {result.made
                ? hintedRef.current.size > 0
                  ? `Made ${puzzle.contract}, with a hint along the way: try it without one to mark it solved.`
                  : `Solved: ${puzzle.contract} made.`
                : `${puzzle.contract} went down. Review the hand to see where the coach would have played differently, then try again.`}
            </p>
          )}
          <HandResult hand={result} view={view} puzzle={!!puzzle && !replaying} />
          <div className="actions">
            <button type="button" onClick={() => setReview(result)}>
              Review hand
            </button>
            {puzzle && !replaying ? (
              <>
                <button type="button" onClick={() => void leavePuzzle()}>
                  All puzzles
                </button>
                <button type="button" className={result.made ? "" : "primary"} onClick={() => startPuzzle(puzzle)} autoFocus={!result.made}>
                  Try again
                </button>
                <button type="button" className={result.made ? "primary" : ""} onClick={() => startPuzzle(nextPuzzle(puzzle))} autoFocus={!!result.made}>
                  Next puzzle
                </button>
              </>
            ) : replaying ? (
              // A replay always leads back to the real game, even when it reaches the game's end.
              <button type="button" className="primary" onClick={endReplay} autoFocus>
                Back to the game
              </button>
            ) : careerLevel && view.phase === "gameOver" ? (
              <button type="button" className="primary" onClick={finishCareerGame} autoFocus>
                Continue the story
              </button>
            ) : inTour && view.phase === "gameOver" ? (
              <button type="button" className="primary" onClick={finishTourRound} autoFocus>
                {tour && tour.current === ROUNDS - 1 ? "Final results" : "Round results"}
              </button>
            ) : view.phase === "gameOver" ? (
              <>
                <button type="button" onClick={() => {
                  setResult(null);
                  setScreen("menu");
                }}>
                  Main menu
                </button>
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
              </>
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
            pending={grading}
            afterwards={mode === "solo" && !settings.grade}
            coach={coach}
            save={coachSave()}
            onReplay={mode === "solo" && !puzzle ? startReplay : undefined}
            onCode={mode === "solo" ? showCode : undefined}
            end={reviewEnd}
          />
        </Dialog>
      )}
      {showTricks && view && (
        <TricksDialog tricks={view.tricks} players={view.players} focus={showTricks.focus} onClose={() => setShowTricks(null)} />
      )}
      {showDecisions && view && (
        <DecisionsDialog
          grades={replaying ? [] : grades}
          deal={deal}
          pending={grading}
          onClose={() => setShowDecisions(false)}
          onReplay={mode === "solo" ? startReplay : undefined}
          onCode={mode === "solo" ? showCode : undefined}
        />
      )}
      {tourDialog && tour && (
        <TournamentDialog
          t={tour}
          personas={personas}
          after={tourDialog.after}
          onPlay={playTourRound}
          onClose={() => setTourDialog(null)}
          onRestart={
            tourDialog.after === undefined && !inTour
              ? () => {
                  setTourDialog(null);
                  setShowNewTour(true);
                }
              : undefined
          }
          helpers={helpers}
          onHelpOff={turnOffHelp}
          onLeave={leaveTourRound}
          onQuit={quitTournament}
        />
      )}
      {showRecap && tour && <RecapDialog t={tour} progress={recapProgress} onClose={() => setShowRecap(false)} />}
      {showNewTour && (
        <NewTournamentDialog
          onStart={beginTournament}
          onRecap={
            tour?.finished && tour.mine?.some(Boolean)
              ? () => {
                  setShowNewTour(false);
                  setShowRecap(true);
                }
              : undefined
          }
          onCancel={() => {
            setShowNewTour(false);
            // Backing out of replacing a tournament leads back to it.
            if (tourRef.current && !tourRef.current.finished) setTourDialog({});
          }}
        />
      )}
      {showFinal && tour && (
        <FinalReveal
          t={tour}
          personas={personas}
          progress={recapProgress}
          onDone={() => {
            updateTour((t) => {
              t.finished = true;
            });
            if (tourRef.current) {
              const broken = recordTournament(tourRef.current);
              if (broken.length) setRecordToast(broken.join(" · "));
              noteTournament(tourRef.current);
              checkAchievements();
            }
            setShowFinal(false);
            setScreen("menu");
            // A career chapter's tournament: did you place high enough?
            const done = tourRef.current;
            if (done?.career) {
              const you = standings(done).find((s) => s.participant.id === 0);
              endLevel(done.career.chapter, !!you && you.place <= done.career.place, done.assisted === false);
            }
          }}
        />
      )}
      {showCareer && (
        <CareerDialog
          career={career}
          resumable={careerResumable}
          onName={(name) =>
            updateCareer((c) => {
              c.name = name;
            })
          }
          onPlay={startChapter}
          onWatch={watchScene}
          helpers={helpers}
          onHelpOff={turnOffHelp}
          onResume={resumeCareer}
          onReset={() => {
            updateCareer((c) => {
              c.done = [];
              c.playing = null;
            });
            setCareerGame(false);
          }}
          onClose={() => setShowCareer(false)}
        />
      )}
      {scene && (
        <Cutscene
          key={scene.id}
          steps={scene.steps}
          name={career.name}
          label={scene.label}
          music={settings.storyMusic}
          sound={settings.sound}
          onMusic={(on) => changeSettings({ ...settings, storyMusic: on })}
          onSound={(on) => changeSettings({ ...settings, sound: on })}
          onDone={() => {
            setScene(null);
            scene.then();
          }}
        />
      )}
      {codeShown && (
        <Dialog title="Hand code" onClose={() => setCodeShown(null)}>
          <p>This code names this exact moment of the game. Give it to Claude with your question.</p>
          <input className="hand-code" readOnly value={codeShown} onFocus={(e) => e.currentTarget.select()} autoFocus aria-label="Hand code" />
          <p className="note">{navigator.clipboard ? "It's been copied to your clipboard." : "Select it and copy it."}</p>
          <div className="actions">
            <button type="button" onClick={() => navigator.clipboard?.writeText(codeShown).catch(() => {})}>
              Copy again
            </button>
            <button type="button" className="primary" onClick={() => setCodeShown(null)}>
              Done
            </button>
          </div>
        </Dialog>
      )}
      {showScores && view && (
        <Dialog title="Score pad" wide onClose={() => setShowScores(false)}>
          <ScorePad history={view.history} players={view.players} onReview={(h) => { setShowScores(false); setReview(h); }} />
        </Dialog>
      )}
      {showLearn && (
        <Learn
          key={learnTab}
          coach={coachRef.current}
          initialTab={learnTab}
          onPlayHand={lan ? undefined : startPuzzle}
          onClose={() => {
            setShowLearn(false);
            setLearnTab("lessons");
          }}
        />
      )}
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
            // Back at a finished game: its result again, the way on.
            else if (view.phase === "gameOver") setResult(view.history.filter((h) => !h.passedOut).at(-1) ?? null);
          }}
        />
      )}
    </div>
  );
}

function HandResult({ hand, view, puzzle }: { hand: HandView; view: View; puzzle?: boolean }) {
  const c = hand.contract!;
  const team = c.declarer % 2;
  const ours = team === 0;
  const who = ours ? (c.declarer === 0 ? "You" : `${view.players[c.declarer].name} (your partner)`) : view.players[c.declarer].name;
  const won = view.winner;
  const short = view.handLimit !== null;
  return (
    <div className="hand-result">
      <p className={hand.made ? "made" : "down"}>
        {who} {hand.made ? "made" : "went down in"}{" "}
        <strong>
          <BidText label={c.label} />
        </strong> with {hand.tricksWon[team]} tricks.
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
      {short && won === null && !puzzle && (
        <p className="note">
          {view.handsPlayed < view.handLimit!
            ? `${view.handLimit! - view.handsPlayed} of ${view.handLimit} hands to go.`
            : "Tied after the last hand: one more hand to settle it."}
        </p>
      )}
      {won !== null && !puzzle && (
        <p className="game-over">
          {won === 0 ? "You and your partner win the game!" : `${view.players[1].name} and ${view.players[3].name} win the game.`}
          {short && ` (${view.handsPlayed} hands)`}
        </p>
      )}
    </div>
  );
}
