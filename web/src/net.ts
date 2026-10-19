// Playing at a table hosted on the local network by fh-host (crates/host). The host keeps the game
// and runs the bots and the coach; this client sends your choices and receives the table as your
// seat sees it. It reconnects on its own (phones sleep, wifi blips), and a stored client id gets
// you your seat back.

import type { Coach, ExpertView, GradeView, HintView, Persona, Snapshot, TraceView } from "./engine";
import type { Speed } from "./settings";

export interface HostInfo {
  app: "fh-host";
  version: string;
  /** The address to share with the table. */
  url: string;
}

export interface LobbySeat {
  person: { name: string; connected: boolean } | null;
  /** The bot that plays this seat when no one sits in it. */
  bot: string;
  botName: string;
}

/** The host's table. Seats are absolute here: 0 South, 1 West, 2 North, 3 East. */
export interface Lobby {
  you: number | null;
  seats: LobbySeat[];
  playing: boolean;
  over: boolean;
  pace: Speed;
  /** A short game's number of hands, or null for a game to 500. */
  hands: number | null;
  /** People the table waits on to continue after a hand. */
  waiting: string[];
  personas: Persona[];
}

export const SEAT_NAMES = ["South", "West", "North", "East"];

/** The host's details, or null when the app wasn't loaded from a host. */
export async function detectHost(): Promise<HostInfo | null> {
  try {
    // Never hold up the menu: no answer in a couple of seconds means no host.
    const response = await fetch("./api/info", { cache: "no-store", signal: AbortSignal.timeout(2500) });
    if (!response.ok) return null;
    const info = (await response.json()) as HostInfo;
    return info.app === "fh-host" ? info : null;
  } catch {
    return null;
  }
}

function stored(key: string, make: () => string): string {
  try {
    const value = localStorage.getItem(key);
    if (value) return value;
    const fresh = make();
    localStorage.setItem(key, fresh);
    return fresh;
  } catch {
    return make();
  }
}

/** Your name at the table, remembered on this device. */
export function savedName(): string {
  try {
    return localStorage.getItem("fh500.name") ?? "";
  } catch {
    return "";
  }
}

function storeName(name: string) {
  try {
    localStorage.setItem("fh500.name", name);
  } catch {
    // The name just won't be remembered.
  }
}

export interface NetEvents {
  lobby(lobby: Lobby): void;
  /** The table changed (another player's move, or a bot's). */
  snapshot(snapshot: Snapshot): void;
  /** The game was ended; everyone is back in the lobby. */
  ended(): void;
  notice(text: string): void;
  connection(connected: boolean): void;
}

type Pending = { resolve: (value: unknown) => void; reject: (error: Error) => void };

export class NetClient implements Coach {
  /** Identifies this browser to the host, so a reconnect finds your seat. Not secret. */
  readonly clientId = stored("fh500.clientId", () =>
    Array.from(crypto.getRandomValues(new Uint8Array(12)), (b) => b.toString(16).padStart(2, "0")).join(""),
  );
  private ws: WebSocket | null = null;
  private pending = new Map<number, Pending>();
  private nextId = 1;
  private retry = 0;
  private closed = false;
  private ping: ReturnType<typeof setInterval>;
  private onVisible = () => {
    if (document.visibilityState === "visible" && (!this.ws || this.ws.readyState > WebSocket.OPEN)) this.connect();
    else this.send({ type: "ping" });
  };

  constructor(
    private on: NetEvents,
    private name: string,
  ) {
    this.connect();
    this.ping = setInterval(() => this.send({ type: "ping" }), 10_000);
    document.addEventListener("visibilitychange", this.onVisible);
  }

  private connect() {
    if (this.closed) return;
    const url = new URL("./ws", location.href);
    url.protocol = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(url);
    this.ws = ws;
    ws.onopen = () => {
      this.retry = 0;
      this.send({ type: "hello", clientId: this.clientId, name: this.name });
      this.on.connection(true);
    };
    ws.onmessage = (message: MessageEvent<string>) => {
      const data = JSON.parse(message.data) as { type: string } & Record<string, unknown>;
      switch (data.type) {
        case "lobby":
          this.on.lobby(data as unknown as Lobby);
          break;
        case "snapshot":
          this.on.snapshot(data.snapshot as Snapshot);
          break;
        case "ended":
          this.on.ended();
          break;
        case "notice":
          this.on.notice(String(data.text));
          break;
        case "reply": {
          const waiting = this.pending.get(data.id as number);
          if (!waiting) break;
          this.pending.delete(data.id as number);
          if (data.ok) waiting.resolve(data.result);
          else waiting.reject(new Error(String(data.error)));
          break;
        }
      }
    };
    ws.onclose = () => {
      if (this.ws !== ws) return;
      this.ws = null;
      for (const waiting of this.pending.values()) waiting.reject(new Error("lost the connection to the host"));
      this.pending.clear();
      this.on.connection(false);
      const wait = Math.min(5000, 500 * 2 ** this.retry++);
      setTimeout(() => this.connect(), wait);
    };
  }

  private send(message: object): boolean {
    if (this.ws?.readyState !== WebSocket.OPEN) return false;
    this.ws.send(JSON.stringify(message));
    return true;
  }

  private request<T>(message: object): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      if (!this.send({ ...message, id })) {
        this.pending.delete(id);
        reject(new Error("not connected to the host"));
      }
    });
  }

  close() {
    this.closed = true;
    clearInterval(this.ping);
    document.removeEventListener("visibilitychange", this.onVisible);
    this.ws?.close();
  }

  rename(name: string) {
    this.name = name;
    storeName(name);
    this.send({ type: "rename", name });
  }

  sit(seat: number) {
    this.send({ type: "sit", seat });
  }

  stand() {
    this.send({ type: "stand" });
  }

  setBot(seat: number, persona: string) {
    this.send({ type: "setBot", seat, persona });
  }

  setPace(pace: Speed) {
    this.send({ type: "setPace", pace });
  }

  /** A short game of `hands` hands, or 0 for a game to 500. */
  setLength(hands: number) {
    this.send({ type: "setLength", hands });
  }

  start() {
    this.send({ type: "start" });
  }

  /** Seen the hand's result: ready for the next deal. */
  continue() {
    this.send({ type: "continue" });
  }

  endGame() {
    this.send({ type: "endGame" });
  }

  /** Your action, by its index in `view.legal`; `expect` is the view's action count. */
  act(index: number, expect: number): Promise<Snapshot> {
    return this.request({ type: "act", index, expect });
  }

  // The coach runs on the host, for your seat. The save argument is unused: the host has the game.
  hint(_save: string): Promise<HintView> {
    return this.request({ type: "hint" });
  }

  grade(_save: string, number: number): Promise<GradeView | null> {
    return this.request({ type: "grade", number });
  }

  gradeDiscards(_save: string, number: number): Promise<GradeView | null> {
    return this.request({ type: "gradeDiscards", number });
  }

  trace(_save: string, number: number): Promise<TraceView | null> {
    return this.request({ type: "trace", number });
  }

  expert(_save: string, number: number): Promise<ExpertView> {
    return this.request({ type: "expert", number });
  }
}
