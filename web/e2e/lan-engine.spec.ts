// Solo play from a phone (or any browser) that loaded the app from fh-host: the host's computer
// answers the engine's requests over /engine (crates/wasm/src/service.rs), so a phone never runs
// the bots or the coach itself. A dropped connection picks up from the last save, and a host that
// stops answering hands the game to this device's own engine.

import { expect, test, type Page } from "@playwright/test";
import { startGame, takeTurn } from "./helpers";

/** Takes turns until a few of your cards have gone. */
async function playSome(page: Page, cards: number) {
  const hand = page.getByLabel("Your hand").locator(".card");
  for (let i = 0; i < 400; i++) {
    if ((await hand.count()) <= 10 - cards && (await hand.count()) > 0) return;
    const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
    if (await result.isVisible()) return;
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  throw new Error("the game didn't move");
}

test("solo on a host: the host's computer does the thinking", async ({ page }) => {
  const sent: string[] = [];
  page.on("websocket", (ws) => {
    if (ws.url().endsWith("/engine")) ws.on("framesent", (frame) => sent.push(String(frame.payload)));
  });
  const workers: string[] = [];
  page.on("worker", (worker) => workers.push(worker.url()));
  await startGame(page, { north: "steady", west: "steady", east: "steady" });
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  await playSome(page, 2);
  expect(sent.some((frame) => frame.includes('"op":"create"'))).toBe(true);
  expect(sent.some((frame) => frame.includes('"op":"act"'))).toBe(true);
  // Grades come from the host too.
  await expect.poll(() => sent.some((frame) => frame.includes('"op":"grade'))).toBe(true);
  // No engine of its own on this device.
  expect(workers).toEqual([]);
});

test("solo on a host: a dropped connection picks up from the last save", async ({ page }) => {
  let connections = 0;
  let dropped = false;
  const resynced: number[] = [];
  await page.routeWebSocket(/\/engine$/, (ws) => {
    const n = ++connections;
    const server = ws.connectToServer();
    ws.onMessage((message) => {
      const text = String(message);
      // The first card you play is lost with its connection, before the host answers.
      if (!dropped && text.includes('"op":"act"')) {
        dropped = true;
        void ws.close();
        return;
      }
      if (text.startsWith('{"id":0,"op":"load"')) resynced.push(n);
      server.send(message);
    });
    server.onMessage((message) => ws.send(message));
  });
  await startGame(page, { north: "steady", west: "steady", east: "steady" });
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  await playSome(page, 3);
  expect(dropped).toBe(true);
  // A new connection got the game back before the lost request was sent again.
  expect(resynced.length).toBeGreaterThan(0);
  expect(resynced[0]).toBeGreaterThan(1);
});

test("solo on a host: a host that stops answering hands the game to this device", async ({ page }) => {
  // Connections open but nothing ever comes back.
  await page.routeWebSocket(/\/engine$/, () => {});
  const workers: string[] = [];
  page.on("worker", (worker) => workers.push(worker.url()));
  test.setTimeout(240_000);
  await startGame(page, { north: "steady", west: "steady", east: "steady" });
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10, { timeout: 60_000 });
  await playSome(page, 1);
  expect(workers.length).toBeGreaterThan(0);
});
