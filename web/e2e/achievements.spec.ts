// Achievements: challenges with progress, measured from what's kept (so past play counts) and from
// moments noted as they happen. The page lists them in Learn → Achievements.

import { expect, test } from "@playwright/test";
import { freshStart } from "./helpers";

test("past play counts toward achievements, quietly, and the page shows progress", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  await freshStart(page);
  await page.evaluate(() => {
    const bids = { "8♦": { bids: 3, made: 2 }, "10NT": { bids: 1, made: 1 } };
    localStorage.setItem("fh500.records", JSON.stringify({ highGame: 510, bestHand: null, highBid: null, gamesPlayed: 3, gamesWon: 2, bids, counted: [] }));
    localStorage.setItem(
      "fh500.progress",
      JSON.stringify({ games: [], quiz: { asked: 9, correct: 5 }, bidding: { asked: 0, correct: 0 }, puzzles: { asked: 0, correct: 0 } }),
    );
  });
  await page.reload();
  await page.getByRole("navigation", { name: "Main menu" }).waitFor();
  // Already reached on start: no banner for those.
  await expect(page.locator(".record-toast")).toHaveCount(0);
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Learn/ }).click();
  await page.getByRole("tab", { name: "Achievements" }).click();
  const done = page.locator(".achievement.done");
  for (const title of ["Off the mark", "No trumps needed", "All in", "The big one", "First win"]) await expect(done.filter({ hasText: title })).toBeVisible();
  await expect(page.locator(".achievement", { hasText: "Full set" })).toContainText("2 of 5");
  await expect(page.locator(".achievement", { hasText: "Card counter" })).toContainText("5 of 25");
  await expect(page.locator(".achievement", { hasText: "Seasoned" })).toContainText("2 of 25");
  await expect(page.getByText(/^5 of 30/)).toBeVisible();
  await page.screenshot({ path: "test-results/achievements.png", fullPage: true });
});

test("moments: slams, sets, streaks, comebacks, shutouts, giant killing, a clean game", async ({}, info) => {
  test.skip(info.project.name !== "desktop", "pure logic");
  const store = new Map<string, string>();
  Object.assign(globalThis, {
    localStorage: { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => store.set(k, v), removeItem: (k: string) => store.delete(k) },
  });
  const { achievements, noteGame, noteGrades, noteHand } = await import("../src/achievements");
  type Hand = Parameters<typeof noteHand>[1];
  type Game = Parameters<typeof noteGame>[1];
  const hand = (number: number, declarer: number, level: number, made: boolean, tricks: [number, number], after: [number, number]) =>
    ({ number, passedOut: false, contract: { declarer, level, strain: "H", value: 0, label: `${level}♥` }, made, tricksWon: tricks, scoresAfter: after }) as unknown as Hand;
  const game = (winner: 0 | 1, scores: [number, number], history: Hand[], opponents = ["steady", "steady"]) =>
    ({ winner, scores, history, players: [{ persona: "human" }, { persona: opponents[0] }, { persona: "steady" }, { persona: opponents[1] }] }) as unknown as Game;
  const status = (id: string) => achievements(true).find((s) => s.achievement.id === id)!;

  noteHand("g1", hand(1, 2, 7, true, [10, 0], [250, 0])); // partner's slam
  noteHand("g1", hand(1, 2, 7, true, [10, 0], [250, 0])); // the same hand again: not counted twice
  noteHand("g1", hand(2, 1, 8, false, [5, 5], [300, -280])); // West's 8 set
  noteHand("g1", hand(3, 3, 7, false, [6, 4], [360, -400])); // East's 7 set
  expect(status("slam").done).not.toBeNull();
  expect(status("brick-wall").done).not.toBeNull();
  expect(status("defender").value).toBe(2);

  // Behind by 200 then won, the other side below zero, against two experts.
  noteGame("g1", game(0, [520, -400], [hand(1, 1, 8, true, [3, 7], [30, 240]), hand(2, 0, 9, true, [9, 1], [520, -400])], ["shark", "oracle"]));
  expect(status("comeback").done).not.toBeNull();
  expect(status("shutout").done).not.toBeNull();
  expect(status("giant-killer").done).not.toBeNull();
  noteGame("g2", game(0, [500, 100], []));
  expect(status("hot-streak").value).toBe(2);
  noteGame("g3", game(1, [100, 500], []));
  noteGame("g4", game(0, [500, 100], []));
  expect(status("hot-streak").value).toBe(2); // a loss broke the streak
  expect(status("hot-streak").done).toBeNull();

  const grade = (verdict: string, hinted = false) => ({ verdict, hinted }) as unknown as Parameters<typeof noteGrades>[1][number];
  noteGrades("g2", Array.from({ length: 20 }, (_, i) => grade(i === 5 ? "mistake" : "good")));
  noteGrades("g3", Array.from({ length: 20 }, (_, i) => grade("best", i === 3)));
  noteGrades("g4", Array.from({ length: 10 }, () => grade("best")));
  expect(status("clean-sheet").done).toBeNull();
  noteGrades("g4", Array.from({ length: 16 }, (_, i) => grade(i % 2 ? "inaccuracy" : "best")));
  expect(status("clean-sheet").done).not.toBeNull();
});

test("tournament wins by kind, and unaided", async ({}, info) => {
  test.skip(info.project.name !== "desktop", "pure logic");
  const store = new Map<string, string>();
  Object.assign(globalThis, {
    localStorage: { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => store.set(k, v), removeItem: (k: string) => store.delete(k) },
  });
  const { achievements, noteTournament } = await import("../src/achievements");
  const { recordTournament } = await import("../src/records");
  const { newTournament, ROUNDS } = await import("../src/tournament");
  const status = (id: string) => achievements(true).find((s) => s.achievement.id === id)!;
  /** A finished tournament of `kind` in which you score most (or least) every round. */
  const finished = (seed: number, kind: "random" | "expert" | "chaos" | undefined, win: boolean, assisted?: boolean) => {
    const t = newTournament(seed, kind);
    if (kind === undefined) delete t.kind;
    t.assisted = assisted;
    for (const round of t.rounds) {
      for (const table of round.tables) {
        const yours = table.seats.includes(0);
        const scores: [number, number] = yours ? (win ? [400, 0] : [-200, 0]) : [100, 50];
        table.result = { scores, hands: [scores] };
      }
    }
    t.current = ROUNDS;
    t.finished = true;
    recordTournament(t);
    noteTournament(t);
    return t;
  };

  // Won from before there were kinds (no kind kept): a random one, and not unaided (unknown).
  finished(1, undefined, true);
  expect(status("random-champion").done).not.toBeNull();
  expect(status("unaided").done).toBeNull();
  // An expert tournament lost, then won with help.
  finished(2, "expert", false, false);
  expect(status("expert-champion").done).toBeNull();
  finished(3, "expert", true, true);
  expect(status("expert-champion").done).not.toBeNull();
  expect(status("unaided").done).toBeNull();
  expect(status("chaos-champion").done).toBeNull();
  // A chaos tournament won unaided.
  finished(4, "chaos", true, false);
  expect(status("chaos-champion").done).not.toBeNull();
  expect(status("unaided").done).not.toBeNull();
  expect(status("dynasty").done).not.toBeNull();
});
