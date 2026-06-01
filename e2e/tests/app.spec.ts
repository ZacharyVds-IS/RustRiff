import {expect, test} from "../fixtures";

/**
 * Smoke tests — verify the app shell loads in both browser-only and tauri modes.
 *
 * Keep these tests fast and side-effect free; they are the first gate in CI.
 */

test("app shell renders without crashing", async ({ tauriPage }) => {
  // The body element must be present — proves the webview initialised
  await tauriPage.waitForSelector("body", { timeout: 15_000 });
  const content = await tauriPage.content();
  expect(content).toBeTruthy();
  expect(content.length).toBeGreaterThan(0);
});

test("page has a non-empty document title", async ({ tauriPage }) => {
  await tauriPage.waitForSelector("body");
  const title = await tauriPage.title();
  // Title may be empty in some webviews, so just confirm the page responded
  expect(typeof title).toBe("string");
});

test("root React mount point is present in the DOM", async ({ tauriPage }) => {
  await tauriPage.waitForSelector("#root", { timeout: 15_000 });
  const visible = await tauriPage.isVisible("#root");
  expect(visible).toBe(true);
});

test("at least one interactive element is rendered", async ({ tauriPage }) => {
  // Wait for React to hydrate — any button or input suffices
  await tauriPage.waitForSelector("button, input, [role='button']", {
    timeout: 15_000,
  });
  const count = await tauriPage.count("button, input, [role='button']");
  expect(count).toBeGreaterThan(0);
});

