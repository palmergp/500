// Runs the Rust engine (and its bots' thinking) off the main thread, so the page stays responsive
// while a search bot works. A second instance runs the coach, so analysis never delays play.
// Each request gets one reply carrying the same id.

import init, { coachExpert, coachGrade, coachHint, coachTrace, drillBidding, drillPlay, WasmTable, personas } from "./pkg/fh_wasm.js";
import type { Request, Response, Snapshot } from "./protocol";

const ready = init();
let table: WasmTable | null = null;

let started = 0;

function snapshot(events: unknown): Snapshot {
  const ms = performance.now() - started;
  return { view: table!.view(), save: table!.save(), events: (events ?? []) as Snapshot["events"], ms };
}

function handle(request: Request): unknown {
  switch (request.op) {
    case "personas":
      return personas();
    case "create":
      table?.free();
      table = new WasmTable(request.seed >>> 0, request.west, request.north, request.east);
      return snapshot([]);
    case "load":
      table?.free();
      table = WasmTable.load(request.save);
      return snapshot([]);
    case "act":
      return snapshot(table!.act(request.index));
    case "stepBot": {
      const events = table!.stepBot();
      return events === null ? null : snapshot(events);
    }
    case "hint":
      return coachHint(request.save);
    case "grade":
      return coachGrade(request.save, request.number);
    case "trace":
      return coachTrace(request.save, request.number);
    case "expert":
      return coachExpert(request.save, request.number);
    case "drillBidding":
      return drillBidding(request.seed >>> 0);
    case "drillPlay":
      return drillPlay(request.seed >>> 0);
  }
}

self.onmessage = async (message: MessageEvent<Request>) => {
  await ready;
  const request = message.data;
  started = performance.now();
  let response: Response;
  try {
    response = { id: request.id, ok: true, result: handle(request) };
  } catch (error) {
    response = { id: request.id, ok: false, error: String(error) };
  }
  self.postMessage(response);
};
