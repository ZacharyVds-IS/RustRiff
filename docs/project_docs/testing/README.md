# Testing Documentation

This directory contains comprehensive guides for testing in RustRiff across the full stack: frontend (TypeScript/React), backend (Rust), and end-to-end user flows.

## Documentation Structure

### [Testing.md](./Testing.md) — General Testing Concepts
**For everyone.** Start here to understand:
- Types of tests: unit, integration, E2E, and audio/DSP testing
- What code coverage *actually* means and its limitations
- Mutation testing and why it matters
- How to run tests across the entire project

**Read this if:** You're new to the project or want to understand the testing philosophy.

### [frontend-testing.md](./frontend-testing.md) — Frontend-Specific Testing
**For frontend developers.** Details:
- Frontend test stack (Vitest, React Testing Library, Playwright)
- Unit test patterns (Arrange-Act-Assert, mocking Tauri commands, state testing)
- E2E test patterns (browser-only vs. Tauri mode, deterministic testing, IPC verification)
- Best practices for writing effective frontend tests

**Read this if:** You're writing or modifying frontend tests in `src/` or `e2e/`.

### [backend-testing.md](./backend-testing.md) — Backend-Specific Testing
**For backend developers.** Covers:
- Backend test stack (Cargo, Rust test framework)
- RustRiff's `success_path` / `failure_path` pattern
- Hot path testing (real-time DSP constraints, allocation-free code)
- Service and infrastructure layer testing patterns
- Audio/DSP-specific considerations

**Read this if:** You're writing or modifying Rust tests in `src-tauri/src/`, especially audio or effect processing code.

## Quick Links By Task

| I want to... | See... |
|---|---|
| Understand testing philosophy | [Testing.md](./Testing.md) |
| Check test coverage | [Testing.md → Test Coverage vs. Actual Test Quality](./Testing.md#test-coverage-vs-actual-test-quality) |
| Write a React component unit test | [frontend-testing.md → Unit Test Patterns](./frontend-testing.md#unit-test-patterns) |
| Write an E2E test for a user flow | [frontend-testing.md → E2E Test Patterns](./frontend-testing.md#e2e-test-patterns) |
| Test a Rust effect processor | [backend-testing.md → Hot Path Testing](./backend-testing.md#hot-path-testing) |
| Test a Rust service | [backend-testing.md → Service Layer Testing](./backend-testing.md#service-layer-testing) |
| Run tests locally | [Testing.md → Running Tests](./Testing.md#running-tests) |
| Check mutation testing score | [frontend-testing.md → Mutation Testing](./frontend-testing.md#mutation-testing) |

## Running Tests

### Frontend

```bash
npm run test:ui              # Unit tests
npm run test:ui-coverage     # With coverage report
npm run test:e2e:browser     # E2E in browser-only mode (fast, CI default)
npm run test:e2e             # E2E with native Tauri app
npm run test:mutation        # Mutation testing
```

### Backend

```bash
cargo test --all             # Run all Rust tests
cargo test --all -- --nocapture  # With output
cargo test --lib services::effects  # Specific module
RUST_LOG=debug cargo test --all -- --nocapture  # With tracing
```

## Testing Strategy

RustRiff uses a layered testing approach:

1. **Unit tests** catch logic bugs fast (frontend hooks, backend services)
2. **Integration tests** ensure layers talk correctly (frontend state ↔ backend IPC)
3. **E2E tests** prove user flows work end-to-end
4. **Audio/DSP tests** verify real-time correctness (no allocations, deterministic timing)

**Goal:** Use *all four* to build confidence at multiple levels of abstraction.

## Key Principles

- **Test behavior, not implementation.** Test what the code *does*, not how it does it.
- **Use mocks to isolate.** Mock external dependencies (Tauri IPC, filesystem, audio hardware).
- **Make tests deterministic.** Avoid timing-sensitive waits; assert on state outcomes instead.
- **Hot path is sacred.** Audio processing code must never allocate, block, or fail in the DSP loop.
- **High mutation score.** Aim for effective tests that catch regressions, not just coverage.

## Contributing Tests

When adding a feature or fixing a bug:

1. **Write a test first** (or at least alongside the code)
2. **Group success and failure cases** (following the `success_path` / `failure_path` pattern in backend)
3. **Use clear test names** that encode what's being tested and the expected outcome
4. **Run the full suite** before committing to catch regressions

```bash
# Frontend
npm run test:ui && npm run test:e2e:browser

# Backend
cargo test --all

# Both
npm run test:ui && cargo test --all
```

## References

- [General Testing Guide](./Testing.md)
- [Frontend Testing Guide](./frontend-testing.md)
- [Backend Testing Guide](./backend-testing.md)
- [RustRiff Architecture](../arc42/arc42_project_descriptions.md)
- [Project Structure](../project-structure.md)

