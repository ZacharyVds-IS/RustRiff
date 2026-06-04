import {expect, test} from "../fixtures";

/**
 * Generated / sanity tests — lightweight baseline checks auto-generated as a
 * starting point for each new screen or feature area. Replace these with
 * focused functional tests as the feature matures.
 */

test("app root mounts without a JavaScript error", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  // A mounted #root with the app title present is the baseline sanity check
  await expect(tauriPage.getByText("Rust Riff")).toBeVisible();
});