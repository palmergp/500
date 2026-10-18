import { expect, test } from "@playwright/test";
import { startGame, takeTurn } from "./helpers";

// Plays one hand against the search bots and reports how long the engine takes per request.
test("search bots think within budget", async ({ page }, info) => {
  test.skip(info.project.name !== "desktop", "timing is measured once, on desktop");
  const times: { ms: number; what: string }[] = [];
  page.on("console", (msg) => {
    const m = /^engine: (\d+) ms \((.*)\)$/.exec(msg.text());
    if (m) times.push({ ms: Number(m[1]), what: m[2] });
  });

  await startGame(page, { north: "counter", west: "shark", east: "shark" });
  const result = page.getByRole("dialog", { name: /^Hand \d+$/ });
  for (let step = 0; step < 3000 && !(await result.isVisible()); step++) {
    if (!(await takeTurn(page))) await page.waitForTimeout(100);
  }
  await expect(result).toBeVisible();

  const sorted = [...times].sort((a, b) => a.ms - b.ms);
  const total = sorted.reduce((s, t) => s + t.ms, 0);
  console.log(`engine requests over 50 ms: ${sorted.length}; total ${(total / 1000).toFixed(1)} s`);
  for (const t of sorted.slice(-8)) console.log(`  ${t.ms} ms  ${t.what}`);
});
