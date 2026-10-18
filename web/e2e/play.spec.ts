import { expect, test } from "@playwright/test";
import { startGame, takeTurn } from "./helpers";

test("plays a hand through to the score", async ({ page }, info) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await startGame(page);
  // Turn music on too, so the audio engine is exercised (sound effects are on by default).
  await page.getByRole("button", { name: "Music" }).click();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  await page.screenshot({ path: `test-results/${info.project.name}-1-deal.png` });

  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  let shotPlay = false;
  let shotBidding = false;
  for (let step = 0; step < 2000 && !(await result.isVisible()); step++) {
    if (!shotBidding && (await page.locator(".bidding-box").count())) {
      await page.screenshot({ path: `test-results/${info.project.name}-1b-bidding.png` });
      shotBidding = true;
    }
    if (!shotPlay && (await page.locator(".trick-area .card").count()) >= 2) {
      await page.screenshot({ path: `test-results/${info.project.name}-2-trick.png` });
      shotPlay = true;
    }
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await expect(result).toBeVisible();
  await page.screenshot({ path: `test-results/${info.project.name}-3-result.png` });

  // The score pad lists the hand, and the review shows all four hands.
  await result.getByRole("button", { name: "Review hand" }).click();
  const review = page.getByRole("dialog", { name: "Hand review" });
  await expect(review.locator(".review-hand")).toHaveCount(6);
  await page.screenshot({ path: `test-results/${info.project.name}-4-review.png`, fullPage: true });
  expect(errors).toEqual([]);
});

test("restores a game after reload", async ({ page }) => {
  await startGame(page);
  const hand = page.getByLabel("Your hand");
  await expect(hand.locator(".card")).toHaveCount(10);
  const before = await hand.locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label")));
  await page.reload();
  await expect(page.getByRole("dialog", { name: "New game" })).toHaveCount(0);
  await expect
    .poll(() => hand.locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label"))))
    .toEqual(before);
});
