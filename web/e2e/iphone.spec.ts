// Added to an iPhone's home screen, the app runs full screen under the status bar and above the
// home bar: the layout must keep clear of both (safe-area insets, simulated here in Chromium), still
// without scrolling the page. And the home-screen tags and icon must be there.

import { expect, test, type Browser } from "@playwright/test";
import { startGame } from "./helpers";

const SHAPES = [
  { name: "portrait", viewport: { width: 393, height: 852 }, insets: { top: 59, bottom: 34, left: 0, right: 0 } },
  { name: "landscape", viewport: { width: 852, height: 393 }, insets: { top: 0, bottom: 21, left: 59, right: 59 } },
];

async function iphone(browser: Browser, shape: (typeof SHAPES)[number]) {
  const context = await browser.newContext({ viewport: shape.viewport, isMobile: true, hasTouch: true, deviceScaleFactor: 3 });
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send("Emulation.setSafeAreaInsetsOverride" as never, { insets: shape.insets } as never);
  return { context, page };
}

for (const shape of SHAPES) {
  test(`iPhone ${shape.name}: the table keeps clear of the notch and the home bar`, async ({ browser, browserName }, info) => {
    test.skip(info.project.name !== "desktop" || browserName !== "chromium", "Chromium simulates the insets");
    const { context, page } = await iphone(browser, shape);
    await startGame(page);
    await expect(page.getByLabel("Your hand").locator(".card")).toHaveCount(10);
    const { width, height } = shape.viewport;
    const { top, bottom, left, right } = shape.insets;
    const bar = await page.locator(".topbar").boundingBox();
    expect(bar!.y).toBeGreaterThanOrEqual(top);
    expect(bar!.x).toBeGreaterThanOrEqual(left - 1);
    // Every card in your hand is inside the safe area.
    for (const box of await page.getByLabel("Your hand").locator(".card").evaluateAll((cards) => cards.map((c) => c.getBoundingClientRect().toJSON()))) {
      expect(box.bottom).toBeLessThanOrEqual(height - bottom + 1);
      expect(box.left).toBeGreaterThanOrEqual(left - 1);
      expect(box.right).toBeLessThanOrEqual(width - right + 1);
    }
    // Still one screen: the page itself doesn't scroll.
    const overflow = await page.evaluate(() => document.scrollingElement!.scrollHeight - window.innerHeight);
    expect(overflow).toBeLessThanOrEqual(0);
    await page.screenshot({ path: `test-results/iphone-${shape.name}.png` });

    // The menu, too.
    await page.getByRole("button", { name: "Menu" }).click();
    await page.getByRole("navigation", { name: "Main menu" }).waitFor();
    const menu = await page.locator(".menu-card").boundingBox();
    expect(menu!.y).toBeGreaterThanOrEqual(top);
    await context.close();
  });
}

test("home-screen tags and icon", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "one pass is enough");
  await page.goto("/");
  await expect(page.locator('meta[name="apple-mobile-web-app-capable"]')).toHaveAttribute("content", "yes");
  await expect(page.locator('meta[name="apple-mobile-web-app-status-bar-style"]')).toHaveAttribute("content", "black-translucent");
  const href = await page.locator('link[rel="apple-touch-icon"]').getAttribute("href");
  const size = await page.evaluate(async (src) => {
    const image = new Image();
    image.src = src!;
    await image.decode();
    return [image.naturalWidth, image.naturalHeight];
  }, href);
  expect(size).toEqual([180, 180]);
  // Each build names its own offline cache, so an installed app picks up a new version.
  const sw = await (await page.request.get("/sw.js")).text();
  expect(sw).toMatch(/const CACHE = "fh500-v2-[0-9a-f]{10}"/);
});

test("touch: press and hold shows a tooltip without pressing; a tap on a label shows its tooltip", async ({ browser, browserName }, info) => {
  test.skip(info.project.name !== "desktop" || browserName !== "chromium", "Chromium sends the touches");
  const { context, page } = await iphone(browser, SHAPES[0]);
  const cdp = await context.newCDPSession(page);
  const centre = async (selector: string) => {
    const box = (await page.locator(selector).first().boundingBox())!;
    return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  };
  const touch = async (selector: string, ms: number) => {
    const point = await centre(selector);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [point] });
    await page.waitForTimeout(ms);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  };
  await startGame(page);
  const tip = page.locator(".touch-tip");

  // Holding the Menu button reads its tooltip, and letting go doesn't open the menu.
  await touch('button[aria-label="Menu"]', 800);
  await expect(tip).toContainText("Main menu");
  await page.waitForTimeout(300);
  await expect(page.getByRole("navigation", { name: "Main menu" })).toHaveCount(0);
  const bubble = (await tip.boundingBox())!;
  expect(bubble.x).toBeGreaterThanOrEqual(0);
  expect(bubble.x + bubble.width).toBeLessThanOrEqual(393);

  // A tap on a player's name plate (no button there) shows their style.
  const blurb = await page.locator(".seat-west").getAttribute("title");
  expect(blurb).toBeTruthy();
  await touch(".seat-west .seat-name", 60);
  await expect(tip).toHaveText(blurb!);

  // The next touch puts it away, and a quick tap on a button still presses it.
  await touch('button[aria-label="Menu"]', 60);
  await expect(tip).toHaveCount(0);
  await page.getByRole("navigation", { name: "Main menu" }).waitFor();

  // In a dialog too: the bubble shows on top of it (dialogs sit in the browser's top layer).
  await page.getByRole("button", { name: /^Play solo/ }).click();
  await page.getByRole("dialog", { name: "New game" }).waitFor();
  await touch('.dialog label.inline[title]', 800);
  await expect(tip).toBeVisible();
  // Opened after the dialog, in the top layer, so it's drawn above it.
  expect(await tip.evaluate((el) => el.matches(":popover-open"))).toBe(true);
  await page.screenshot({ path: "test-results/iphone-touch-tip.png" });
  await context.close();
});
