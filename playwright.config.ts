import {defineConfig, devices} from "@playwright/test";
import {existsSync} from "fs";
import {resolve} from "path";

/**
 * RustRiff Playwright E2E configuration.
 *
 * Two projects are defined:
 *   browser-only  – Headless Chromium with mocked Tauri IPC.
 *                   Fast, no binary needed. Used in every CI run.
 *   tauri         – Socket bridge to the real native webview.
 *                   Requires the binary built with --features e2e-testing.
 *                   Set TAURI_BINARY env-var to the binary path (default:
 *                   ./src-tauri/target/release/rustriff[.exe]).
 *
 * Run locally:
 *   npm run test:e2e:browser   # browser-only (no Tauri binary needed)
 *   npm run test:e2e           # build tauri binary + run both projects
 *
 * Build Tauri binary first (required for tauri tests):
 *   npm run tauri build -- --no-bundle --features e2e-testing
 */

const IS_WIN = process.platform === "win32";
const DEFAULT_BINARY = IS_WIN
  ? "./src-tauri/target/release/rustriff.exe"
  : "./src-tauri/target/release/rustriff";
const TAURI_BINARY = process.env.TAURI_BINARY ?? DEFAULT_BINARY;

// Check if binary exists for tauri project
const BINARY_EXISTS = existsSync(resolve(TAURI_BINARY));

// Derive the project type directly from defineConfig so it stays in sync with Playwright.
type PlaywrightProject = NonNullable<Parameters<typeof defineConfig>[0]["projects"]>[number];

// Determine which projects to run.
const projects: PlaywrightProject[] = [
  {
    name: "browser-only",
    use: { ...devices["Desktop Chrome"] },
  },
];

// Add tauri project if binary exists or explicitly forced.
if (BINARY_EXISTS || process.env.FORCE_TAURI === "true") {
  projects.push({
    name: "tauri",
    use: {},  // No browser device needed; the fixture drives the native webview.
  });
}

export default defineConfig({
  testDir: "./e2e/tests",
  timeout: 60_000,
  expect: { timeout: 10_000 },
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  // Only one Tauri instance may be running at a time
  workers: 1,
  reporter: [
    ["html", { open: "never", outputFolder: "playwright-report" }],
    ...(process.env.CI ? [["github"] as const] : []),
  ],
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 1420",
    url: "http://127.0.0.1:1420",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },

  projects,
});

