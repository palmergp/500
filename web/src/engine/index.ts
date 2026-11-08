// The game engine (Rust compiled to WebAssembly), running in a Web Worker, or on the host computer
// (crates/wasm/src/service.rs, natively) when the app was loaded from one.

import type { Request, Response, Snapshot } from "./protocol";
import type { DrillView, ExpertView, GradeView, HintView, Persona, TraceView } from "./types";

export * from "./types";
export type { Snapshot } from "./protocol";

/** Who plays the other seats (persona ids, or "any" for a random one) and the game's length. */
export interface Seating {
  west: string;
  north: string;
  east: string;
  /** A short game's number of hands, or 0 for a game to 500. */
  hands: number;
  /** Display names for West, North and East (a tournament's players). */
  names?: [string, string, string];
}

type Op = Request extends infer R ? (R extends { op: infer O } ? O : never) : never;
type Args<O extends Op> = Omit<Extract<Request, { op: O }>, "id" | "op">;
type Pending = { resolve: (value: unknown) => void; reject: (error: Error) => void; request: Request; remote: boolean };

let hostEngine = false;

/** Has the computer hosting the app answer engine requests (bots' moves, the coach) rather than
 * this device: a phone's WebAssembly is far slower. Set before creating any client. Without a
 * host, or when it can't be reached, each client uses its own worker on this device. */
export function setHostEngine(on: boolean) {
  hostEngine = on;
}

/** Requests that need the game itself (the play engine's table), rather than a save. */
const NEEDS_GAME = new Set<Op>(["act", "autoAct", "stepBot"]);
/** How long the host may leave a ping unanswered before its connection counts as lost. */
const SILENCE_MS = 6000;
/** After failing to reach the host, how long to use this device's engine before trying again. */
const RETRY_MS = 15000;

/** The engine: a worker on this device, or the host computer (`/engine`: the same requests and
 * replies over a WebSocket). Requests are answered in order. If the host's connection drops,
 * whatever was waiting on it is sent again: to a fresh connection, else to this device's worker.
 * Either way the play engine first gets the game back from its last save, so nothing is lost. */
class WorkerClient {
  private worker: Worker | null = null;
  private socket: WebSocket | null = null;
  private socketOpen = false;
  /** Whether the current connection has answered anything (one that never does is no use). */
  private socketAnswered = false;
  /** Requests waiting for the connection to open. */
  private queued: Request[] = [];
  private remoteDownUntil = 0;
  /** Which engine holds the current game ("remote": the current connection). */
  private holder: "local" | "remote" | null = null;
  private lastSave: string | null = null;
  private pending = new Map<number, Pending>();
  private nextId = 1;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private pingSentAt = 0;

  /** `stateful`: this client holds a game (the play engine), which a new engine must be given. */
  constructor(private stateful = false) {
    if (typeof document !== "undefined") {
      // Back from a locked screen: ask again rather than judge by a ping sent before the sleep.
      document.addEventListener("visibilitychange", () => {
        if (document.visibilityState === "visible") this.pingSentAt = 0;
      });
    }
  }

