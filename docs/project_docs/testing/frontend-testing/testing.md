# Frontend Testing: Packages & Implementation

> **Documentation Reorganized:** The testing docs have been restructured for clarity. 
> 
> - **New Location:** See [`docs/project_docs/testing/frontend-testing.md`](../frontend-testing.md)
> - **General Concepts:** See [`docs/project_docs/testing/Testing.md`](../Testing.md)
> - **Backend Testing:** See [`docs/project_docs/testing/backend-testing.md`](../backend-testing.md)
> - **Navigation:** Start at [`docs/project_docs/testing/README.md`](../README.md)

This doc covers the specific tools and patterns used for frontend testing in RustRiff. For general testing concepts (types of tests, coverage, mutation testing), see [Testing.md](../Testing.md).

## Test Stack

### Unit & Integration Test Tools

| Package | Version | Purpose |
|---------|---------|---------|
| `vitest` | Latest | Test runner and assertion framework |
| `jsdom` | Latest | Browser-like DOM environment for unit tests |
| `@testing-library/react` | Latest | Render and interact with React components |
| `@testing-library/user-event` | Latest | Realistic user input simulation (vs. fireEvent) |
| `@vitest/coverage-v8` | Latest | Coverage reports using V8 |

### E2E Test Tools

| Package | Version | Purpose |
|---------|---------|---------|
| `@playwright/test` | Latest | Browser automation and E2E framework |
| `@srsholmes/tauri-playwright` | Latest | Tauri-specific Playwright utilities |

### Quality Tools

| Package | Version | Purpose |
|---------|---------|---------|
| `@stryker-mutator/core` | Latest | Mutation testing orchestration |
| `@stryker-mutator/vitest-runner` | Latest | Vitest integration for mutation testing |

## Unit Test Patterns

### Arrange-Act-Assert (AAA)
Structure tests clearly by separating setup, action, and verification:

```typescript
test("should enable submit button when form is valid", () => {
  // Arrange: set up component state
  const { getByRole } = render(<MyForm />);
  const input = getByRole("textbox");
  
  // Act: trigger the action
  await userEvent.type(input, "valid input");
  
  // Assert: verify the outcome
  expect(getByRole("button", { name: /submit/i })).toBeEnabled();
});
```

### Testing Hooks

Use `@testing-library/react`'s `renderHook` for isolated hook testing:

```typescript
test("useEffectChain updates when effects are added", () => {
  const { result } = renderHook(() => useEffectChain(mockChainId));
  expect(result.current.effects).toHaveLength(0);
  
  act(() => {
    result.current.addEffect("distortion");
  });
  
  expect(result.current.effects).toHaveLength(1);
});
```

### Mocking Tauri Commands

Mock IPC calls using `vi.mock()`:

```typescript
vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn((command, args) => {
    if (command === "get_effects") {
      return Promise.resolve([mockEffect1, mockEffect2]);
    }
    return Promise.reject(new Error(`Unknown command: ${command}`));
  }),
}));
```

### Testing State Management

For Redux/state-based flows, test selectors and reducers independently:

```typescript
test("effectList selector filters by chain", () => {
  const state = {
    effects: [
      { id: "1", chainId: "chain-a" },
      { id: "2", chainId: "chain-b" },
    ],
  };
  
  const result = selectEffectsByChain(state, "chain-a");
  expect(result).toHaveLength(1);
  expect(result[0].id).toBe("1");
});
```

## E2E Test Patterns

### Browser-Only vs. Tauri Mode

**Browser-only mode** (default, fast):
- Runs in Chromium with mocked Tauri IPC
- No app build required
- Use for CI and local iteration
- Ideal for testing UI flows independent of backend

```bash
npm run test:e2e:browser
```

**Tauri mode** (slow, full integration):
- Runs against the real native app binary
- Requires building the Tauri app first
- Use for validating full frontend/backend alignment
- Guarantees real IPC and backend behavior

```bash
npm run test:e2e
```

### Writing Deterministic E2E Tests

Avoid timing-sensitive selectors; instead, assert on state outcomes:

```typescript
test("creating an effect closes the dialog", async ({ page }) => {
  // ❌ DON'T: Wait for text that may not render in mock mode
  // await expect(page.getByText("NewEffect")).toBeVisible();
  
  // ✅ DO: Assert on complete state change (dialog closing)
  const addEffectDialog = page.locator('[role="dialog"]', { has: page.getByText("Add Effect") });
  await expect(addEffectDialog).toBeHidden({ timeout: 10_000 });
});
```

### Checking IPC in E2E Tests

Use capability-based detection instead of mode gates:

```typescript
test("selecting ASIO driver calls switch_driver command", async ({ page }) => {
  // Only make IPC assertion if mock layer exists (browser-only)
  const hasMockLayer = await page.evaluate(() => {
    return typeof (window as any).__TAURI_MOCK__ !== "undefined";
  });
  
  if (hasMockLayer) {
    // Assert the command was called with correct args
    const calls = await page.evaluate(() => (window as any).__TAURI_MOCK__.invoke_history);
    expect(calls).toContainEqual(
      expect.objectContaining({
        cmd: "switch_driver",
        args: { driver: "ASIO" },
      })
    );
  }
});
```

## Running Tests

### Unit Tests

```bash
# Run all unit tests
npm run test:ui

# Run with watch mode during development
npm run test:ui -- --watch

# Generate coverage report
npm run test:ui-coverage
```

### E2E Tests

```bash
# Fast browser-only suite (CI default)
npm run test:e2e:browser

# Full suite with Tauri app
npm run test:e2e

# E2E with watch mode
npm run test:e2e -- --watch
```

### Mutation Testing

```bash
# Run mutation testing and generate report
npm run test:mutation

# View current mutation score
npm run test:mutation:dashboard
```

Current mutation score:

[![Mutation testing badge](https://img.shields.io/endpoint?style=flat&url=https%3A%2F%2Fbadge-api.stryker-mutator.io%2Fgithub.com%2FZacharyVds-IS%2FGuitar-Amplifier%2Fmain)](https://dashboard.stryker-mutator.io/reports/github.com/ZacharyVds-IS/Guitar-Amplifier/main)

## Tips for Effective Frontend Tests

1. **Test user behavior, not implementation**: Test that clicking a button calls the right handler, not that state is updated by a specific Redux action.

2. **Use `userEvent` over `fireEvent`**: `userEvent` simulates real user interactions (typing character-by-character, focusing fields) vs. synthetic DOM events.

3. **Avoid wait-for-text in E2E**: Instead, assert on dialog/form state (visibility, disabled buttons) that happens as a side effect of the action.

4. **Mock external dependencies**: Tauri commands, API calls, timers—isolate the unit under test.

5. **Group related tests in describe blocks**: Makes reports clearer and helps with selective test runs.

```typescript
describe("EffectChain Component", () => {
  describe("when adding an effect", () => {
    test("should display add dialog");
    test("should close dialog on success");
    test("should show error on failure");
  });
  
  describe("when reordering effects", () => {
    test("should update effect order in UI");
    test("should persist order to backend");
  });
});
```

## References

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Playwright Documentation](https://playwright.dev/docs/intro)
- [Stryker Mutation Testing](https://stryker-mutator.io/)


