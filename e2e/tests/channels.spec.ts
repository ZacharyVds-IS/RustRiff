import {expect, resetIpcMockState, test} from "../fixtures";

type MockInvocation = {
  cmd: string;
  args: unknown;
};

/**
 * Channel management tests — verify the ChannelSelector, AddChannelDialog,
 * and the resulting IPC command dispatch.
 */

test.beforeEach(() => {
  resetIpcMockState();
});

test("opening the Add Channel dialog shows the New Channel heading", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  // The DropdownSelector adds an "Add New Channel" option at the bottom of the list.
  // Click the channel Select to open the dropdown, then pick the add option.
  await tauriPage.getByRole("combobox").click();
  await tauriPage.getByRole("option", {name: "Add New Channel"}).click();

  const dialog = tauriPage.getByRole("dialog", {name: "New Channel"});
  await expect(dialog).toBeVisible({timeout: 5_000});
});

test("Create button in Add Channel dialog is disabled when name is empty", async ({tauriPage}) => {
  await tauriPage.waitForSelector("#root", 20_000);

  await tauriPage.getByRole("combobox").click();
  await tauriPage.getByRole("option", {name: "Add New Channel"}).click();

  const dialog = tauriPage.getByRole("dialog", {name: "New Channel"});
  await expect(dialog).toBeVisible({timeout: 5_000});

  // Create button must be disabled when no name has been typed yet
  await expect(dialog.getByRole("button", {name: "Create"})).toBeDisabled();
});

test("adding a channel fires the add_channel IPC command with the typed name", async ({tauriPage}, testInfo) => {
  await tauriPage.waitForSelector("#root", 20_000);

  const hasMockCalls = await tauriPage.evaluate(() =>
    typeof (globalThis as typeof globalThis & {
      __TAURI_CLEAR_MOCK_CALLS__?: () => void;
      __TAURI_GET_MOCK_CALLS__?: () => Array<{cmd: string; args: unknown}>;
    }).__TAURI_GET_MOCK_CALLS__ === "function",
  );

  if (hasMockCalls) {
    await tauriPage.evaluate("globalThis.__TAURI_CLEAR_MOCK_CALLS__()");
  }

  // Open the Add Channel dialog
  await tauriPage.getByRole("combobox").click();
  await tauriPage.getByRole("option", {name: "Add New Channel"}).click();

  const dialog = tauriPage.getByRole("dialog", {name: "New Channel"});
  await expect(dialog).toBeVisible({timeout: 5_000});

  await dialog.getByRole("textbox", {name: "Channel name"}).fill("Lead");

  const createButton = dialog.getByRole("button", {name: "Create"});
  await expect(createButton).toBeEnabled({timeout: 5_000});
  await createButton.click();

  // Wait for the IPC call to be recorded with a longer timeout
  if (hasMockCalls) {
    await expect.poll(async () =>
      tauriPage.evaluate(`
        (() => {
          const calls = globalThis.__TAURI_GET_MOCK_CALLS__();
          return calls.filter((invocation) => invocation.cmd === "add_channel").length;
        })()
      `),
      {timeout: 10_000}
    ).toBe(1);

    const invocations = await tauriPage.evaluate(`
      (() => {
        const calls = globalThis.__TAURI_GET_MOCK_CALLS__();
        return calls.filter((invocation) => invocation.cmd === "add_channel");
      })()
    `) as MockInvocation[];

    expect(invocations[0]).toBeDefined();
    expect(invocations[0]!.args).toMatchObject({channelName: "Lead"});
  }
});









