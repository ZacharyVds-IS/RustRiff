import {expect, resetIpcMockState, test,} from "../fixtures";

/**
 * Effect chain tests — verify the effect creation dialog, IPC dispatch,
 * and the subsequent appearance of added effects in the pedal board.
 */

type MockInvocation = {
  cmd: string;
  args: unknown;
};

test.beforeEach(() => {
  resetIpcMockState();
});

test("creating an effect executes add_effect command", async ({tauriPage}, testInfo) => {
    test.skip(testInfo.project.name.includes("tauri"), "IPC mock assertions run in browser-only mode.");

    await tauriPage.waitForSelector("#root", 20_000);
    await tauriPage.evaluate("globalThis.__TAURI_CLEAR_MOCK_CALLS__()");

    await tauriPage
      .locator('button:has(svg[data-testid="AddCircleIcon"])')
      .click();

    const addEffectDialog = tauriPage.getByRole("dialog", {name: "New Effect"});
    await expect(addEffectDialog).toBeVisible();

    await addEffectDialog.getByRole("combobox", {name: "Effect Type"}).click();
    await tauriPage.getByRole("option", {name: "Hard-Clipping Distortion"}).click();
    await addEffectDialog.getByRole("textbox", {name: "Name"}).fill("TestEffect");
    await addEffectDialog.locator('input[type="color"]').fill("#ff0000");

    const createButton = addEffectDialog.getByRole("button", {name: "Create"});
    await expect(createButton).toBeEnabled({timeout: 5_000});
    await createButton.click();

    await expect.poll(async () => tauriPage.evaluate(`
      (() => {
        const calls = globalThis.__TAURI_GET_MOCK_CALLS__();
        return calls.filter((invocation) => invocation.cmd === "add_effect").length;
      })()
    `), {timeout: 10_000}).toBe(1);

    const addEffectInvocations = await tauriPage.evaluate(`
      (() => {
        const calls = globalThis.__TAURI_GET_MOCK_CALLS__();
        return calls.filter((invocation) => invocation.cmd === "add_effect");
      })()
    `) as MockInvocation[];

    const addEffectInvocation = addEffectInvocations[0];

    expect(addEffectInvocation).toBeDefined();
    if (!addEffectInvocation) {
      throw new Error("Expected one add_effect invocation but none were captured.");
    }

    expect(addEffectInvocation.args).toMatchObject({
      effectDto: {
        kind: "HCDistortion",
        data: {
          name: "TestEffect",
          color: "#ff0000",
        },
      },
    });
});
test("Create button in Add Effect dialog is disabled until a name is provided", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage
    .locator('button:has(svg[data-testid="AddCircleIcon"])')
    .click();

  const addEffectDialog = tauriPage.getByRole("dialog", {name: "New Effect"});
  await expect(addEffectDialog).toBeVisible();

  // Before filling in a name the Create button must be disabled
  await expect(addEffectDialog.getByRole("button", {name: "Create"})).toBeDisabled();

  // After selecting an effect type and entering a name it becomes enabled
  await addEffectDialog.getByRole("combobox", {name: "Effect Type"}).click();
  await tauriPage.getByRole("option", {name: "Hard-Clipping Distortion"}).click();
  await addEffectDialog.getByRole("textbox", {name: "Name"}).fill("My Dist");

  await expect(addEffectDialog.getByRole("button", {name: "Create"})).toBeEnabled({timeout: 5_000});
});
