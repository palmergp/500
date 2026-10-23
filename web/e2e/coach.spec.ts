import { expect, test, type Page } from "@playwright/test";
import { startGame, takeTurn } from "./helpers";

/** On your turn: ask for a hint, then play the hinted card or call when there is one. */
async function followHint(page: Page): Promise<boolean> {
  const hintButton = page.locator(".hint-button:not([disabled])");
  try {
    await hintButton.click({ timeout: 1000 });
  } catch {
    return takeTurn(page); // no hint available right now (not our turn, or the table moved on)
  }
  const panel = page.getByRole("note", { name: "Coach's hint" });
  try {
    await panel.waitFor({ timeout: 20_000 });
  } catch {
    return takeTurn(page);
  }
  const hinted = page.locator(".hand .card.hinted, .bidding-box .call.hinted");
  if ((await page.locator(".discard-bar").count()) === 0 && (await hinted.count()) > 0) {
    await hinted.first().click({ timeout: 2000 }).catch(() => {});
    const joker = page.locator(".joker-choice .call").first();
    if (await joker.count()) await joker.click();
    return true;
  }
  return takeTurn(page);
}

test("hints, grades and the hand review", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough; layout is covered by play.spec");
  test.setTimeout(240_000);
  await startGame(page);

  // The first hint, and its grade after following it.
  while ((await page.locator(".hint-button").count()) === 0) await page.waitForTimeout(100);
  await page.locator(".hint-button").click();
  const panel = page.getByRole("note", { name: "Coach's hint" });
  await expect(panel).toContainText("Coach suggests");
  await expect(page.locator(".hinted")).toHaveCount(1);
  await page.screenshot({ path: `test-results/coach-1-hint.png` });
  await page.locator(".hinted").click();
  const grade = page.getByRole("status", { name: "Coach's grade" });
  await expect(grade).toBeVisible({ timeout: 20_000 });
  await expect(grade.locator(".verdict")).toHaveText("Best");
  await page.screenshot({ path: `test-results/coach-2-grade.png` });

  // Play the hand out, following hints.
  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  for (let step = 0; step < 2000 && !(await result.isVisible()); step++) {
    if (!(await followHint(page))) await page.waitForTimeout(100);
  }
  await result.getByRole("button", { name: "Review hand" }).click();
  const review = page.getByRole("dialog", { name: "Hand review" });

  await review.getByRole("tab", { name: /Your decisions/ }).click();
  await expect(review.locator(".decision-list > li").first()).toBeVisible({ timeout: 20_000 });
  await page.screenshot({ path: `test-results/coach-3-decisions.png`, fullPage: true });

  await review.getByRole("tab", { name: "Trick by trick" }).click();
  await expect(review.getByText(/With every card visible and perfect play/)).toBeVisible({ timeout: 30_000 });
  await expect(review.locator(".trick-list > li")).toHaveCount(10);
  await page.screenshot({ path: `test-results/coach-4-tricks.png`, fullPage: true });

  await review.getByRole("tab", { name: "Luck vs skill" }).click();
  await review.getByRole("button", { name: /Replay with The Counter/ }).click();
  await expect(review.getByText(/With The Counter in your seat/)).toBeVisible({ timeout: 60_000 });
  await page.screenshot({ path: `test-results/coach-5-luck.png`, fullPage: true });
});

test("the hint panel fits a phone screen", async ({ page }, info) => {
  test.skip(info.project.name !== "phone", "phone layout only");
  await startGame(page);
  const hintButton = page.locator(".hint-button:not([disabled])");
  await hintButton.waitFor({ timeout: 60_000 });
  await hintButton.click();
  await expect(page.getByRole("note", { name: "Coach's hint" })).toBeVisible({ timeout: 20_000 });
  const width = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(width).toBeLessThanOrEqual(0);
  await page.screenshot({ path: `test-results/coach-phone-hint.png`, fullPage: true });
});
