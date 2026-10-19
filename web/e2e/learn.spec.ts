import { expect, test } from "@playwright/test";
import { startGame } from "./helpers";

test("lessons, drills and progress", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  test.setTimeout(240_000);
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await startGame(page);
  await page.getByRole("button", { name: "Learn" }).click();
  const learn = page.getByRole("dialog", { name: "Learn" });

  // Lesson 1, answering its quiz correctly (answers: J♦, void, no).
  await learn.getByRole("button", { name: /1\. The cards/ }).click();
  await expect(learn.getByRole("heading", { name: /1\. The cards/ })).toBeVisible();
  await learn.getByRole("button", { name: "J♦", exact: true }).click();
  await learn.getByRole("button", { name: "Play anything: you're void in diamonds" }).click();
  await learn.getByRole("button", { name: "No", exact: true }).click();
  await expect(learn.getByText("Lesson complete.")).toBeVisible();
  await page.screenshot({ path: "test-results/learn-1-lesson.png", fullPage: true });
  await learn.getByRole("button", { name: "← All lessons" }).click();
  await expect(learn.getByText(/1 of \d+ lessons done/)).toBeVisible();

  // Bidding trainer.
  await learn.getByRole("tab", { name: "Bidding trainer" }).click();
  const pass = learn.locator(".bidding-box .call.pass:not([disabled])");
  const anyCall = learn.locator(".bidding-box .call:not([disabled])").first();
  await expect(anyCall).toBeVisible({ timeout: 60_000 });
  if (await pass.count()) await pass.click();
  else await anyCall.click();
  await expect(learn.getByRole("button", { name: "Next hand" })).toBeVisible({ timeout: 30_000 });
  await page.screenshot({ path: "test-results/learn-2-bidding.png", fullPage: true });

  // Play puzzle.
  await learn.getByRole("tab", { name: "Play puzzles" }).click();
  const playable = learn.locator(".hand .card.playable").first();
  await expect(playable).toBeVisible({ timeout: 120_000 });
  await page.screenshot({ path: "test-results/learn-3-puzzle.png", fullPage: true });
  await playable.click();
  const joker = learn.locator(".joker-choice .call").first();
  if (await joker.count()) await joker.click();
  await expect(learn.getByRole("button", { name: "Next puzzle" })).toBeVisible({ timeout: 30_000 });
  await page.screenshot({ path: "test-results/learn-4-puzzle-answer.png", fullPage: true });

  // Progress records the drills.
  await learn.getByRole("tab", { name: "Progress" }).click();
  await expect(learn.getByText(/bidding trainer \(1\)/)).toBeVisible();
  await expect(learn.getByText(/play puzzles \(1\)/)).toBeVisible();
  await page.screenshot({ path: "test-results/learn-5-progress.png", fullPage: true });
  expect(errors).toEqual([]);
});

test("the counting quiz and void markers during play", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  test.setTimeout(240_000);
  const { takeTurn } = await import("./helpers");
  await startGame(page);
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  const coachDialog = page.getByRole("dialog", { name: "Settings" });
  await coachDialog.getByLabel(/Counting quiz/).check();
  await coachDialog.getByRole("button", { name: "Done" }).click();

  const quiz = page.getByRole("dialog", { name: "Counting quiz" });
  let answered = false;
  let sawVoid = false;
  for (let step = 0; step < 3000 && !answered; step++) {
    if (!sawVoid && (await page.locator(".void-chip").count()) > 0) {
      sawVoid = true;
      await page.screenshot({ path: "test-results/learn-6-voids.png" });
    }
    if (await quiz.isVisible()) {
      await page.screenshot({ path: "test-results/learn-7-quiz.png" });
      await quiz.locator(".quiz-options button").first().click();
      await expect(quiz.getByText(/Right!|Not quite\./)).toBeVisible();
      await quiz.getByRole("button", { name: "Continue" }).click();
      answered = true;
      break;
    }
    const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
    if (await result.isVisible()) await result.getByRole("button", { name: /Next hand|New game/ }).click();
    else if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  expect(answered).toBe(true);
});

