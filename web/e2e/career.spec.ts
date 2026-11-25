// Story mode: name yourself, watch Chapter One's opening scene (a title card, typed lines, a
// choice), play its game against the story's characters, and see how the story goes on.

import { expect, test } from "@playwright/test";
import { freshStart, takeTurn } from "./helpers";

test("story mode: a chapter's scene, its game against the story's players, and the story going on", async ({ page }, info) => {
  test.setTimeout(300_000);
  const desktop = info.project.name === "desktop";
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
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
  // The scene has its own music and sound, each with a switch.
  await expect(scene.getByRole("button", { name: "Story music" })).toHaveAttribute("aria-pressed", "true");
  await expect(scene.getByRole("button", { name: "Sound" })).toHaveAttribute("aria-pressed", "true");
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
  // Turning the music off keeps it off (it's a setting).
  await scene.getByRole("button", { name: "Story music" }).click();
  await expect(scene.getByRole("button", { name: "Story music" })).toHaveAttribute("aria-pressed", "false");
  expect(await page.evaluate(() => JSON.parse(localStorage.getItem("fh500.settings") ?? "{}").storyMusic)).toBe(false);
  await scene.getByRole("button", { name: /^Skip scene/ }).click();
  await expect(scene).toHaveCount(0);
  expect(errors).toEqual([]);

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
  // Grading was on (and Finish hand used), so the chapter can't count as unaided; the list says why.
  await expect(hub.locator(".career-unaided-note")).toContainText("Right now");
  if (won) {
    const first = hub.locator(".career-chapter").first();
    await expect(first).toContainText("✓");
    await expect(first.getByRole("img", { name: "beaten unaided" })).toHaveCount(0);
    await expect(hub.locator(".career-chapter").nth(1)).toContainText("Forty Thousand");
    // A finished chapter's scenes can be watched again, without playing it.
    await first.getByRole("button", { name: "Ending" }).click();
    const again = page.getByRole("dialog", { name: "Chapter 1: Tuesday Night" });
    await expect(again.locator(".cut-text")).not.toBeEmpty();
    await again.getByRole("button", { name: /^Skip scene/ }).click();
    await expect(hub).toBeVisible();
    await expect(page.getByLabel("Your hand")).toHaveCount(0);
  } else {
    await expect(hub.locator(".career-chapter").first().getByRole("button", { name: "Try again" })).toBeVisible();
  }
  // The main menu says where the story's up to.
  await hub.locator(".actions").getByRole("button", { name: "Close" }).click();
  await expect(menu.getByRole("button", { name: /^Story/ })).toContainText(won ? "Chapter 2: Forty Thousand" : "Chapter 1: Tuesday Night");
});

test("story: chapters beaten unaided are marked, and the story has its achievements", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  await freshStart(page);
  // Two chapters done, the first with no help at all.
  await page.evaluate(() => localStorage.setItem("fh500.career", JSON.stringify({ name: "Sam", done: [0, 1], unaided: [0], playing: null })));
  await page.reload();
  const menu = page.getByRole("navigation", { name: "Main menu" });
  await expect(menu.getByRole("button", { name: /^Story/ })).toContainText("🙌 1 of 10 unaided");
  await menu.getByRole("button", { name: /^Story/ }).click();
  const hub = page.getByRole("dialog", { name: "Story" });
  const chapters = hub.locator(".career-chapter");
  await expect(chapters.nth(0).getByRole("img", { name: "beaten unaided" })).toBeVisible();
  await expect(chapters.nth(1).getByRole("img", { name: "beaten unaided" })).toHaveCount(0);
  await expect(chapters.nth(0).getByRole("button", { name: "Opening" })).toBeVisible();
  await page.screenshot({ path: `test-results/${info.project.name}-story-list.png` });
  // The opening plays and leads back to the list.
  await chapters.nth(0).getByRole("button", { name: "Opening" }).click();
  const scene = page.getByRole("dialog", { name: "Chapter 1: Tuesday Night" });
  await expect(scene.getByText("Tuesday Night")).toBeVisible();
  await scene.getByRole("button", { name: /^Skip scene/ }).click();
  await expect(hub).toBeVisible();
  await hub.locator(".actions").getByRole("button", { name: "Close" }).click();

  // Learn → Achievements: the Story group counts the chapters done and the unaided one.
  await menu.getByRole("button", { name: /^Learn/ }).click();
  await page.getByRole("tab", { name: "Achievements" }).click();
  await expect(page.getByRole("heading", { name: "Story" })).toBeVisible();
  const achievement = (title: string) => page.locator(".achievement").filter({ hasText: title });
  await expect(achievement("Tuesday regular")).toHaveClass(/done/);
  await expect(achievement("Square with the house")).toHaveClass(/done/);
  await expect(achievement("Past the velvet rope")).not.toHaveClass(/done/);
  await expect(achievement("Clean hands")).toContainText("1 of 10");
});
