import { expect, test } from "@playwright/test";
import { freshStart, takeTurn } from "./helpers";

test("pause after the bidding: every call waits for Continue, and the bid shows in its tile", async ({ page }, info) => {
  test.setTimeout(120_000);
  await freshStart(page, { pauseAfterBidding: true });
  await page.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).getByRole("button", { name: "Deal" }).click();
  const summary = page.locator(".auction-summary");
  for (let step = 0; step < 300 && !(await summary.isVisible()); step++) {
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await expect(summary).toBeVisible();
  await expect(summary.locator(".auction-row")).toHaveCount(4);
  await expect(summary).toContainText(/plays \d/);
  await page.screenshot({ path: `test-results/${info.project.name}-bidding-pause.png` });
  // Nothing moves while it waits: no discard to make, no card led.
  await page.waitForTimeout(1500);
  await expect(page.locator(".discard-bar")).toHaveCount(0);
  await expect(page.locator(".trick-area .card")).toHaveCount(0);
  await expect(summary).toBeVisible();
  await summary.getByRole("button", { name: "Continue" }).click();
  await expect(summary).toHaveCount(0);
  await expect(page.locator(".discard-bar, .trick-area .card").first()).toBeVisible({ timeout: 15_000 });
  // The bid in its tile, with trumps named.
  const tile = page.locator(".bid-tile").first();
  await expect(tile).toBeVisible();
  await expect(tile).toContainText(/Trumps|No trumps/);
});

test("hearts and diamonds read red in calls and bids", async ({ page }) => {
  await freshStart(page);
  await page.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).getByRole("button", { name: "Deal" }).click();
  await page.locator(".table").waitFor();
  // A heart in a call bubble (light) and on the felt: red either way, and a spade isn't.
  const colors = await page.evaluate(() => {
    const make = (parent: Element, cls: string) => {
      const span = document.createElement("span");
      span.className = `suit-sym suit-${cls}`;
      span.textContent = "x";
      parent.appendChild(span);
      return getComputedStyle(span).color;
    };
    const bubble = document.createElement("div");
    bubble.className = "bubble";
    document.querySelector(".table")!.appendChild(bubble);
    return { felt: make(document.querySelector(".table")!, "H"), bubble: make(bubble, "D"), spade: make(bubble, "S") };
  });
  const rgb = (c: string) => c.match(/\d+/g)!.map(Number);
  const [r, g, b] = rgb(colors.felt);
  expect(r).toBeGreaterThan(g + 80);
  expect(r).toBeGreaterThan(b + 80);
  const [r2, g2] = rgb(colors.bubble);
  expect(r2).toBeGreaterThan(g2 + 80);
  const [r3, g3] = rgb(colors.spade);
  expect(Math.abs(r3 - g3)).toBeLessThan(40);
});
