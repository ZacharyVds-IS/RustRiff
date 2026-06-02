import {expect, test} from "../fixtures";

/**
 * Navigation tests — verify that clicking nav buttons correctly transitions between screens.
 *
 * These tests cover the three primary routes: Home (amp controls), Tuner, and Settings.
 * All tests run in both browser-only and native Tauri mode.
 */

test("clicking Tuner navigates to the tuner waiting screen", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.getByRole("button", {name: "Tuner"}).click();

  // Tuner renders an idle spinner and instructional text when no pitch data is received yet
  // Wait for the instructional text to appear with a longer timeout to account for navigation
  await expect(
    tauriPage.getByText("Listening for audio input"),
  ).toBeVisible({timeout: 10_000});
});

test("clicking the Rust Riff logo returns to the home screen from Tuner", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.getByRole("button", {name: "Tuner"}).click();
  await expect(tauriPage.getByText("Listening for audio input")).toBeVisible({timeout: 10_000});

  // The app title is a react-router <Link> which renders as an <a> element
  await tauriPage.getByRole("link", {name: "Rust Riff"}).click();

  await expect(tauriPage.getByText("On/Off")).toBeVisible({timeout: 5_000});
});

test("navigating between all three screens in sequence works", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  // Home → Settings
  await tauriPage.getByRole("button", {name: "Settings"}).click();
  await expect(tauriPage.getByRole("heading", {name: "Settings"})).toBeVisible({timeout: 15_000});

  // Settings → Tuner
  await tauriPage.getByRole("button", {name: "Tuner"}).click();
  await expect(tauriPage.getByText("Listening for audio input")).toBeVisible({timeout: 10_000});

  // Tuner → Home
  await tauriPage.getByRole("button", {name: "Home"}).click();
  await expect(tauriPage.getByText("On/Off")).toBeVisible({timeout: 5_000});
});