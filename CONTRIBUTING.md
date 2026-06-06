# Contributing to RustRiff

Thank you for your interest in contributing to RustRiff! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Please read and follow our Code of Conduct.

## Architecture Overview

RustRiff follows a strict layered architecture. When contributing, ensure your changes respect these boundaries:

- **`src/`** - Frontend UI and React state management
- **`src-tauri/src/commands/`** - Tauri IPC boundary (Transport layer) - keep these thin!
- **`src-tauri/src/services/`** - Application and business logic
- **`src-tauri/src/domain/`** - DTOs and domain concepts (Single Source of Truth)
- **`src-tauri/src/infrastructure/`** - Low-level integrations (Filesystem, Audio, Persistence)

## Getting Started

1. **Fork the repository** and clone it to your local machine
2. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Set up your development environment** by following the prerequisites in the [README.md](README.md)

## Development Workflow

### Running the App Locally
```bash
npm install
npm run tauri dev
```

### Running Tests
Test coverage is mandatory for all behavior changes. Run the following commands:

**Frontend Tests (React/TypeScript):**
```bash
npm run test
```

**Backend Tests (Rust):**
```bash
npm run test:rust
```

**End-to-End Tests:**
```bash
npm run test:e2e:build
npm run test:e2e
```

**Mutation Tests (code quality):**
```bash
npm run test:mutation
```

### Code Quality Checks

Before opening a PR, run all quality checks:

**Linting:**
```bash
npm run lint                  # Frontend linting
npm run format-rust          # Rust formatting check
npm run clippy-rust          # Rust static analysis
```

## Review Criteria

All pull requests must adhere to these core principles:

### 1. **Core Architectural Integrity**
- Changes must preserve the layered architecture
- Thin IPC commands that delegate to services
- No business logic in the transport layer

### 2. **Audio & DSP Performance (Hot Path Constraints)**
Any code in `audio_service.rs` or effect processors is subject to strict real-time constraints:
- ❌ **No heap allocations** (Vec::new(), Box, String) in the process loop
- ❌ **No locking or blocking** (mutexes that could be held by slow threads)
- ❌ **No filesystem I/O** or network calls during audio processing
- ✅ Use preallocated buffers and explicit ownership
- ✅ Use cached transforms and predictable block processing

### 3. **Frontend & Backend Contract Safety**
- **Use generated types:** Import `*Dto` types from generated domain layer
- **Flag handwritten TS types** that duplicate backend DTOs
- **Single Source of Truth:** Avoid duplicating constants in both Rust and TypeScript
- **No top-level await:** Use lazy async getters for config instead

### 4. **Frontend State & Identity Logic**
- React `key` props must be unique and persistent
- Selection logic must be based on unique backend IDs, not array indices
- Avoid transient UI flows with duplicate placeholder IDs

### 5. **Persistence & State Recovery**
- Runtime state changes must be reflected in persisted config
- Runtime-only values must NOT be accidentally persisted
- Reloading persisted state must produce identical effect order and parameters

### 6. **Error Handling & UX**
- Developer logs must contain technical context (file paths, specific errors)
- User-facing messages must be clear and actionable
- Graceful fallbacks for missing resources (e.g., default IR on startup failure)

### 7. **Testing Standards**
All behavior changes require tests for both success and failure paths:
```rust
#[cfg(test)]
mod tests {
    mod success_path {
        // Happy path tests
    }
    
    mod failure_path {
        // Edge cases, error handling
    }
}
```

## Pull Request Process

1. **Update tests** for any behavior changes (both success and failure paths)
2. **Run all checks** before pushing:
   ```bash
   npm run lint
   npm run format-rust
   npm run clippy-rust
   npm run test
   npm run test:rust
   ```
3. **Keep commits focused** - one feature/fix per PR
4. **Write a clear PR description** including:
   - Problem statement: What issue does this solve?
   - Solution summary: How does your change work?
   - Testing notes: What did you test?
   - Any architectural considerations

5. **Respond to feedback** - maintainers may request changes to ensure code quality

## Release Process

RustRiff uses automated cross-platform releases. When your changes are merged to `main` and a version tag is pushed:

1. **Tag creation triggers** the `Tauri Release` GitHub Action
2. **Automated builds** run on Windows, macOS, and Linux simultaneously
3. **Cross-compilation** is handled by Tauri's official GitHub Action
4. **Installers and binaries** are attached to the GitHub Release

Example:
```bash
git tag v1.0.0
git push origin v1.0.0
```

This automatically builds and releases all platform-specific installers.

## Documentation

RustRiff uses auto-generated API documentation:

**Frontend API (TypeScript):**
```bash
npm run docs:generateReactDocs
```

**Backend API (Rust):**
```bash
npm run docs:generateRustDocs
```

**Full documentation site:**
```bash
npm run docs:dev      # Development mode
npm run docs:build    # Production build
```

Documentation lives in the `docs/` folder and is published via GitHub Pages.

## Continuous Integration

The project uses GitHub Actions for automated testing and building:

- **Main CI** (`main.yml`) - Runs on every push and PR to validate code quality
- **Release** (`release.yml`) - Automatically builds and releases on version tags
- **Documentation** (`publish-docs.yml`) - Auto-publishes docs on main branch

## Questions or Need Help?

- Check existing [GitHub Issues](https://github.com/ZacharyVds-IS/Guitar-Amplifier/issues)
- Review the [project documentation](https://zacharyvds-is.github.io/Guitar-Amplifier/)
- Open a discussion or new issue with your question

## License

By contributing to RustRiff, you agree that your contributions will be licensed under the GPL-3.0-or-later License. See [LICENSE.md](LICENSE.md) for details.

---

Thank you for making RustRiff better! 🎸

