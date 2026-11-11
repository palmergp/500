import type { Locator, Page } from "@playwright/test";

/** Clicks if the element is there; the table can move on between finding and clicking. */
async function tryClick(target: Locator): Promise<boolean> {
  if ((await target.count()) === 0) return false;
  try {
    await target.click({ timeout: 1500 });
    return true;
  } catch {
    return false;
  }
}

/** A top-bar control by name: a button, or on small screens an item in the "More" menu (opened
 * for you). */
export async function topBarControl(page: Page, name: string): Promise<Locator> {
  const more = page.getByRole("button", { name: "More", exact: true });
  if (await more.isVisible()) {
    if ((await more.getAttribute("aria-expanded")) !== "true") await more.click();
    return page.getByRole("menu", { name: "More" }).getByRole("menuitem", { name: new RegExp(`^${name}`) });
  }
  return page.getByRole("button", { name, exact: true });
}

/** Takes the human's turn with a simple policy: pass if allowed, otherwise the first legal option. */
export async function takeTurn(page: Page): Promise<boolean> {
  if (await tryClick(page.locator(".bidding-box .call.pass:not([disabled])"))) return true;
  if (await tryClick(page.locator(".bidding-box .call:not([disabled])").first())) return true;
  const discard = page.locator(".discard-bar button");
  if (await discard.count()) {
    const needed = Number((await discard.innerText()).split("/")[1]);
    for (let i = 0; i < needed; i++) await tryClick(page.locator(".hand .card.playable:not(.selected)").first());
    return tryClick(discard);
  }
  if (await tryClick(page.locator(".joker-choice .call").first())) return true;
  return tryClick(page.locator(".hand .card.playable").first());
}

/** Starts from a clean slate (no saved game, fast bots) at the main menu. */
export async function freshStart(page: Page, settings: Record<string, unknown> = {}) {
  await page.goto("/");
  await page.evaluate((extra) => {
    localStorage.clear();
    sessionStorage.clear();
    localStorage.setItem("fh500.settings", JSON.stringify({ speed: "fast", ...extra }));
  }, settings);
  await page.reload();
  await page.getByRole("navigation", { name: "Main menu" }).waitFor();
}

/** Clears any saved game and deals a new one with the given seats at fast speed. */
export async function startGame(page: Page, seats: { north?: string; west?: string; east?: string; hands?: "4 hands" } = {}) {
  await freshStart(page);
  await page.getByRole("button", { name: /^Play solo/ }).click();
  const dialog = page.getByRole("dialog", { name: "New game" });
  if (seats.north) await dialog.getByLabel("Your partner (North)").selectOption(seats.north);
  if (seats.west) await dialog.getByLabel("Left opponent (West)").selectOption(seats.west);
  if (seats.east) await dialog.getByLabel("Right opponent (East)").selectOption(seats.east);
  if (seats.hands) await dialog.getByLabel(seats.hands).check();
  await dialog.getByRole("button", { name: "Deal" }).click();
}
