import { expect, test } from "@playwright/test";
import { takeTurn } from "./helpers";

test("installs and plays offline", async ({ page, context }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  await page.goto("/");
  // The manifest is linked and parses.
  const manifest = await page.evaluate(async () => {
    const href = document.querySelector('link[rel="manifest"]')?.getAttribute("href");
    return href ? ((await (await fetch(href)).json()) as { name: string; icons: unknown[] }) : null;
  });
  expect(manifest?.name).toContain("500");
  expect(manifest?.icons.length).toBeGreaterThan(0);

  // Wait for the service worker to install, precache and take control.
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready;
  });
  await page.reload();
  await expect.poll(() => page.evaluate(() => !!navigator.serviceWorker.controller)).toBe(true);

  // Now go offline and start a fresh game: the page, workers and WebAssembly engine come from the cache.
  await context.setOffline(true);
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
    localStorage.setItem("fh500.settings", JSON.stringify({ speed: "fast" }));
  });
  await page.reload();
  await page.getByRole("button", { name: /^Play solo/ }).click({ timeout: 20_000 });
  const dialog = page.getByRole("dialog", { name: "New game" });
  await dialog.getByRole("button", { name: "Deal" }).click();
  await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
  // The bots move, so the engine is running.
  for (let i = 0; i < 40; i++) {
    if (await page.locator(".contract").count()) break;
    if (!(await takeTurn(page))) await page.waitForTimeout(150);
  }
  await expect(page.locator(".contract")).toBeVisible({ timeout: 30_000 });
  await context.setOffline(false);
});

test("display options and screen-reader announcements", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass on desktop");
  const { startGame, takeTurn } = await import("./helpers");
  await startGame(page);
  const widthBefore = await page.locator(".hand .card").first().evaluate((el) => el.getBoundingClientRect().width);
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Settings" });
  await dialog.getByLabel(/Four-color deck/).check();
  await dialog.getByLabel(/Larger cards/).check();
  await dialog.getByRole("button", { name: "Done" }).click();
  const widthAfter = await page.locator(".hand .card").first().evaluate((el) => el.getBoundingClientRect().width);
  expect(widthAfter).toBeGreaterThan(widthBefore);
  // A diamond, if we hold one, is blue.
  const diamond = page.locator(".hand .card.suit-D").first();
  if (await diamond.count()) {
    expect(await diamond.evaluate((el) => getComputedStyle(el).color)).toBe("rgb(21, 101, 192)");
  }
  // Announcements reach the live region.
  const live = page.locator("[aria-live=polite].visually-hidden");
  for (let i = 0; i < 40 && !(await live.innerText()).trim(); i++) {
    if (!(await takeTurn(page))) await page.waitForTimeout(150);
  }
  await expect(live).not.toBeEmpty();
  await page.screenshot({ path: "test-results/display-options.png" });
});
