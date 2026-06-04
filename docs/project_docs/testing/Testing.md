# Testing Concepts

## Types of Tests

### Unit Tests
Test one piece in isolation (component, hook, function) with mocked dependencies.
- **Use for:** Logic, state management, utility functions
- **Best practice:** Test behavior, not implementation; use Arrange-Act-Assert pattern
- **Frontend:** React hooks, Redux selectors
- **Backend:** Service methods, domain logic

### Integration Tests  
Test components together across layer boundaries (frontend ↔ backend, service ↔ persistence).
- **Use for:** IPC contracts, state sync, config persistence
- **Keys:** Tauri calls must match types, live state syncs with persisted state

### End-to-End (E2E) Tests
Test complete user journeys (click → action → outcome).
- **Use for:** Important workflows (add effect, switch channel, change driver)
- **Best practice:** Assert visible outcomes, not implementation details; keep deterministic
- **Modes:** Browser-only (fast, mocked) vs. Tauri (slow, real app)

### Audio/DSP Tests
Test real-time audio without allocations, blocking, or failures in the hot path.
- **Use for:** Effect processors, resampling, buffer management
- **Hot path rule:** No `Vec::new()`, no mutexes, no I/O in DSP loop

## Code Coverage vs. Test Quality

**Coverage** = % of lines executed (not whether they work correctly)  
**Quality** = Mutation testing score (whether tests catch bugs)

100% coverage with weak tests is **false confidence**. Use mutation testing to verify effectiveness.

## Running Tests

**Frontend:**
```bash
npm run test:ui              # Unit tests
npm run test:ui-coverage     # With coverage
npm run test:e2e:browser     # Fast E2E (browser-only, CI default)
npm run test:e2e             # Full E2E (native Tauri app)
npm run test:mutation        # Mutation testing
```

**Backend:**
```bash
cargo test --all             # All tests
cargo test --all -- --nocapture  # With output
cargo test --lib services::effects  # Specific module
```


