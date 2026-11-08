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
