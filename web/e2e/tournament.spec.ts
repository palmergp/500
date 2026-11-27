// Tournament mode: start one, play a round's four hands, see the round's results and standings,
// and start the next round with a different partner.

import { expect, test } from "@playwright/test";
import { freshStart, takeTurn } from "./helpers";

test("a tournament round: play your table, see the standings, a new partner next round", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(900_000);
  await freshStart(page);
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  // First, who the other 11 are.
  await page.getByRole("dialog", { name: "New tournament" }).getByRole("button", { name: /Random/ }).click();
  const dialog = page.getByRole("dialog", { name: "Tournament" });
  await expect(dialog.locator(".standings tbody tr")).toHaveCount(12);
  await expect(dialog.getByText(/Random tournament/)).toBeVisible();
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
  // The only way on is the next round: nothing closes it (closing it left a table with nothing to do).
  await expect(after.getByRole("button", { name: "Close" })).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(after).toBeVisible();
  // Grading was on, and Finish hand played your cards: that's help.
  await expect(after.getByText(/You've had help in this tournament/)).toBeVisible();
  await page.screenshot({ path: "test-results/tournament-round.png", fullPage: true });
  // Leaving for now: to the menu, and the tournament carries on from there.
  await expect(after.getByRole("button", { name: "Quit", exact: true })).toBeVisible();
  await after.getByRole("button", { name: "Main menu" }).click();
  const menu = page.getByRole("navigation", { name: "Main menu" });
  await expect(menu.getByRole("button", { name: /^Tournament.*Round 2 of 8/ })).toBeVisible();
  await expect(menu.getByRole("button", { name: /^Continue/ })).toHaveCount(0);
  await menu.getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "Tournament" }).getByRole("button", { name: "Play round 2" }).click();

  // Round 2: a different partner.
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  const partner2 = (await page.locator(".seat-north .seat-name").innerText()).replace(/partner|dealer/g, "").trim();
  expect(partner2).not.toBe(partner1);
});

test("kinds: random styles, all experts, or all gamblers", async ({}, info) => {
  test.skip(info.project.name !== "desktop", "pure logic");
  const { newTournament, EXPERT_STYLES } = await import("../src/tournament");
  for (let seed = 1; seed <= 20; seed++) {
    const bots = (kind?: "random" | "expert" | "chaos") => newTournament(seed, kind).participants.slice(1).map((p) => p.persona);
    expect(bots("expert").every((p) => EXPERT_STYLES.includes(p))).toBe(true);
    expect(bots("chaos").every((p) => p === "gambler")).toBe(true);
    expect(bots()).toEqual(bots("random"));
  }
  const t = newTournament(3, "chaos");
  expect(t.kind).toBe("chaos");
  expect(t.assisted).toBe(false);
});

