// Two people at a table hosted by fh-host (started by playwright.config.ts on port 5599), each
// in their own browser: the lobby, a deal, private hands, the coach and a reconnect.

import { expect, test, type Browser, type Page } from "@playwright/test";
import { takeTurn } from "./helpers";

async function join(browser: Browser, name: string, seat: string): Promise<Page> {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto("/");
  const lobby = page.getByRole("dialog", { name: "Table" });
  await expect(lobby).toBeVisible({ timeout: 20_000 });
  await lobby.getByLabel("Your name").fill(name);
  await lobby.getByLabel("Your name").press("Enter");
  await lobby.getByRole("group", { name: "Seats" }).getByLabel(seat, { exact: true }).getByRole("button", { name: "Sit here" }).click();
  await expect(lobby.getByLabel(seat, { exact: true })).toContainText(name);
  return page;
}

async function hand(page: Page): Promise<string[]> {
  return page.getByLabel("Your hand").locator(".card").evaluateAll((cards) => cards.map((c) => c.getAttribute("aria-label") ?? ""));
}

test("two people play at a LAN table", async ({ browser }) => {
  const ann = await join(browser, "Ann", "South");
  const bob = await join(browser, "Bob", "West");

  // Each sees the other in the lobby; pick the bots and the pace, then deal.
  const lobby = ann.getByRole("dialog", { name: "Table" });
  await expect(lobby.getByLabel("West", { exact: true })).toContainText("Bob");
  await lobby.getByLabel("North bot").selectOption("steady");
  await lobby.getByLabel("East bot").selectOption("steady");
  await lobby.getByLabel("fast").click();
  await expect(lobby.getByLabel("fast")).toBeChecked();
  await expect(bob.getByRole("dialog", { name: "Table" }).getByLabel("fast")).toBeChecked();
  await lobby.getByRole("button", { name: "Deal" }).click();

  // Both are dealt ten cards, and no card is in both hands.
  for (const page of [ann, bob]) await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  const [a, b] = [await hand(ann), await hand(bob)];
  expect(a.filter((card) => b.includes(card))).toEqual([]);
  // Bob sees himself as "You" with Ann on his right; Ann sees Bob on her left.
  await expect(ann.locator(".scores")).toContainText("You & Steady");
  await expect(bob.locator(".scores")).toContainText("You & Steady");
  await expect(bob.locator(".scores")).toContainText("Ann");

  // Ask the host's coach for a hint on someone's first turn.
  let hinted = false;
  for (let i = 0; i < 60 && !hinted; i++) {
    for (const page of [ann, bob]) {
      const button = page.getByRole("button", { name: "Hint" });
      if (await button.count()) {
        await button.click();
        await expect(page.getByRole("note", { name: "Coach's hint" })).toBeVisible({ timeout: 30_000 });
        hinted = true;
        break;
      }
    }
    if (!hinted) await ann.waitForTimeout(200);
  }
  expect(hinted).toBe(true);

  // Bob's phone reloads mid-deal: he gets his seat and cards back.
  await bob.reload();
  await expect(bob.getByLabel("Your hand").locator(".card").first()).toBeVisible({ timeout: 20_000 });
  expect((await hand(bob)).every((card) => b.includes(card))).toBe(true);

  // Play the deal out; the table waits until both have seen the result.
  for (let i = 0; i < 600; i++) {
    const results = await Promise.all([ann, bob].map((p) => p.getByRole("button", { name: "Next hand" }).count()));
    if (results.every((n) => n > 0)) break;
    // A passed-out deal has no result dialog; it just redeals.
    for (const page of [ann, bob]) if (!(await page.getByRole("button", { name: "Next hand" }).count())) await takeTurn(page);
    await ann.waitForTimeout(100);
  }
  await ann.getByRole("button", { name: "Next hand" }).click();
  await expect(ann.locator(".status")).toContainText("Waiting for Bob");
  await bob.getByRole("button", { name: "Next hand" }).click();
  for (const page of [ann, bob]) await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10, { timeout: 20_000 });
  await ann.screenshot({ path: "test-results/lan-ann.png" });
  await bob.screenshot({ path: "test-results/lan-bob.png" });

  // End the game from the table menu: everyone is back in the lobby.
  await ann.getByRole("button", { name: "Table" }).click();
  await ann.getByRole("button", { name: "End game…" }).click();
  await ann.getByRole("button", { name: "End game", exact: true }).click();
  await expect(bob.getByRole("dialog", { name: "Table" }).getByRole("button", { name: "Deal" })).toBeVisible();
});
