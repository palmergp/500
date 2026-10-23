// The table fits the window (no scrolling at any point of a hand), and the table's aids: who is
// to act, the bid off to the side, trick piles, past tricks, short games and random opponents.

import { expect, test, type Page } from "@playwright/test";
import { freshStart, startGame, takeTurn } from "./helpers";

/** Whether the page scrolls: it never should while playing. */
async function scrolls(page: Page): Promise<string | null> {
  return page.evaluate(() => {
    const el = document.scrollingElement!;
    const tall = el.scrollHeight > window.innerHeight + 1;
    const wide = el.scrollWidth > window.innerWidth + 1;
    return tall || wide ? `page is ${el.scrollWidth}×${el.scrollHeight} in a ${window.innerWidth}×${window.innerHeight} window` : null;
  });
}

test("the main menu, and tooltips on the table's buttons", async ({ page }) => {
  await freshStart(page);
  const menu = page.getByRole("navigation", { name: "Main menu" });
  for (const name of [/^Play solo/, /^Play with friends/, /^Learn/, /^Settings/]) await expect(menu.getByRole("button", { name })).toBeVisible();
  await menu.getByRole("button", { name: /^Settings/ }).click();
  await expect(page.getByRole("dialog", { name: "Settings" }).getByLabel("Music")).toBeVisible();
  await page.getByRole("dialog", { name: "Settings" }).getByRole("button", { name: "Done" }).click();
  await page.screenshot({ path: `test-results/menu-${test.info().project.name}.png` });

  await menu.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).getByRole("button", { name: "Deal" }).click();
  await expect(page.getByRole("button", { name: "Learn" })).toHaveAttribute("title", /lessons/i);
  // Back to the menu and on with the same game.
  await page.getByRole("button", { name: "Menu" }).click();
  await page.getByRole("button", { name: /^Continue/ }).click();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
});

test("a whole hand fits on the screen, with the coach's panels open", async ({ page }, info) => {
  test.setTimeout(240_000);
  await freshStart(page, { tracker: true, grade: true });
  await page.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).getByRole("button", { name: "Deal" }).click();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  expect(await scrolls(page)).toBeNull();

  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  const seen = new Set<string>();
  for (let step = 0; step < 3000; step++) {
    if (await result.isVisible()) break;
    if (await page.locator(".bidding-box").count()) seen.add("bidding-seen");
    // When it's our turn, open the hint first, so the panel is on screen with the bidding box.
    const hint = page.locator(".hint-button:not([disabled])");
    if (await hint.count()) {
      await hint.click({ timeout: 2000 }).catch(() => {});
      await page.getByRole("note", { name: "Coach's hint" }).waitFor({ timeout: 15_000 }).catch(() => {});
    }
    const phase = (await page.locator(".bidding-box").count()) ? "bidding" : (await page.locator(".discard-bar").count()) ? "discard" : (await page.locator(".trick-area .card").count()) ? "play" : "";
    if (phase && !seen.has(phase)) {
      seen.add(phase);
      await page.screenshot({ path: `test-results/fit-${info.project.name}-${phase}.png` });
    }
    const problem = await scrolls(page);
    expect(problem, `while ${phase || "waiting"}`).toBeNull();
    // No call bubble may cover the bidding box.
    const covered = await page.evaluate(() => {
      const box = document.querySelector(".bidding-box")?.getBoundingClientRect();
      if (!box) return null;
      for (const b of document.querySelectorAll(".bubble")) {
        const r = b.getBoundingClientRect();
        if (r.width && r.left < box.right - 1 && r.right > box.left + 1 && r.top < box.bottom - 1 && r.bottom > box.top + 1) return b.textContent;
      }
      return null;
    });
    expect(covered, "a call bubble covers the bidding box").toBeNull();
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  expect(seen.has("bidding-seen") || seen.has("bidding")).toBe(true);
});