test("an unaided tournament: the dialog says what would count as help, and a hint does", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(120_000);
  // Grading and the trump counter off; out-of-suit marks still on.
  await freshStart(page, { grade: false, trumpCounter: false, tracker: false });
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  const choose = page.getByRole("dialog", { name: "New tournament" });
  await expect(choose.locator(".kind-choices button")).toHaveCount(3);
  await choose.getByRole("button", { name: /Expert/ }).click();
  const dialog = page.getByRole("dialog", { name: "Tournament" });
  await expect(dialog.getByText(/Expert tournament/)).toBeVisible();
  await expect(dialog.locator(".unaided-note")).toHaveText(/Unaided so far.*Out-of-suit marks is on in Settings: turn it off/);
  // Closing it and coming back: the same tournament, no new choice.
  await dialog.getByRole("button", { name: "Close", exact: true }).last().click();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Settings/ }).click();
  await page.getByRole("dialog", { name: "Settings" }).getByText("Show who is out of a suit").click();
  await page.getByRole("dialog", { name: "Settings" }).getByRole("button", { name: "Done" }).click();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  await expect(page.getByRole("dialog", { name: "New tournament" })).toHaveCount(0);
  await expect(dialog.locator(".unaided-note")).toHaveText(/^🙌 Unaided so far: no hints, undos or Finish hand\. Every helper is off\.$/);
  await expect(dialog.getByRole("button", { name: "Turn off all help" })).toHaveCount(0);
  await dialog.getByRole("button", { name: "Play round 1" }).click();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  const hint = page.getByRole("button", { name: "Hint" });
  for (let step = 0; step < 200 && !(await hint.isVisible()); step++) await page.waitForTimeout(100);
  await hint.click();
  await expect(page.getByRole("note", { name: "Coach's hint" })).toBeVisible({ timeout: 30_000 });
  const saved = await page.evaluate(() => JSON.parse(localStorage.getItem("fh500.tournament")!));
  expect(saved.assisted).toBe(true);
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
      const b = 60 + 23 * ((r * 3 + i) % 7) - (r === 2 && i === 1 ? 300 : 0);
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
  // Every player's game in every round: a row each in final order, the round's best starred, and
  // choosing a player lists their games.
  const grid = final.locator(".game-grid table");
  await expect(grid.locator("tbody tr")).toHaveCount(12);
  await expect(grid.locator("thead th")).toHaveCount(2 + ROUNDS + 1);
  const { standings: order } = await import("../src/tournament");
  await expect(grid.locator("tbody tr").first().locator(".game-total")).toHaveText(String(order(t)[0].total));
  await expect(grid.locator(".best-mark")).not.toHaveCount(0);
  // A score's tooltip names the table, then each hand's bid, who declared it and what it scored.
  await expect(grid.locator(".game-cell").first()).toHaveAttribute(
    "title",
    /^Round 1: with .+ against .+ and .+, -?\d+ to -?\d+, (won|lost|tied)\nHand 1: 8♥ by .+, made \((\+\d+|0)\)\nHand 2: 7♠ by .+, made \((\+\d+|0)\)$/,
  );
  await expect(final.getByRole("heading", { name: "Your games" })).toBeVisible();
  await expect(final.locator(".player-games li")).toHaveCount(ROUNDS);
  const other = grid.locator("tbody th button", { hasNotText: "You" }).first();
  const otherName = (await other.innerText()).trim();
  await other.click();
  await expect(final.getByRole("heading", { name: new RegExp(`^${otherName}.*'s games$`) })).toBeVisible();
  await expect(final.locator(".player-games li").first()).toContainText(/^R1/);
  await expect(grid.locator(".game-cell", { hasText: /^-\d+/ })).not.toHaveCount(0);
  await page.waitForTimeout(800);
  await page.screenshot({ path: "test-results/tournament-final.png", fullPage: true });
  // On a phone the grid scrolls sideways inside the dialog; the page doesn't.
  await page.setViewportSize({ width: 393, height: 852 });
  await grid.scrollIntoViewIfNeeded();
  expect(await page.evaluate(() => document.scrollingElement!.scrollWidth - window.innerWidth)).toBeLessThanOrEqual(0);
  // Scrolled to the last round, place, name and total stay put.
  const firstRow = grid.locator("tbody tr").first();
  const totalBefore = (await firstRow.locator(".game-total").boundingBox())!.x;
  await final.locator(".game-grid-scroll").evaluate((el) => (el.scrollLeft = el.scrollWidth));
  expect((await firstRow.locator(".game-total").boundingBox())!.x).toBeCloseTo(totalBefore, 0);
  await page.screenshot({ path: "test-results/tournament-final-phone.png" });
  await page.setViewportSize({ width: 1280, height: 720 });
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

test("your recap: how you bid, where your points came from, and how you played", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  const { newTournament, ROUNDS, yourTable } = await import("../src/tournament");
  const t = newTournament(9);
  t.mine = [];
  t.rounds.forEach((round, r) => {
    round.tables.forEach((table, i) => {
      table.result = { scores: i === yourTable(t, r) ? [210, 90] : [100 + 10 * i, 120], hands: [[260, 0], [-140, 0], [30, 0], [60, 0]] };
    });
    // Each game: your 8♥ made, partner's 7♠ down, West's 7NT made, East's 8♦ beaten.
    t.mine!.push({
      save: "",
      hands: [
        { declarer: 0, label: "8♥", made: true, youBid: true, points: 260 },
        { declarer: 2, label: "7♠", made: false, youBid: false, points: -140 },
        { declarer: 1, label: "7NT", made: true, youBid: false, points: 30 },
        { declarer: 3, label: "8♦", made: false, youBid: false, points: 60 },
      ],
      grades: [
        { action: 1, phase: "bidding", verdict: "best", loss: 0, deal: 1, trick: null, chosen: "8♥", best: "8♥", reason: null },
        { action: 12, phase: "discarding", verdict: "good", loss: 1, deal: 1, trick: null, chosen: "4♣ 5♣ 6♠", best: "4♣ 5♣ 7♠", reason: null },
        { action: 40, phase: "playing", verdict: "mistake", loss: 20 + r, deal: 2, trick: 3, chosen: "9♠", best: "A♠", reason: "Cash the A♠ while it's still good." },
      ],
      graded: true,
    });
  });
  t.current = ROUNDS;
  await freshStart(page);
  await page.evaluate((saved) => localStorage.setItem("fh500.tournament", saved), JSON.stringify(t));
  await page.reload();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "Tournament" }).getByRole("button", { name: "Final results" }).click();
  const final = page.getByRole("dialog", { name: "Tournament results" });
  for (const step of ["Reveal third place", "Reveal second place", "Reveal the winner"]) await final.getByRole("button", { name: step }).click();
  await final.getByRole("button", { name: "Your recap" }).click();
  const recap = page.getByRole("dialog", { name: "Your tournament" });
  await expect(recap.locator(".recap-tile", { hasText: "Games won" })).toContainText("8of 8");
  await expect(recap.locator(".recap-tile", { hasText: "Your bids made" })).toContainText("100%");
  await expect(recap.getByText(/You bid in 8 of 32 hands \(25%\)/)).toBeVisible();
  await expect(recap.getByText(/your side beat 8 of those bids \(50%\)/)).toBeVisible();
  const row = (name: string) => recap.locator(".recap-table tr", { hasText: name });
  await expect(row("Your bids")).toContainText("+2080124%");
  await expect(row("Your partners' bids")).toContainText("−1120−67%");
  await expect(row("Tricks on the opponents' bids")).toContainText("+72043%");
  await expect(recap.getByText(/graded 24 of your decisions: 67% best or good/)).toBeVisible();
  await expect(recap.getByText(/8\.2 points a decision/)).toBeVisible();
  await expect(row("Card play")).toContainText("Card play80%23.58");
  await expect(recap.locator(".recap-costly li").first()).toContainText("Round 8, hand 2, trick 3: you chose 9♠; the coach preferred A♠ (27 points)");
  await page.screenshot({ path: "test-results/tournament-recap.png", fullPage: true });
  await recap.getByRole("button", { name: "Done" }).click();
  // Later, from the menu: the finished tournament's recap is still there.
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "New tournament" }).getByRole("button", { name: "Your last tournament's recap" }).click();
  await expect(page.getByRole("dialog", { name: "Your tournament" }).locator(".recap-tile").first()).toBeVisible();
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

