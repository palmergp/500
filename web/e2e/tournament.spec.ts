// Tournament mode: start one, play a round's four hands, see the round's results and standings,
// and start the next round with a different partner.

import { expect, test } from "@playwright/test";
import { freshStart, takeTurn } from "./helpers";

test("a tournament round: play your table, see the standings, a new partner next round", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(900_000);
  await freshStart(page);
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  const dialog = page.getByRole("dialog", { name: "Tournament" });
  await expect(dialog.locator(".standings tbody tr")).toHaveCount(12);
  await expect(dialog.getByText(/Round 1 of 8: you partner/)).toBeVisible();
  await dialog.getByRole("button", { name: "Play round 1" }).click();

  // Your table: the players have names, and the standings are a click away.
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  const partner1 = (await page.locator(".seat-north .seat-name").innerText()).replace(/partner|dealer/g, "").trim();
  expect(partner1).not.toMatch(/^The /);
  await expect(page.getByRole("button", { name: "Standings" })).toBeVisible();

  // Four hands, quickly.
  const finish = page.getByRole("button", { name: /^Finish hand/ });
  const roundResults = page.getByRole("button", { name: "Round results" });
  for (let step = 0; step < 5000 && !(await roundResults.isVisible()); step++) {
    const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
    if (await result.isVisible()) {
      const next = result.getByRole("button", { name: "Next hand" });
      if (await next.isVisible()) await next.click();
      continue;
    }
    if (await finish.isVisible()) {
      await finish.click().catch(() => {});
      continue;
    }
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await roundResults.click();

  // The round's results: every table (the other two played out by the engine), then the standings.
  const after = page.getByRole("dialog", { name: "Tournament: round 1 of 8" });
  await expect(after.getByRole("heading", { name: "This round's tables" })).toBeVisible();
  const next = after.getByRole("button", { name: "Next round" });
  await expect(next).toBeEnabled({ timeout: 600_000 });
  await expect(after.locator(".round-tables li")).toHaveCount(3);
  await expect(after.getByText(/High game:/)).toBeVisible();
  await expect(after.locator(".award", { hasText: "High hand:" })).toContainText(/(on|defending) .+ bid/);
  await expect(after.getByText(/You scored/)).toBeVisible();
  await page.screenshot({ path: "test-results/tournament-round.png", fullPage: true });
  await next.click();

  // Round 2: a different partner.
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  const partner2 = (await page.locator(".seat-north .seat-name").innerText()).replace(/partner|dealer/g, "").trim();
  expect(partner2).not.toBe(partner1);
});

test("the schedule: 8 rounds, a different partner every round, you always South", async ({}, info) => {
  test.skip(info.project.name !== "desktop", "pure logic");
  const { newTournament } = await import("../src/tournament");
  for (let seed = 1; seed <= 50; seed++) {
    const t = newTournament(seed);
    expect(t.participants).toHaveLength(12);
    const partners = new Set<string>();
    for (const round of t.rounds) {
      const seen = new Set<number>();
      for (const table of round.tables) {
        for (const id of table.seats) seen.add(id);
        for (const [a, b] of [[table.seats[0], table.seats[2]], [table.seats[1], table.seats[3]]]) {
          const key = [a, b].sort((x, y) => x - y).join("-");
          expect(partners.has(key), `seed ${seed}: ${key} partnered twice`).toBe(false);
          partners.add(key);
        }
        if (table.seats.includes(0)) expect(table.seats[0]).toBe(0);
      }
      expect(seen.size).toBe(12);
    }
    expect(t.rounds).toHaveLength(8);
  }
});

test("the final results: third, second, then the winner, then the awards", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  const { newTournament, ROUNDS } = await import("../src/tournament");
  const t = newTournament(7);
  t.rounds.forEach((round, r) =>
    round.tables.forEach((table, i) => {
      const a = 100 + 37 * ((r + i) % 5);
      const b = 60 + 23 * ((r * 3 + i) % 7);
      table.result = { scores: [a, b], hands: [[a, 0], [0, b]], bids: [{ seat: 2, label: "8♥", made: true }, { seat: 3, label: "7♠", made: true }] };
    }),
  );
  t.current = ROUNDS;
  await freshStart(page);
  await page.evaluate((saved) => localStorage.setItem("fh500.tournament", saved), JSON.stringify(t));
  await page.reload();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "Tournament" }).getByRole("button", { name: "Final results" }).click();
  const final = page.getByRole("dialog", { name: "Tournament results" });
  await expect(final.locator(".podium-step")).toHaveCount(0);
  await final.getByRole("button", { name: "Reveal third place" }).click();
  await expect(final.locator(".podium-step")).toHaveCount(1);
  await final.getByRole("button", { name: "Reveal second place" }).click();
  await final.getByRole("button", { name: "Reveal the winner" }).click();
  await expect(final.locator(".podium-step.place-1")).toBeVisible();
  await expect(final.getByText(/High game:/)).toBeVisible();
  // The high hand names its bid and who made it.
  await expect(final.locator(".award", { hasText: "High hand:" })).toContainText(/on .+'s (8♥|7♠) bid/);
  await page.waitForTimeout(800);
  await page.screenshot({ path: "test-results/tournament-final.png", fullPage: true });
  await final.getByRole("button", { name: "Done" }).click();
  await expect(page.getByRole("navigation", { name: "Main menu" })).toBeVisible();

  // Counted in your stats, and it's an achievement.
  await expect(page.locator(".record-toast")).toContainText("Achievement: Tournament player");
  const { standings } = await import("../src/tournament");
  const you = standings(t).find((s) => s.participant.id === 0)!;
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Learn/ }).click();
  await page.getByRole("tab", { name: /Progress/ }).click();
  await expect(page.locator(".stat", { hasText: "tournaments won" })).toContainText(`${you.place === 1 ? 1 : 0} of 1`);
  await expect(page.locator(".stat", { hasText: "average place" })).toContainText(`${you.place}.0`);
  await expect(page.locator(".stat", { hasText: "highest tournament score" })).toContainText(String(you.total));
  await page.getByRole("tab", { name: "Achievements" }).click();
  await expect(page.locator(".achievement.done", { hasText: "Tournament player" })).toBeVisible();

  // A reload doesn't count it again.
  await page.reload();
  const records = await page.evaluate(() => JSON.parse(localStorage.getItem("fh500.records")!).tournaments.played);
  expect(records).toBe(1);
});

test("the best hand record says which bid it was", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  await freshStart(page);
  const records = { highGame: 420, bestHand: { points: 300, label: "9♠", declarer: 1, made: false }, highBid: null, gamesPlayed: 1, gamesWon: 1, bids: {}, counted: [] };
  await page.evaluate((r) => localStorage.setItem("fh500.records", r), JSON.stringify(records));
  await page.reload();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Learn/ }).click();
  await page.getByRole("tab", { name: /Progress/ }).click();
  await expect(page.locator(".stat", { hasText: "+300" })).toContainText("best hand: defending West's 9♠ bid");
});