test("a full hand of card backs stays inside its seat on laptop screens", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "sizes are set here");
  await startGame(page);
  for (const [width, height] of [[1024, 640], [1280, 720], [1366, 768], [1100, 700], [1536, 864]]) {
    await page.setViewportSize({ width, height });
    await expect(page.locator(".seat-west .backs .card")).toHaveCount(10);
    for (const seat of ["west", "east", "north"]) {
      const box = await page.locator(`.seat-${seat}`).boundingBox();
      const cards = await page.locator(`.seat-${seat} .backs .card`).evaluateAll((els) => els.map((e) => e.getBoundingClientRect()).map((r) => [r.left, r.right]));
      for (const [left, right] of cards) {
        expect(left, `${seat} at ${width}×${height}`).toBeGreaterThanOrEqual(box!.x - 1);
        expect(right, `${seat} at ${width}×${height}`).toBeLessThanOrEqual(box!.x + box!.width + 1);
      }
    }
    // The side seats stay in their columns, clear of the middle of the table.
    const center = (await page.locator(".center").boundingBox())!;
    const west = (await page.locator(".seat-west").boundingBox())!;
    const east = (await page.locator(".seat-east").boundingBox())!;
    expect(west.x + west.width, `West at ${width}×${height}`).toBeLessThanOrEqual(center.x + 1);
    expect(east.x, `East at ${width}×${height}`).toBeGreaterThanOrEqual(center.x + center.width - 1);
    await page.screenshot({ path: `test-results/backs-${width}x${height}.png` });
  }
});

for (const [width, height] of [[1280, 720], [1229, 590], [1366, 657], [1536, 730], [1024, 600]]) test(`with large cards, the cards played to a trick don't overlap (${width}×${height})`, async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "sizes are set here");
  test.setTimeout(180_000);
  await page.setViewportSize({ width, height });
  await freshStart(page, { bigCards: true });
  await page.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).getByRole("button", { name: "Deal" }).click();
  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  let checked = 0;
  for (let step = 0; step < 3000 && checked < 6 && !(await result.isVisible()); step++) {
    const rects = await page.locator(".trick-card .card").evaluateAll((els) => els.map((e) => e.getBoundingClientRect()).map((r) => ({ l: r.left, r: r.right, t: r.top, b: r.bottom })));
    if (rects.length >= 2) {
      checked++;
      for (let i = 0; i < rects.length; i++)
        for (let j = i + 1; j < rects.length; j++) {
          const [a, c] = [rects[i], rects[j]];
          const overlap = Math.min(a.r, c.r) - Math.max(a.l, c.l) > 1 && Math.min(a.b, c.b) - Math.max(a.t, c.t) > 1;
          // West and East cards sit beside North and South's: only a corner may touch.
          const area = (Math.min(a.r, c.r) - Math.max(a.l, c.l)) * (Math.min(a.b, c.b) - Math.max(a.t, c.t));
          expect(overlap && area > 0.15 * (a.r - a.l) * (a.b - a.t), `cards ${i} and ${j} overlap`).toBe(false);
        }
      if (checked === 1) await page.screenshot({ path: `test-results/big-cards-trick-${width}x${height}.png` });
    }
    if (!(await takeTurn(page))) await page.waitForTimeout(50);
  }
  expect(checked).toBeGreaterThan(0);
  expect(await page.evaluate(() => document.scrollingElement!.scrollHeight <= window.innerHeight + 1)).toBe(true);
});

