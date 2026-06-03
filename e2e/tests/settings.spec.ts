import {expect, test} from "../fixtures";

/**
 * Settings screen tests — verify that all major configuration sections are visible
 * and the screen renders correctly with the mocked audio device data.
 *
 * These tests intentionally focus on structure and labels rather than live device
 * interaction, as audio device selection requires a running Tauri backend.
 */

test("settings screen shows the Settings heading", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Settings"}).first().click();

  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/settings");

  // Target the screen heading explicitly to avoid matching the nav button label.
  const settingsHeading = tauriPage.getByRole("heading", {name: "Settings"});
  await expect(settingsHeading).toBeVisible({timeout: 20_000});
});
test("settings screen shows the Latency section with buffer size and round-trip controls", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Settings"}).first().click();

  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/settings");

  // LatencySection renders a "Latency" subtitle - use heading role to avoid strict mode
  const latencyHeading = tauriPage.getByRole("heading", {name: "Latency"});
  await expect(latencyHeading).toBeVisible({timeout: 20_000});

  await expect(tauriPage.getByText("Buffer Size")).toBeVisible({timeout: 20_000});
  await expect(tauriPage.getByRole("button", {name: "Measure Round-Trip"})).toBeVisible({timeout: 10_000});
});
