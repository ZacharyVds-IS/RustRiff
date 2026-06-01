import {expect, test} from "../fixtures";

/**
 * Smoke tests — verify the app shell loads in both browser-only and tauri modes.
 *
 * Keep these tests fast and side-effect free; they are the first gate in CI.
 */

test("app shell renders header and primary navigation", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await expect(tauriPage.getByText("Rust Riff")).toBeVisible();
  await expect(tauriPage.getByRole("button", {name: "Home"})).toBeVisible();
  await expect(tauriPage.getByRole("button", {name: "Tuner"})).toBeVisible();
  await expect(tauriPage.getByRole("button", {name: "Settings"})).toBeVisible();
});

test("home view renders amp controls", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await expect(tauriPage.getByText("On/Off")).toBeVisible();
  await expect(tauriPage.getByText("Volume")).toBeVisible();
  await expect(tauriPage.getByText("Gain")).toBeVisible();
  await expect(tauriPage.getByText("Tone stack")).toBeVisible();
  await expect(tauriPage.getByText("Master")).toBeVisible();
});