test("trick piles, past tricks, and who called what", async ({ page }) => {
  test.setTimeout(240_000);
  await startGame(page);
  // During the bidding, each call shows by the seat that made it, and the seat to act glows. The page
  // notes when it sees them, so a quick auction can't slip past between checks.
  await page.evaluate(() => {
    const seen: Record<string, boolean> = { bubble: false, glow: false };
    (window as unknown as { seen: Record<string, boolean> }).seen = seen;
    const look = () => {
      if (document.querySelector(".bubble")) seen.bubble = true;
      if (document.querySelector(".seat.to-act")) seen.glow = true;
    };
    look();
    new MutationObserver(look).observe(document.body, { subtree: true, childList: true, attributes: true });
  });

  // Play until a few tricks are in: each trick makes a pile by the side that took it.
  for (let i = 0; i < 2000; i++) {
    const done = await page.locator(".pile").count();
    if (done >= 3) break;
    const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
    if (await result.isVisible()) await result.getByRole("button", { name: /Next hand/ }).click();
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  const seen = await page.evaluate(() => (window as unknown as { seen: Record<string, boolean> }).seen);
  expect(seen).toEqual({ bubble: true, glow: true });
  const piles = await page.locator(".pile").count();
  expect(piles).toBeGreaterThanOrEqual(3);
  await page.screenshot({ path: `test-results/piles-${test.info().project.name}.png` });
  // Click a pile: the tricks so far, one per pile.
  await page.locator(".pile").first().click();
  const tricks = page.getByRole("dialog", { name: "Tricks this hand" });
  await expect(tricks.locator(".past-tricks > li")).toHaveCount(await page.locator(".pile").count(), { timeout: 5000 }).catch(async () => {
    // A trick may have finished meanwhile; the dialog lists at least the piles we saw.
    expect(await tricks.locator(".past-tricks > li").count()).toBeGreaterThanOrEqual(piles);
  });
  await expect(tricks.locator(".past-tricks > li").first().locator(".card")).toHaveCount(4);
  await page.screenshot({ path: `test-results/past-tricks-${test.info().project.name}.png` });
  await tricks.getByRole("button", { name: "Back to the game" }).click();
});

test("a four-hand game against random opponents", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  await startGame(page, { west: "any", north: "any", east: "any", hands: "4 hands" });
  await expect(page.getByText("Hand 1 of 4")).toBeVisible();
  // The random picks are real players, named at the table.
  const names = await page.locator(".seat-name").allInnerTexts();
  expect(names.join(" ")).not.toContain("Random");
});

test("the three kitty discards get one grade", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  test.setTimeout(240_000);
  await startGame(page, { north: "rock", west: "rock", east: "rock" });
  // Bid 10NT at the first chance, so we win the kitty.
  const discard = page.locator(".discard-bar button");
  // Only ever bid 10NT or wait: a general "take a turn" could pass if the box appears mid-check.
  for (let i = 0; i < 600 && !(await discard.count()); i++) {
    const top = page.locator(".bidding-box .call:not([disabled])", { hasText: /^10NT$/ });
    if (await top.count()) await top.click({ timeout: 2000 }).catch(() => {});
    else await page.waitForTimeout(100);
  }
  await expect(discard).toBeVisible();
  for (let i = 0; i < 3; i++) await page.locator(".hand .card.playable:not(.selected)").first().click();
  await discard.click();
  const grade = page.getByRole("status", { name: "Coach's grade" }).filter({ hasText: "Put back" });
  await expect(grade).toBeVisible({ timeout: 60_000 });
  // One grade for the set, naming all three cards (not three grades flashing by).
  const head = await grade.locator(".coach-head").innerText();
  expect(head.split("·")[0].match(/(10|[2-9AKQJ])[♠♣♦♥]|Joker/g)?.length).toBe(3);
  await page.screenshot({ path: "test-results/discard-grade.png" });
});

test("your graded decisions, during the hand", async ({ page }, info) => {
  test.setTimeout(240_000);
  await startGame(page);
  // Play until a few of our decisions are graded and the cards are being played.
  const button = page.getByRole("button", { name: /^Your decisions/ });
  for (let i = 0; i < 2000; i++) {
    const label = (await button.count()) ? await button.first().innerText() : "";
    const graded = Number(/\((\d+)\)/.exec(label)?.[1] ?? 0);
    if (graded >= 3 && (await page.locator(".trick-area").count())) break;
    const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
    if (await result.isVisible()) await result.getByRole("button", { name: /Next hand/ }).click();
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await button.first().click();
  const dialog = page.getByRole("dialog", { name: "Your decisions" });
  const items = dialog.locator(".decision-list > li");
  expect(await items.count()).toBeGreaterThanOrEqual(3);
  // Each says when it was made, and opens to the coach's reasons.
  await expect(items.first().locator(".decision-when")).toHaveText(/Bidding|Kitty|Trick \d+/);
  const first = items.first().locator("details");
  if (!(await first.evaluate((d) => (d as HTMLDetailsElement).open))) await first.locator("summary").click();
  await expect(items.first().locator("ul, .note").first()).toBeVisible();
  await dialog.getByRole("tab", { name: /Whole game/ }).click();
  expect(await dialog.locator(".decision-list > li").count()).toBeGreaterThanOrEqual(3);
  await page.screenshot({ path: `test-results/decisions-${info.project.name}.png` });
  await dialog.getByRole("button", { name: "Back to the game" }).click();
  await expect(dialog).toHaveCount(0);
});
