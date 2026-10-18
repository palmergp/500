// The game engine (Rust compiled to WebAssembly), running in a Web Worker.

import type { Request, Response, Snapshot } from "./protocol";
import type { DrillView, ExpertView, GradeView, HintView, Persona, TraceView } from "./types";

export * from "./types";
export type { Snapshot } from "./protocol";

export interface Seating {
  west: string;
  north: string;
  east: string;
}

type Pending = { resolve: (value: unknown) => void; reject: (error: Error) => void };
type Op = Request extends infer R ? (R extends { op: infer O } ? O : never) : never;
type Args<O extends Op> = Omit<Extract<Request, { op: O }>, "id" | "op">;

/** A worker running the engine. Requests are answered in order. */
class WorkerClient {
  private worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
  private pending = new Map<number, Pending>();
  private nextId = 1;

  constructor() {
    this.worker.onmessage = (message: MessageEvent<Response>) => {
      const reply = message.data;
      const waiting = this.pending.get(reply.id);
      if (!waiting) return;
      this.pending.delete(reply.id);
      if (reply.ok) waiting.resolve(reply.result);
      else waiting.reject(new Error(reply.error));
    };
  }

  protected call<O extends Op>(op: O, args: Args<O>): Promise<unknown> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ id, op, ...args });
    });
  }
}

/** The worker holding the game being played. */
export class Engine extends WorkerClient {
  personas(): Promise<Persona[]> {
    return this.call("personas", {}) as Promise<Persona[]>;
  }

  create(seed: number, seating: Seating): Promise<Snapshot> {
    return this.call("create", { seed, ...seating }) as Promise<Snapshot>;
  }

  /** Rebuilds a saved game; rejects if the save is unreadable. */
  load(save: string): Promise<Snapshot> {
    return this.call("load", { save }) as Promise<Snapshot>;
  }

  /** Your action, by its index in `view.legal`. */
  act(index: number): Promise<Snapshot> {
    return this.call("act", { index }) as Promise<Snapshot>;
  }

  /** The next bot move, or null if it's your turn or the game is over. */
  stepBot(): Promise<Snapshot | null> {
    return this.call("stepBot", {}) as Promise<Snapshot | null>;
  }
}

/** What the game screens ask of the coach: the browser's own worker, or the LAN host. */
export interface Coach {
  hint(save: string): Promise<HintView>;
  grade(save: string, number: number): Promise<GradeView | null>;
  trace(save: string, number: number): Promise<TraceView | null>;
  expert(save: string, number: number): Promise<ExpertView>;
}

/** A second worker for the coach: it rebuilds positions from saves, so play never waits on it. */
export class CoachClient extends WorkerClient implements Coach {
  hint(save: string): Promise<HintView> {
    return this.call("hint", { save }) as Promise<HintView>;
  }

  /** The grade for action `number` of the game, or null if there was no real choice. */
  grade(save: string, number: number): Promise<GradeView | null> {
    return this.call("grade", { save, number }) as Promise<GradeView | null>;
  }

  /** The all-cards-visible trace of deal `number`, or null if it was passed out. */
  trace(save: string, number: number): Promise<TraceView | null> {
    return this.call("trace", { save, number }) as Promise<TraceView | null>;
  }

  /** Deal `number` replayed with The Counter in your seat. */
  expert(save: string, number: number): Promise<ExpertView> {
    return this.call("expert", { save, number }) as Promise<ExpertView>;
  }

  /** A bidding decision to practice: a fresh deal at your turn to call. */
  drillBidding(seed: number): Promise<DrillView> {
    return this.call("drillBidding", { seed }) as Promise<DrillView>;
  }

  /** A card-play puzzle: a position with one clearly best play. */
  drillPlay(seed: number): Promise<DrillView> {
    return this.call("drillPlay", { seed }) as Promise<DrillView>;
  }
}
