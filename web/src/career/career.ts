// Career progress, kept in this browser: what you're called, the chapters you've finished, and the
// one you're playing (its game's seed, or its tournament's id, so Continue finds it again).

import type { HandView } from "../engine/types";
import { CHAPTERS, CHARACTERS, type Banter, type CharacterId } from "./story";

export interface CareerState {
  /** What the story calls you. Empty until you've said. */
  name: string;
  /** Chapters finished, by index. */
  done: number[];
  /** Chapters ever beaten with no help: no hints, undos or Finish hand, and the helpers off (as
   * an unaided tournament). Kept through Start over, like an achievement. */
  unaided: number[];
  /** The chapter under way: its level's game (seed) or tournament (id), tries that fell short, and
   * whether this try has had help (a story tournament keeps its own, `Tournament.assisted`). */
  playing: { chapter: number; seed: number | null; tour: string | null; attempts: number; assisted?: boolean } | null;
}

const KEY = "fh500.career";

export function loadCareer(): CareerState {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return { name: "", done: [], unaided: [], playing: null, ...(JSON.parse(raw) as Partial<CareerState>) };
  } catch {
    // A fresh career.
  }
  return { name: "", done: [], unaided: [], playing: null };
}

export function saveCareer(c: CareerState) {
  try {
    localStorage.setItem(KEY, JSON.stringify(c));
  } catch {
    // Without storage, the career lasts as long as the page.
  }
}

/** The next chapter to play: the first not yet finished (CHAPTERS.length once all are). */
export function nextChapter(c: CareerState): number {
  const i = CHAPTERS.findIndex((_, i) => !c.done.includes(i));
  return i < 0 ? CHAPTERS.length : i;
}

/** A chapter can be played once every chapter before it is done. */
export function unlocked(c: CareerState, chapter: number): boolean {
  return chapter <= nextChapter(c);
}

/** Story text with your name in it. */
export function fill(text: string, name: string): string {
  return text.replaceAll("{name}", name || "kid");
}

/** Who says what after a hand: picked by how it went, varied from hand to hand. */
export function banterFor(banter: Banter, hand: HandView): { who: CharacterId; text: string } | null {
  const c = hand.contract;
  if (!c) return null;
  const ours = c.declarer % 2 === 0;
  const list = ours ? (hand.made ? banter.made : banter.set) : hand.made ? banter.theyMade : banter.theySet;
  if (list.length === 0) return null;
  const [who, text] = list[hand.number % list.length];
  return { who, text };
}

export function characterName(id: CharacterId | "you", name: string): string {
  return id === "you" ? name || "You" : CHARACTERS[id].name;
}