  protected call<O extends Op>(op: O, args: Args<O>): Promise<unknown> {
    const id = this.nextId++;
    const request = { id, op, ...args } as Request;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject, request, remote: false });
      this.dispatch(request);
    });
  }

  private dispatch(request: Request) {
    const waiting = this.pending.get(request.id);
    if (!waiting) return;
    waiting.remote = hostEngine && Date.now() >= this.remoteDownUntil;
    if (waiting.remote) this.sendRemote(request);
    else this.transmit(request, "local");
  }

  /** Sends a request to an engine, first giving it the game if it doesn't have it. */
  private transmit(request: Request, to: "local" | "remote") {
    const post = (r: Request) => (to === "local" ? this.localWorker().postMessage(r) : this.socket!.send(JSON.stringify(r)));
    if (this.stateful) {
      if (request.op === "create" || request.op === "load") this.holder = to;
      else if (NEEDS_GAME.has(request.op) && this.holder !== to && this.lastSave) {
        post({ id: 0, op: "load", save: this.lastSave });
        this.holder = to;
      }
    }
    post(request);
  }

  private reply(reply: Response) {
    if (reply.ok && this.stateful) {
      const save = (reply.result as { save?: unknown } | null)?.save;
      if (typeof save === "string") this.lastSave = save;
    }
    const waiting = this.pending.get(reply.id);
    if (!waiting) return;
    this.pending.delete(reply.id);
    if (reply.ok) waiting.resolve(reply.result);
    else waiting.reject(new Error(reply.error));
  }

  private localWorker(): Worker {
    if (this.worker) return this.worker;
    const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    worker.onmessage = (message: MessageEvent<Response>) => this.reply(message.data);
    // A worker that can't load (say, offline before the engine was cached) fails every request,
    // so the page says so rather than waiting forever.
    worker.onerror = (event) => {
      event.preventDefault();
      for (const [id, waiting] of this.pending) {
        if (waiting.remote) continue;
        waiting.reject(new Error(event.message || "the engine failed to load"));
        this.pending.delete(id);
      }
    };
    this.worker = worker;
    return worker;
  }

  private sendRemote(request: Request) {
    if (this.socket && this.socketOpen) return this.transmit(request, "remote");
    this.queued.push(request);
    if (this.socket) return;
    const url = new URL("./engine", location.href);
    url.protocol = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(url);
    this.socket = ws;
    this.socketOpen = false;
    this.socketAnswered = false;
    if (this.holder === "remote") this.holder = null;
    const opening = setTimeout(() => this.lost(ws), 3000);
    ws.onopen = () => {
      clearTimeout(opening);
      if (ws !== this.socket) return;
      this.socketOpen = true;
      const queued = this.queued;
      this.queued = [];
      for (const r of queued) if (this.pending.has(r.id)) this.transmit(r, "remote");
      this.pingTimer = setInterval(() => this.ping(ws), 2000);
    };
    ws.onmessage = (message: MessageEvent<string>) => {
      if (ws !== this.socket) return;
      this.pingSentAt = 0;
      const data = JSON.parse(message.data) as Response | { pong: number };
      if ("pong" in data) return;
      this.socketAnswered = true;
      this.reply(data);
    };
    ws.onclose = () => {
      clearTimeout(opening);
      this.lost(ws);
    };
  }

  /** While the host has work of ours, checks it's still there. */
  private ping(ws: WebSocket) {
    if (![...this.pending.values()].some((w) => w.remote)) {
      this.pingSentAt = 0;
      return;
    }
    if (this.pingSentAt === 0) {
      this.pingSentAt = Date.now();
      ws.send(JSON.stringify({ ping: this.pingSentAt }));
    } else if (Date.now() - this.pingSentAt > SILENCE_MS) {
      this.lost(ws);
    }
  }

  /** The host's connection is gone: send what was waiting on it again (see `dispatch`). */
  private lost(ws: WebSocket) {
    if (ws !== this.socket) return;
    this.socket = null;
    this.socketOpen = false;
    this.queued = [];
    this.pingSentAt = 0;
    if (this.pingTimer) clearInterval(this.pingTimer);
    this.pingTimer = null;
    if (this.holder === "remote") this.holder = null;
    // A connection that worked (a locked phone's, say) is worth opening again; one that never
    // did means the host can't help for now.
    if (!this.socketAnswered) this.remoteDownUntil = Date.now() + RETRY_MS;
    ws.onclose = null;
    ws.close();
    const resend = [...this.pending.values()].filter((w) => w.remote).map((w) => w.request);
    for (const r of resend.sort((a, b) => a.id - b.id)) this.dispatch(r);
  }
}

/** The worker holding the game being played. */
export class Engine extends WorkerClient {
  constructor() {
    super(true);
  }

  personas(): Promise<Persona[]> {
    return this.call("personas", {}) as Promise<Persona[]>;
  }

  create(seed: number, seating: Seating): Promise<Snapshot> {
    const { west, north, east, hands, names } = seating;
    return this.call("create", { seed, west, north, east, hands, names }) as Promise<Snapshot>;
  }

  /** Rebuilds a saved game; rejects if the save is unreadable. */
  load(save: string): Promise<Snapshot> {
    return this.call("load", { save }) as Promise<Snapshot>;
  }

  /** Your action, by its index in `view.legal`. */
  act(index: number): Promise<Snapshot> {
    return this.call("act", { index }) as Promise<Snapshot>;
  }

  /** Your turn, played for you by a quick bot ("finish the hand"). */
  autoAct(): Promise<Snapshot> {
    return this.call("autoAct", {}) as Promise<Snapshot>;
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
  /** The whole kitty discard starting at action `number`, graded as one decision. */
  gradeDiscards(save: string, number: number): Promise<GradeView | null>;
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

  gradeDiscards(save: string, number: number): Promise<GradeView | null> {
    return this.call("gradeDiscards", { save, number }) as Promise<GradeView | null>;
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

  /** A saved position (one of your past mistakes), to replay as a puzzle. */
  position(save: string): Promise<DrillView> {
    return this.call("position", { save }) as Promise<DrillView>;
  }

  /** A card-play puzzle: a position with one clearly best play. */
  drillPlay(seed: number): Promise<DrillView> {
    return this.call("drillPlay", { seed }) as Promise<DrillView>;
  }
}

/** A tournament table played out by four bots: each side's final score, and every hand's. */
export interface SimulatedGame {
  scores: [number, number];
  hands: [number, number][];
}

/** A third worker for a tournament's other tables, so they never slow the game you're playing. */
export class Simulator extends WorkerClient {
  simulate(seed: number, players: [string, string, string, string], hands: number): Promise<SimulatedGame> {
    return this.call("simulate", { seed, players, hands }) as Promise<SimulatedGame>;
  }
}
