// Full-hand puzzles (Learn → Full hands): play a hard bid out from the kitty at the table; nothing
// counts and your own game is left as it was. And the counting quiz's questions.

import { expect, test } from "@playwright/test";
import { startGame, takeTurn } from "./helpers";

test("a full-hand puzzle: from Learn to the table, played out, graded, and back to your game", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(600_000);
  await startGame(page);
  const hand = page.getByLabel("Your hand");
  await expect(hand.locator(".card")).toHaveCount(10);
  const mine = await hand.locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label")));

  // Learn pauses the bots: your game as it is now.
  await page.getByRole("button", { name: "Learn", exact: true }).click();
  const saved = await page.evaluate(() => localStorage.getItem("fh500.save"));
  await page.getByRole("tab", { name: "Full hands" }).click();
  await page.screenshot({ path: "test-results/puzzles-list.png" });
  const first = page.locator(".hand-puzzle").first();
  const contract = (await first.locator("strong").innerText()).trim();
  await first.click();

  // At the table: the bid is yours, the kitty is in your hand, and there's no Finish hand.
  await expect(page.locator(".puzzle-banner")).toContainText(`Puzzle 1: make ${contract}`);
  await expect(hand.locator(".card")).toHaveCount(13);
  await expect(page.getByRole("button", { name: /^Finish hand/ })).toHaveCount(0);
  await page.screenshot({ path: "test-results/puzzles-table.png" });

  const result = page.getByRole("dialog", { name: "Puzzle 1" });
  for (let step = 0; step < 3000 && !(await result.isVisible()); step++) {
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await expect(result.locator(".puzzle-verdict")).toHaveText(/Solved|went down|with a hint/);
  await expect(result.getByRole("button", { name: "Try again" })).toBeVisible();
  await page.screenshot({ path: "test-results/puzzles-result.png" });
  await expect(result.getByRole("button", { name: "Next puzzle" })).toBeVisible();
  // Graded, shown once it's over.
  await result.getByRole("button", { name: "Review hand" }).click();
  const review = page.getByRole("dialog", { name: "Hand review" });
  await review.getByRole("tab", { name: /^Your decisions \(\d+\)/ }).click();
  await expect(review.locator(".verdict").first()).toBeVisible();
  await review.getByRole("button", { name: "Close" }).click();
  // Nothing counted, and your game wasn't touched.
  // (A bot's move already under way when Learn opened may still land in it: it's still your game.)
  const after = JSON.parse((await page.evaluate(() => localStorage.getItem("fh500.save")))!) as { seed: number; log: number[]; hands?: number };
  const before = JSON.parse(saved!) as { seed: number; log: number[] };
  expect(after.seed).toBe(before.seed);
  expect(after.hands).toBeUndefined();
  expect(after.log.slice(0, before.log.length)).toEqual(before.log);
  expect(after.log.length).toBeLessThanOrEqual(before.log.length + 1);
  expect(await page.evaluate(() => localStorage.getItem("fh500.records"))).toBeNull();

  await result.getByRole("button", { name: "All puzzles" }).click();
  await expect(page.getByRole("tab", { name: "Full hands" })).toHaveAttribute("aria-selected", "true");
  await page.getByRole("dialog", { name: "Learn" }).getByRole("button", { name: "Close" }).click();
  await expect.poll(() => hand.locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label")))).toEqual(mine);
  await expect(page.locator(".puzzle-banner")).toHaveCount(0);
});

test("the counting quiz: the highest card not yet played, never on the last trick, and how often", async ({}, info) => {
  test.skip(info.project.name !== "desktop", "pure logic");
  const { makeQuestion, quizDue } = await import("../src/components/CountingQuiz");
  type View = Parameters<typeof makeQuestion>[0];
  const card = (rank: string, suit: string | null) => ({ id: 0, rank, suit, label: "", red: false, bower: null, trump: false });
  // Hearts are trumps, and you declared: the Joker, J♥, A♠ and K♠ have gone, so the left bower (J♦)
  // is the highest trump left and your Q♠ the highest spade. Your discards are low diamonds.
  const base = {
    phase: "playing",
    humanToAct: true,
    contract: { declarer: 0, strain: "H", level: 8, label: "8♥" },
    played: [card("Joker", null), card("J", "H"), card("5", "H"), card("6", "H"), card("A", "S"), card("K", "S"), card("5", "S"), card("6", "S")],
    discards: [card("4", "D"), card("5", "D"), card("6", "D")],
    hand: [card("Q", "S"), card("A", "H"), card("9", "C")],
    unseenBySuit: [0, 0, 0, 0],
    shownOut: [[], [false, false, false, false], [false, false, false, false], [false, false, false, false]],
    players: [{ name: "You" }, { name: "West" }, { name: "North" }, { name: "East" }],
    trumpsUnseen: 0,
  };
  const asked: Record<string, string> = {};
  for (let n = 0; n < 200; n++) {
    const q = makeQuestion({ ...base, actionCount: n } as unknown as View);
    if (q.text.startsWith("Which is the highest")) asked[q.text] = q.answer;
  }
  expect(asked["Which is the highest trump (♥) not yet played?"]).toBe("J♦");
  expect(asked["Which is the highest ♠ not yet played?"]).toBe("Q♠");
  // Diamonds and clubs: nothing higher has gone, so no question about them.
  expect(Object.keys(asked)).toHaveLength(2);
  const spades = makeQuestion({ ...base, actionCount: Object.keys(asked).length ? findAction(base, "♠") : 0 } as unknown as View);
  expect(spades.options).toEqual(["A♠", "K♠", "Q♠", "J♠"]);
  expect(spades.explain).toContain("in your hand");

  // The last trick (one card left) is never quizzed; more turns are at "high" than at "low".
  const due = (cards: number, rate: "low" | "medium" | "high") =>
    Array.from({ length: 400 }, (_, n) => quizDue({ ...base, hand: base.hand.slice(0, cards), actionCount: n } as unknown as View, rate)).filter(Boolean).length;
  expect(due(1, "high")).toBe(0);
  expect(due(3, "low")).toBeLessThan(due(3, "medium"));
  expect(due(3, "medium")).toBeLessThan(due(3, "high"));

  function findAction(b: typeof base, symbol: string): number {
    for (let n = 0; n < 200; n++) if (makeQuestion({ ...b, actionCount: n } as unknown as View).text.includes(`highest ${symbol}`)) return n;
    return -1;
  }
});
