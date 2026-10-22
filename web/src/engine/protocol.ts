// Messages between the page and the engine worker.

import type { EventView, View } from "./types";

export type Request =
  | { id: number; op: "personas" }
  | { id: number; op: "create"; seed: number; west: string; north: string; east: string; hands: number }
  | { id: number; op: "load"; save: string }
  | { id: number; op: "act"; index: number }
  | { id: number; op: "stepBot" }
  | { id: number; op: "autoAct" }
  | { id: number; op: "hint"; save: string }
  | { id: number; op: "grade"; save: string; number: number }
  | { id: number; op: "gradeDiscards"; save: string; number: number }
  | { id: number; op: "trace"; save: string; number: number }
  | { id: number; op: "expert"; save: string; number: number }
  | { id: number; op: "drillBidding"; seed: number }
  | { id: number; op: "drillPlay"; seed: number }
  | { id: number; op: "position"; save: string };

export type Response = { id: number; ok: true; result: unknown } | { id: number; ok: false; error: string };

/** The table after a request: what you see, how to restore it, and what just happened. */
export interface Snapshot {
  view: View;
  save: string;
  events: EventView[];
  /** Milliseconds the engine spent on this request (mostly bot thinking). */
  ms: number;
  /** Identifies a LAN game (whose save stays on the host, so is empty here). */
  game?: string;
}
