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
  await page.getByRole("button", { name: "Coach", exact: true }).click();
  const coachDialog = page.getByRole("dialog", { name: "Coach" });
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
