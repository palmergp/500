import { expect, test, type Page } from "@playwright/test";

/** Takes the human's turn with a simple policy: pass if allowed, otherwise the first legal option. */
async function takeTurn(page: Page): Promise<boolean> {
  const pass = page.locator(".bidding-box .call.pass:not([disabled])");
  if (await pass.count()) {
    await pass.click();
    return true;
  }
  const bid = page.locator(".bidding-box .call:not([disabled])").first();
  if (await bid.count()) {
    await bid.click();
    return true;
  }
  const discard = page.locator(".discard-bar button");
  if (await discard.count()) {
    const needed = Number((await discard.innerText()).split("/")[1]);
    const cards = page.locator(".hand .card.playable:not(.selected)");
    for (let i = 0; i < needed; i++) await cards.first().click();
    await discard.click();
    return true;
  }
  const joker = page.locator(".joker-choice .call").first();
  if (await joker.count()) {
    await joker.click();
    return true;
  }
  const card = page.locator(".hand .card.playable").first();
  if (await card.count()) {
    await card.click();
    return true;
  }
  return false;
}

async function startFastGame(page: Page) {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  const dialog = page.getByRole("dialog", { name: "New game" });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel("fast").check();
  await dialog.getByRole("button", { name: "Deal" }).click();
}

test("plays a hand through to the score", async ({ page }, info) => {
  await startFastGame(page);
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
});

test("restores a game after reload", async ({ page }) => {
  await startFastGame(page);
  const hand = page.getByLabel("Your hand");
  await expect(hand.locator(".card")).toHaveCount(10);
  const before = await hand.locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label")));
  await page.reload();
  await expect(page.getByRole("dialog", { name: "New game" })).toHaveCount(0);
  await expect
    .poll(() => hand.locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label"))))
    .toEqual(before);
});