test("your patterns: recurring slips, without decisions made after a hint", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  test.setTimeout(240_000);
  const { freshStart, takeTurn } = await import("./helpers");
  await freshStart(page);
  // A record with two kinds of slip; overbidding cost more.
  await page.evaluate(() => {
    const game = (id: string, patterns: Record<string, { slips: number; loss: number }>) => ({
      id, date: "2026-10-18", finished: true, won: false, decisions: 20,
      verdicts: { best: 12, good: 4, inaccuracy: 2, mistake: 1, blunder: 1 }, loss: 200, lossByPhase: {}, decisionsByPhase: {}, hinted: 3, patterns,
    });
    localStorage.setItem("fh500.progress", JSON.stringify({
      games: [game("a", { overbid: { slips: 2, loss: 180 }, overtake: { slips: 1, loss: 30 } }), game("b", { overtake: { slips: 2, loss: 50 } })],
      quiz: { asked: 0, correct: 0 }, bidding: { asked: 0, correct: 0 }, puzzles: { asked: 0, correct: 0 },
    }));
  });
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Learn/ }).click();
  const learn = page.getByRole("dialog", { name: "Learn" });
  await learn.getByRole("tab", { name: "Progress" }).click();
  const leaks = learn.locator(".leaks > li");
  await expect(leaks).toHaveCount(2);
  await expect(leaks.first()).toContainText("Bidding too high");
  await expect(leaks.nth(1)).toContainText("Overtaking your partner");
  await expect(learn.getByText(/leaving out 6 decisions made after a hint/)).toBeVisible();
  await page.screenshot({ path: "test-results/learn-8-patterns.png", fullPage: true });
  await leaks.first().getByRole("button", { name: /^Lesson:/ }).click();
  await expect(learn.getByRole("heading", { name: /3\. Counting your hand/ })).toBeVisible();
  await learn.getByRole("button", { name: "Close" }).click();

  // A decision made after a hint is marked, and kept out of the record.
  await page.evaluate(() => localStorage.removeItem("fh500.progress"));
  await page.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).getByRole("button", { name: "Deal" }).click();
  for (let i = 0; i < 400 && !(await page.locator(".hint-button:not([disabled])").count()); i++) await page.waitForTimeout(100);
  await page.locator(".hint-button").click();
  await expect(page.getByRole("note", { name: "Coach's hint" })).toBeVisible({ timeout: 30_000 });
  await takeTurn(page);
  await expect(page.getByRole("status", { name: "Coach's grade" })).toBeVisible({ timeout: 30_000 });
  const stored = await page.evaluate(() => ({
    grades: JSON.parse(localStorage.getItem("fh500.grades") ?? "[]") as { hinted?: boolean }[],
    progress: JSON.parse(localStorage.getItem("fh500.progress") ?? "{}") as { games?: { decisions: number; hinted?: number }[] },
  }));
  expect(stored.grades[0]?.hinted).toBe(true);
  const record = stored.progress.games?.at(-1);
  expect(record?.hinted).toBe(1);
  expect(record?.decisions).toBe(0);
});

test("your mistakes come back as puzzles", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  const { startGame } = await import("./helpers");
  await startGame(page);
  // Wait for our first call, and keep that position as if it had been a mistake.
  await page.locator(".bidding-box").waitFor({ timeout: 30_000 });
  await page.evaluate(() => {
    const save = localStorage.getItem("fh500.save")!;
    localStorage.setItem(
      "fh500.mistakes",
      JSON.stringify([{ save, phase: "bidding", chosenLabel: "9♦", bestLabel: "8♦", verdict: "blunder", date: "2026-10-18" }]),
    );
  });
  await page.getByRole("button", { name: "Learn" }).click();
  const learn = page.getByRole("dialog", { name: "Learn" });
  await learn.getByRole("tab", { name: "Your mistakes" }).click();
  await expect(learn.getByText(/You played 9♦, graded a blunder/)).toBeVisible();
  await learn.locator(".bidding-box .call.pass:not([disabled]), .bidding-box .call:not([disabled])").first().click();
  await expect(learn.getByRole("button", { name: "Next mistake" })).toBeVisible({ timeout: 30_000 });
  await page.screenshot({ path: "test-results/learn-9-mistakes.png" });
});
