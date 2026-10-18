// The game engine (Rust compiled to WebAssembly), running in a Web Worker.

import type { Request, Response, Snapshot } from "./protocol";
import type { Persona } from "./types";

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

/** One worker holding one game. Requests are answered in order. */
export class Engine {
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

  private call<O extends Op>(op: O, args: Args<O>): Promise<unknown> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ id, op, ...args });
    });
  }

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
