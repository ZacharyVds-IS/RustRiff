# ADR-006: Mutation Testing Rust

**Status:** Accepted (not in use)  
**Date:** May 2026  
**Context:** RustRiff - Cross-Platform Guitar Amplifier

## Problem

We want to check how good our test actualy test our code. A way of doing this is by doing mutation tests.
These test mutate your code by small increments to check if your unit tests actualy catch changes in the logic.

## Alternatives Considered

| Alternative       | Last Updated (as of date above) | Still Maintained? | How to use                | 
|-------------------|---------------------------------|-------------------|---------------------------|
| **Mutagen**       | 2022                            | No                | attribute + cargo command |
| **cargo-mutants** | May 2026                        | Yes               | cargo command             |

## Decision

"Use" **cargo-mutants** because it's still maintained, and we'd be able to easily run this in the CI pipeline if needed.

## Consequences

- **Positive:** We can check how good our test check of the changes to the source code.
- **Negative:** When running a first test to see if cargo-mutants is a good package, we discorded that after 1h of running it had gone over 350 mutants out of the 1844, this is just 18%. Due to this, we decided to cancel the mutation testing due to time constraints but keep the package in case we have the time in the future.
