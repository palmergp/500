// Mirrors the JSON views in crates/wasm/src/view.rs.

export type SuitCode = "S" | "C" | "D" | "H";
export type StrainCode = SuitCode | "NT";
export type Phase = "bidding" | "discarding" | "playing" | "gameOver";

/** 0 South (you), 1 West, 2 North (partner), 3 East. */
export type SeatIndex = 0 | 1 | 2 | 3;

export interface CardView {
  id: number;
  rank: string;
  suit: SuitCode | null;
  label: string;
  red: boolean;
  bower: "right" | "left" | null;
  trump: boolean;
}

export interface ActionView {
  index: number;
  kind: "pass" | "mention" | "six" | "bid" | "discard" | "play";
  label: string;
  level: number | null;
  strain: StrainCode | null;
  card: number | null;
  namedSuit: SuitCode | null;
}

export interface CallView {
  seat: SeatIndex;
  kind: "pass" | "mention" | "six" | "bid";
  label: string;
}

export interface ContractView {
  declarer: SeatIndex;
  level: number;
  strain: StrainCode;
  value: number;
  label: string;
}

export interface TrickView {
  leader: SeatIndex;
  plays: { seat: SeatIndex; card: CardView }[];
  namedSuit: SuitCode | null;
  winner: SeatIndex | null;
}

export interface PlayerView {
  seat: SeatIndex;
  name: string;
  persona: string;
  blurb: string;
}

export interface HandView {
  number: number;
  dealer: SeatIndex;
  dealNumber: number;
  passedOut: boolean;
  contract: ContractView | null;
  made: boolean | null;
  tricksWon: [number, number];
  scoreChange: [number, number];
  scoresAfter: [number, number];
  calls: CallView[];
  dealt: CardView[][];
  kitty: CardView[];
  discards: CardView[];
  tricks: TrickView[];
}

export interface View {
  phase: Phase;
  toAct: SeatIndex | null;
  humanToAct: boolean;
  dealer: SeatIndex;
  dealNumber: number;
  dealerForced: boolean;
  mustBid: boolean;
  scores: [number, number];
  target: number;
  winner: 0 | 1 | null;
  hand: CardView[];
  handCounts: [number, number, number, number];
  calls: CallView[];
  contract: ContractView | null;
  kitty: CardView[] | null;
  discards: CardView[] | null;
  trick: TrickView | null;
  lastTrick: TrickView | null;
  /** The tricks finished so far this hand, in order. */
  tricks: TrickView[];
  /** A short game's number of hands, or null for a game to 500. */
  handLimit: number | null;
  /** Hands bid and played so far this game. */
  handsPlayed: number;
  tricksWon: [number, number];
  legal: ActionView[];
  players: PlayerView[];
  history: HandView[];
  /** Actions taken so far: identifies a decision for the coach. */
  actionCount: number;
  /** Cards played to tricks this hand. */
  played: CardView[];
  /** Trumps not in your hand, not played, and not among your discards. */
  trumpsUnseen: number | null;
  /** For each seat, the suits (S C D H, by effective suit) it has shown out of this hand. */
  shownOut: boolean[][];
  /** For each suit (S C D H, by effective suit), cards you haven't seen. */
  unseenBySuit: number[];
  /** Whether you haven't seen the Joker yet. */
  jokerUnseen: boolean;
}

// ---- Coach (mirrors crates/wasm/src/coach.rs) --------------------------------------------

export interface OptionView {
  index: number;
  label: string;
  ev: number;
  makes: number | null;
}

export interface HintView {
  best: number;
  bestLabel: string;
  reasons: string[];
  options: OptionView[];
  worlds: number;
  /** Indices (in `view.legal`) of plays about as good as the best: a close call. */
  close: number[];
}

export type Verdict = "best" | "good" | "inaccuracy" | "mistake" | "blunder";

export interface GradeView {
  actionNumber: number;
  phase: Phase;
  chosenLabel: string;
  bestLabel: string;
  verdict: Verdict;
  loss: number;
  reasons: string[];
  options: OptionView[];
  deal: number;
  /** For a card play, which trick of the hand (1–10). */
  trick: number | null;
}

export interface TraceView {
  par: number;
  plays: { seat: SeatIndex; card: CardView; parAfter: number; cost: number }[];
}

export interface DrillView {
  view: View;
  save: string;
}

export interface ExpertView {
  actual: number;
  expert: number;
  expertContract: string | null;
  expertDeclarer: SeatIndex | null;
  expertMade: boolean | null;
}

export interface EventView {
  kind:
    | "dealt"
    | "called"
    | "passedOut"
    | "contractWon"
    | "discarded"
    | "played"
    | "trickWon"
    | "handScored"
    | "gameWon";
  seat: SeatIndex | null;
  text: string;
}

export interface Persona {
  id: string;
  name: string;
  blurb: string;
}

export const SEAT_NAMES = ["South", "West", "North", "East"] as const;

export const SUIT_SYMBOL: Record<SuitCode, string> = { S: "♠", C: "♣", D: "♦", H: "♥" };

export function strainLabel(strain: StrainCode): string {
  return strain === "NT" ? "NT" : SUIT_SYMBOL[strain];
}

export function isRed(strain: StrainCode | null): boolean {
  return strain === "D" || strain === "H";
}
