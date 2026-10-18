// Typed access to the Rust engine compiled to WebAssembly (crates/wasm).

import init, { WasmTable, personas as wasmPersonas } from "./pkg/fh_wasm.js";
import type { EventView, Persona, View } from "./types";

export * from "./types";

let ready: Promise<unknown> | null = null;

export function loadEngine(): Promise<unknown> {
  ready ??= init();
  return ready;
}

export interface Seating {
  west: string;
  north: string;
  east: string;
}

/** A game in progress. Wraps the engine's table so the rest of the app sees plain types. */
export class Table {
  private constructor(private inner: WasmTable) {}

  static create(seed: number, seating: Seating): Table {
    return new Table(new WasmTable(seed >>> 0, seating.west, seating.north, seating.east));
  }

  /** Rebuilds a saved game; throws if the save is unreadable. */
  static load(save: string): Table {
    return new Table(WasmTable.load(save));
  }

  save(): string {
    return this.inner.save();
  }

  view(): View {
    return this.inner.view() as View;
  }

  act(index: number): EventView[] {
    return this.inner.act(index) as EventView[];
  }

  /** The next bot move, or null if it's your turn or the game is over. */
  stepBot(): EventView[] | null {
    return this.inner.stepBot() as EventView[] | null;
  }
}

export function personas(): Persona[] {
  return wasmPersonas() as Persona[];
}
