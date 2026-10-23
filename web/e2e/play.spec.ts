import { writeFileSync } from "node:fs";
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
  // Behind the result is the hand just finished (every trick taken, no cards left), not the next deal.
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(0);
  await expect(page.locator(".pile")).toHaveCount(10);
  await page.screenshot({ path: `test-results/${info.project.name}-3-result.png` });

  // The score pad lists the hand, and the review shows all four hands.
  await result.getByRole("button", { name: "Review hand" }).click();
  const review = page.getByRole("dialog", { name: "Hand review" });
  await expect(review.locator(".review-hand")).toHaveCount(6);
  // Each bid with its bidder's count and the coach's chance when they bid.
  await expect(review.locator(".bid-notes li").first()).toContainText(/bid .+: a count of \d+\.\d/);
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

test("finish the hand: a bot plays your cards quickly to the end", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(120_000);
  await startGame(page);
  // The speed button cycles the pace without opening the settings.
  const speed = page.getByRole("button", { name: /^Speed:/ });
  await expect(speed).toHaveAccessibleName("Speed: fast");
  await speed.click();
  await expect(speed).toHaveAccessibleName("Speed: slow");
  await speed.click();
  await expect(speed).toHaveAccessibleName("Speed: normal");
  await speed.click();
  // Bid and discard as usual, then finish once the cards are being played.
  const finish = page.getByRole("button", { name: /^Finish hand/ });
  for (let step = 0; step < 500 && !(await finish.isVisible()); step++) {
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await finish.click();
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();
  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  await expect(result).toBeVisible({ timeout: 60_000 });
  // The hand counts toward your records and bid stats (Learn → Progress).
  await result.getByRole("button", { name: "Next hand" }).click();
  await page.getByRole("button", { name: "Learn" }).click();
  await page.getByRole("tab", { name: "Progress" }).click();
  await expect(page.getByRole("heading", { name: "Records" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Your bids" })).toBeVisible();
});

test("a hand code, and replaying a hand from its review", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(180_000);
  await startGame(page);
  // The code button names this exact moment (here with no host: a long code carrying the game).
  await page.getByRole("button", { name: "Code" }).click();
  const codeDialog = page.getByRole("dialog", { name: "Hand code" });
  const code = await codeDialog.getByLabel("Hand code").inputValue();
  expect(code).toMatch(/^(500:[A-Za-z0-9_-]+|[0-9A-Z]{4}-[0-9A-Z]{4})$/);
  writeFileSync("test-results/hand-code.txt", code);
  await codeDialog.getByRole("button", { name: "Done" }).click();

  const finish = page.getByRole("button", { name: /^Finish hand/ });
  for (let step = 0; step < 500 && !(await finish.isVisible()); step++) {
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await finish.click();
  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  await expect(result).toBeVisible({ timeout: 60_000 });
  const scores = (await page.locator(".scores").textContent()) ?? "";
  await result.getByRole("button", { name: "Review hand" }).click();
  const review = page.getByRole("dialog", { name: "Hand review" });
  // What partner and the kitty added, against the count's constant.
  await review.getByRole("tab", { name: "Trick by trick" }).click();
  await expect(review.getByRole("heading", { name: "What partner and the kitty added" })).toBeVisible({ timeout: 30_000 });
  await review.getByRole("tab", { name: "Overview" }).click();
  await review.getByRole("button", { name: "Replay this hand" }).click();
  // The replay starts from the deal: a full hand again, and a reminder that it doesn't count.
  await expect(page.locator(".replay-banner")).toBeVisible();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  await page.locator(".replay-banner").getByRole("button", { name: "Back to the game" }).click();
  await expect(page.locator(".replay-banner")).toHaveCount(0);
  await expect(page.locator(".scores")).toHaveText(scores);
});
