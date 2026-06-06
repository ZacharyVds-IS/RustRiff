# Backend Testing

## Test Stack

| Tool | Purpose |
|------|---------|
| `std::test` | Built-in Rust test framework |
| `#[test]` | Test macro |
| `#[should_panic]` | Verify expected panics |
| `mockall` | Mock traits |
| `tracing` | Logging for tests |

## Test Organization: `success_path` & `failure_path`

RustRiff tests use nested modules to organize happy path and edge cases:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(test)]
    mod success_path {
        use super::*;
        // Valid behavior tests
    }
    
    #[cfg(test)]
    mod failure_path {
        use super::*;
        // Error & edge case tests
    }
}
```

## Example: Tone Stack Parameters

```rust
#[cfg(test)]
mod success_path {
    #[test]
    fn bass_set_to_positive_value_should_succeed() {
        let tone_stack = ToneStack::new();
        tone_stack.set_bass(0.5);
        assert_eq!(tone_stack.bass().load(Ordering::Relaxed), 0.5);
    }
}

#[cfg(test)]
mod failure_path {
    #[test]
    #[should_panic(expected = "Bass must be positive and between 0 and 1")]
    fn bass_set_to_negative_value_should_panic() {
        let tone_stack = ToneStack::new();
        tone_stack.set_bass(-0.5);
    }
}
```

## Service Testing with Mocks

```rust
fn new_cm() -> Arc<Mutex<ChannelManager>> {
    Arc::new(Mutex::new(ChannelManager::new()))
}

fn build_service(handler: MockAudioHandlerTrait) -> AudioService {
    AudioService::new_with_handler(Arc::new(handler), new_cm())
}

#[test]
fn audio_service_should_initialize_channels() {
    let mock_handler = MockAudioHandlerTrait::new();
    let service = build_service(mock_handler);
    assert!(!service.channel_manager().lock().unwrap().channels().is_empty());
}
```

## Hot Path Testing (DSP Loop)

Audio processing must have **zero allocations**, **no blocking**, and **predictable timing**.

**Tests verify:**
- Correct audio output (sample rates match, effects apply properly)
- No heap allocations (no `Vec::new()`, `Box`, `String`)
- No mutexes or blocking in the process loop
- Deterministic timing (same code path = same microseconds)

```rust
#[test]
fn resample_policy_should_exist_for_mismatched_rates() {
    let policy_down = ResamplePolicy::from_rates(48_000, 44_100, 32);
    assert!(matches!(policy_down, ResamplePolicy::PreDsp(_)));
    
    let policy_up = ResamplePolicy::from_rates(44_100, 48_000, 32);
    assert!(matches!(policy_up, ResamplePolicy::PostDsp(_)));
}
```

## Running Tests

```bash
cargo test --all                    # All tests
cargo test --all -- --nocapture     # With output
cargo test --lib services::effects  # Specific module
RUST_LOG=debug cargo test --all -- --nocapture  # With tracing
```

## Best Practices

1. **Descriptive test names** — `function_input_should_expected_outcome`
2. **Use Arrange-Act-Assert** — Separate setup, action, verification
3. **Mock external dependencies** — Hardware, filesystem, network
4. **Test invariants, not implementation** — Test the *what*, not the *how*
5. **Group related tests** — Use `success_path` / `failure_path` modules


