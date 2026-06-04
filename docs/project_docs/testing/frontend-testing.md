# Frontend Testing

## Test Stack

| Tool | Purpose |
|------|---------|
| `vitest` | Test runner & assertions |
| `jsdom` | Browser-like environment |
| `@testing-library/react` | Render & interact with React |
| `@testing-library/user-event` | Realistic user input |
| `@playwright/test` | E2E browser automation |
| `@stryker-mutator/core` | Mutation testing |

## Unit Tests — Arrange-Act-Assert Pattern

```typescript
test("should enable submit button when form is valid", () => {
  // Arrange: set up
  const { getByRole } = render(<MyForm />);
  const input = getByRole("textbox");
  
  // Act: do something
  await userEvent.type(input, "valid input");
  
  // Assert: verify
  expect(getByRole("button", { name: /submit/i })).toBeEnabled();
});
```

### Testing Hooks
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
```typescript
vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn((command) => {
    if (command === "get_effects") {
      return Promise.resolve([mockEffect1, mockEffect2]);
    }
    return Promise.reject(new Error(`Unknown: ${command}`));
  }),
}));
```

## E2E Tests

**Browser-only mode** (fast, default):
```bash
npm run test:e2e:browser
```
- Runs in Chromium with mocked Tauri IPC
- No app build required
- Great for CI and iteration

**Tauri mode** (slow, real integration):
```bash
npm run test:e2e
```
- Runs against native app binary
- Full frontend/backend integration
- Use for production confidence

### E2E Best Practices
- ✅ Assert on state outcomes (dialog closing) not UI elements
- ✅ Use capability detection for IPC assertions:
```typescript
const hasMockLayer = await page.evaluate(() => 
  typeof (window as any).__TAURI_MOCK__ !== "undefined"
);
if (hasMockLayer) {
  // Assert IPC calls only when mock exists
}
```
- ❌ Avoid timing-sensitive element visibility waits

## Running Tests

```bash
npm run test:ui              # Unit tests
npm run test:ui-coverage     # With coverage
npm run test:e2e:browser     # E2E (browser-only, fast)
npm run test:e2e             # E2E (native Tauri app)
npm run test:mutation        # Mutation testing
```

## Key Rules

1. **Test behavior, not implementation** — Test what happens, not how it works
2. **Use userEvent over fireEvent** — More realistic interactions
3. **Mock external dependencies** — Tauri IPC, API calls, timers
4. **Group related tests** — Use `describe` blocks for clarity
5. **Avoid timing-sensitive waits** — Assert on state, not UI visibility


