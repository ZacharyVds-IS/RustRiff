# RustRiff Maintainer Review Instructions

This document outlines the mandatory review criteria for the RustRiff desktop guitar amplifier application. Maintainers must prioritize **low-latency audio performance**, **state correctness**, and **frontend/backend contract safety** over superficial style.

---

## 1. Core Architectural Integrity
The project follows a strict layered architecture. Changes that weaken these boundaries should be flagged.
* **Layer Responsibilities:**
    * `src/`: Frontend UI and React state.
    * `src-tauri/src/commands/`: The Tauri IPC boundary (Transport layer). Keep these thin; delegate logic to services.
    * `src-tauri/src/services/`: Application and business logic.
    * `src-tauri/src/domain/`: DTOs and domain concepts (The Source of Truth).
    * `src-tauri/src/infrastructure/`: Low-level integrations (Filesystem, Audio, Persistence).
* **Guarantees:**
    * Does the change preserve real-time audio safety?
    * Does it maintain alignment between frontend state, backend state, and persisted state?
    * Does it avoid "magic" or "clever" code in favor of explicit, predictable logic?

## 2. Audio & DSP Performance (The "Hot Path")
Rust audio processing code has strict real-time constraints. Any code in `audio_service.rs` or effect processors must be reviewed with extreme scrutiny.
* **Strict Prohibitions:**
    * **No heap allocations** (e.g., `Vec::new()`, `Box`, `String`) inside the `process` loop.
    * **No locking or blocking** (no mutexes that could be held by a slow thread).
    * **No filesystem I/O** or network calls during processing.
* **Preferred Patterns:**
    * Use preallocated buffers and explicit ownership.
    * Use cached transforms and predictable block processing.
    * Minimize unnecessary cloning or buffer copies.

## 3. Frontend & Backend Contract Safety
Treat the app as a single contract-driven system across the Tauri IPC boundary.
* **Type Discipline:**
    * **Use Generated Types:** Use `IrProfileDto`, `EffectDto`, etc., from the generated domain layer.
    * **Flag Drift:** Flag handwritten TS types that duplicate backend DTOs.
    * **Single Source of Truth:** Avoid duplicating constants (like default values) in both Rust and TS.
* **Initialization:**
    * Avoid "module-load fragility." Do not perform async IPC or `top-level await` during module import.
    * Use synchronous fallback constants or lazy async getters for config.

## 4. Frontend State & Identity Logic
The app relies heavily on list interactions (reordering, selecting, deleting effects).
* **Identity Checks:**
    * Verify that React `key` and `draggableId` are unique and persistent.
    * Ensure selection logic is based on unique IDs (backend IDs), not array indices or object references.
* **Flag Risks:**
    * Transient UI flows where duplicate placeholder IDs might exist before backend hydration.
    * Logic where a "remove" action could accidentally target multiple items due to non-unique identifiers.

## 5. Persistence & State Recovery
The app must reliably reconstruct the effect chain upon restart.
* **Alignment:** Is the live runtime state change also reflected in the persisted config?
* **Selective Persistence:** Ensure runtime-only values (like transient UI toggles) are *not* accidentally persisted.
* **Reconstruction:** Verify that reloading the persisted state produces the exact same effect order and parameters.

## 6. Error Handling & User Experience
Errors are a product feature.
* **Developer Detail:** Logs must contain enough technical context (file paths, specific Rust errors) for debugging.
* **User Actionability:** Frontend messages must be clear. Distinguish between "Unsupported Format," "File Missing," and "Resource Busy."
* **Graceful Fallbacks:** If a custom IR is missing on startup, does the app fall back to a default safely?

## 7. Testing Standards
Regression tests are required for all behavior changes.
* **Rust:** Use focused inline `#[cfg(test)]` with `success_path` and `failure_path` modules.
* **Coverage:** Ensure both the "happy path" and edge cases (e.g., file lock, invalid ID) are covered.

## 8. How to Respond (Mandatory Review Output Format)
Every review response must be short, concrete, and tied to this guide.

* **For each finding, include all of these fields:**
    * **What is wrong:** A 1-2 sentence explanation of the defect/risk.
    * **Broken rule:** Quote the exact section title from this guide (for example: `2. Audio & DSP Performance (The "Hot Path")`).
    * **Why it matters here:** One sentence linking impact to RustRiff behavior (latency, persistence, DTO contract, UX, etc.).
    * **Suggested fix:** A minimal, actionable fix.
    * **Severity:** `high`, `medium`, or `low`.

* **Ordering:** List findings by severity (high -> low).
* **No findings case:** Explicitly state: `No rule violations found.`

### Response template
```text
Finding: <short title>
Severity: <high|medium|low>
What is wrong: <concise explanation>
Broken rule: <section title from this guide>
Why it matters here: <project-specific impact>
Suggested fix: <minimal concrete action>
```

### Example
```text
Finding: Cabinet usage check can block the audio thread
Severity: high
What is wrong: The command path locks `effect_chain`, which is also locked in the DSP loop.
Broken rule: 2. Audio & DSP Performance (The "Hot Path")
Why it matters here: Lock contention can cause audible dropouts while opening/removing IRs.
Suggested fix: Read cabinet usage from a non-RT metadata snapshot instead of locking the chain.
```
---

### Maintainer's Quick Checklist
- [ ] **Architecture:** Is it in the right layer?
- [ ] **Hot Path:** Does it allocate or block in the audio thread?
- [ ] **Contract:** Are generated DTOs used instead of handwritten TS interfaces?
- [ ] **Identity:** Is the React `key` logic safe for reordering/deletion?
- [ ] **Persistence:** Will this change survive an app restart?
- [ ] **UX:** Is the error message understandable to a guitar player?