test("the reveal waits for the other tables' last games, and plays them out if they never finished", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(300_000);
  const { newTournament, ROUNDS, yourTable } = await import("../src/tournament");
  const t = newTournament(11);
  t.rounds.forEach((round, r) =>
    round.tables.forEach((table, i) => {
      // The last round: only your table finished (the page was closed while the others played).
      if (r === ROUNDS - 1 && i !== yourTable(t, r)) return;
      table.result = { scores: [150 + 10 * i, 120], hands: [[150 + 10 * i, 0], [0, 120]] };
    }),
  );
  t.current = ROUNDS;
  await freshStart(page);
  await page.evaluate((saved) => localStorage.setItem("fh500.tournament", saved), JSON.stringify(t));
  await page.reload();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "Tournament" }).getByRole("button", { name: "Final results" }).click();
  const final = page.getByRole("dialog", { name: "Tournament results" });
  // Nothing given away while they play.
  await expect(final.getByText("The other tables are finishing their last games…")).toBeVisible();
  await expect(final.locator(".podium-step, .game-grid")).toHaveCount(0);
  const reveal = final.getByRole("button", { name: "Reveal third place" });
  await expect(reveal).toBeEnabled({ timeout: 240_000 });
  await reveal.click();
  await expect(final.locator(".podium-step")).toHaveCount(1);
});

test("turn off all help before a tournament, and quit it during the first game", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  test.setTimeout(120_000);
  await freshStart(page);
  const menu = page.getByRole("navigation", { name: "Main menu" });
  await menu.getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "New tournament" }).getByRole("button", { name: /Random/ }).click();
  const dialog = page.getByRole("dialog", { name: "Tournament" });
  // Grading and the other helpers are on by default: one button turns them all off.
  await dialog.getByRole("button", { name: "Turn off all help" }).click();
  await expect(dialog.getByText(/Every helper is off/)).toBeVisible();
  expect(await page.evaluate(() => JSON.parse(localStorage.getItem("fh500.settings") ?? "{}"))).toMatchObject({ grade: false, trumpCounter: false, tracker: false, voids: false });
  await dialog.getByRole("button", { name: "Play round 1" }).click();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);

  // Mid-game: the standings offer a way out.
  await page.getByRole("button", { name: "Standings" }).click();
  page.once("dialog", (d) => void d.accept());
  await page.getByRole("dialog", { name: "Tournament" }).getByRole("button", { name: "Quit", exact: true }).click();
  await expect(menu).toBeVisible();
  await expect(menu.getByRole("button", { name: /^Tournament/ })).toContainText("12 players");
  await expect(menu.getByRole("button", { name: /^Continue/ })).toHaveCount(0);
});

test("on a phone, the tournament's buttons fit on one row", async ({ page }, info) => {
  test.skip(info.project.name !== "phone", "phone layout only");
  await freshStart(page);
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  await page.getByRole("dialog", { name: "New tournament" }).getByRole("button", { name: /Random/ }).click();
  const dialog = page.getByRole("dialog", { name: "Tournament" });
  // Closed and opened again from the menu: an existing tournament.
  await dialog.getByRole("button", { name: "Close", exact: true }).last().click();
  await page.getByRole("navigation", { name: "Main menu" }).getByRole("button", { name: /^Tournament/ }).click();
  const actions = dialog.locator(".actions").last();
  const buttons = actions.getByRole("button");
  await expect(buttons).toHaveText(["Quit", "Close", "Play round 1"]);
  await actions.scrollIntoViewIfNeeded();
  const boxes = await buttons.evaluateAll((els) => els.map((e) => e.getBoundingClientRect()).map((r) => ({ top: Math.round(r.top), right: r.right })));
  expect(new Set(boxes.map((b) => b.top)).size).toBe(1);
  const width = await dialog.evaluate((d) => d.getBoundingClientRect().right);
  for (const b of boxes) expect(b.right).toBeLessThanOrEqual(width);
  await page.screenshot({ path: `test-results/${info.project.name}-tournament-actions.png` });
});
