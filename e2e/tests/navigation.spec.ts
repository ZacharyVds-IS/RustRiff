import {expect, test} from "../fixtures";

/**
 * Navigation tests — verify that clicking nav buttons correctly transitions between screens.
 *
 * These tests cover the three primary routes: Home (amp controls), Tuner, and Settings.
 * All tests run in both browser-only and native Tauri mode.
 */

test("clicking Tuner navigates to the tuner waiting screen", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Tuner"}).first().click();

  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/tuner");

  await expect(tauriPage.getByText("Listening for audio input")).toBeVisible({timeout: 10_000});
});

test("clicking the Rust Riff logo returns to the home screen from Tuner", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Tuner"}).first().click();
  await expect(tauriPage.getByText("Listening for audio input")).toBeVisible({timeout: 10_000});

  // The app title is a react-router <Link> which renders as an <a> element
  await tauriPage.locator("header:visible").first().getByRole("link", {name: "Rust Riff"}).first().click();

  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/");

  await expect(tauriPage.getByText("On/Off")).toBeVisible({timeout: 5_000});
});

test("navigating between all three screens in sequence works", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Settings"}).first().click();
  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/settings");

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Tuner"}).first().click();
  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/tuner");

  await tauriPage.locator("header:visible").first().getByRole("button", {name: "Home"}).first().click();
  await expect.poll(async () => tauriPage.url(), {timeout: 15_000}).toContain("#/");
  await expect(tauriPage.getByText("On/Off")).toBeVisible({timeout: 5_000});
});