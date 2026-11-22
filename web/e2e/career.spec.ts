// Story mode: name yourself, watch Chapter One's opening scene (a title card, typed lines, a
// choice), play its game against the story's characters, and see how the story goes on.

import { expect, test } from "@playwright/test";
import { freshStart, takeTurn } from "./helpers";

test("story mode: a chapter's scene, its game against the story's players, and the story going on", async ({ page }, info) => {
  test.setTimeout(300_000);
  const desktop = info.project.name === "desktop";
  await freshStart(page);
  const menu = page.getByRole("navigation", { name: "Main menu" });
  await menu.getByRole("button", { name: /^Story/ }).click();
  const hub = page.getByRole("dialog", { name: "Story" });
  await hub.getByLabel("What do they call you at the table?").fill("Sam");
  await hub.getByRole("button", { name: "Begin" }).click();
  await expect(hub.getByText("Playing as")).toContainText("Sam");
  // Later chapters stay hidden until you get there.
  await expect(hub.locator(".career-chapter.locked").first()).toContainText("???");
  await hub.locator(".career-chapter").first().getByRole("button", { name: "Play" }).click();

  // The opening scene: a title card, then the diner, typed lines with your name in them.
  const scene = page.getByRole("dialog", { name: "Chapter 1: Tuesday Night" });
  await expect(scene.getByText("Tuesday Night")).toBeVisible();
  await page.screenshot({ path: `test-results/${info.project.name}-career-1-title.png` });
  await scene.getByRole("button", { name: /^Continue/ }).click();
  await expect(scene.locator(".cut-place")).toContainText("Mae's Diner");
  const next = scene.getByRole("button", { name: "Next" });
  for (let i = 0; i < 30 && !(await scene.locator(".cut-text").innerText()).includes("Sam"); i++) await next.click();
  await expect(scene.locator(".cut-name")).toHaveText("Mae Donnelly");
  await expect(scene.locator(".cut-actor.speaking")).toHaveCount(1);
  await page.screenshot({ path: `test-results/${info.project.name}-career-2-scene.png` });
  // Danny arrives; you choose what to say, and it becomes your line.
  const choices = scene.getByRole("group", { name: "What do you say?" });
  for (let i = 0; i < 30 && !(await choices.isVisible()); i++) await next.click();
  await page.screenshot({ path: `test-results/${info.project.name}-career-3-choice.png` });
  await choices.getByRole("button", { name: "You're always late." }).click();
  await expect(scene.locator(".cut-name")).toHaveText("Sam");
  // The scene never makes the page scroll.
  expect(await page.evaluate(() => document.documentElement.scrollHeight <= window.innerHeight + 1)).toBe(true);
  await scene.getByRole("button", { name: /^Skip scene/ }).click();
  await expect(scene).toHaveCount(0);

  // The level: four hands with Danny against Earl and Dot.
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  await expect(page.locator(".seat-north .seat-name")).toContainText("Danny Reyes");
  await expect(page.locator(".seat-west .seat-name")).toContainText("Earl");
  if (!desktop) return;

  const finish = page.getByRole("button", { name: /^Finish hand/ });
  const onward = page.getByRole("button", { name: "Continue the story" });
  let banterSeen = false;
  for (let step = 0; step < 5000 && !(await onward.isVisible()); step++) {
    const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
    if (await result.isVisible()) {
      // After each hand, someone at the table has something to say.
      banterSeen ||= (await result.locator(".career-banter").count()) > 0;
      await result.getByRole("button", { name: "Next hand" }).click().catch(() => {});
      continue;
    }
    if (await finish.isVisible()) {
      await finish.click().catch(() => {});
      continue;
    }
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  expect(banterSeen).toBe(true);
  const won = (await page.locator(".game-over").innerText()).startsWith("You and your partner win");
  await onward.click();

  // Won: the ending scene, then Chapter Two is open. Lost: a short scene, then another try.
  const after = page.getByRole("dialog", { name: "Chapter 1: Tuesday Night" });
  await expect(after).toBeVisible();
  await expect(after.locator(".cut-text")).not.toBeEmpty();
  await after.getByRole("button", { name: /^Skip scene/ }).click();
  await expect(hub).toBeVisible();
  if (won) {
    await expect(hub.locator(".career-chapter").first()).toContainText("✓");
    await expect(hub.locator(".career-chapter").nth(1)).toContainText("Forty Thousand");
  } else {
    await expect(hub.locator(".career-chapter").first().getByRole("button", { name: "Try again" })).toBeVisible();
  }
  // The main menu says where the story's up to.
  await hub.locator(".actions").getByRole("button", { name: "Close" }).click();
  await expect(menu.getByRole("button", { name: /^Story/ })).toContainText(won ? "Chapter 2: Forty Thousand" : "Chapter 1: Tuesday Night");
});
