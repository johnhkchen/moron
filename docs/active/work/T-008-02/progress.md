# T-008-02 Progress: clippy-pedantic-techniques

## Status: Complete

All 34 clippy pedantic warnings in moron-techniques resolved. Zero warnings, all tests pass, workspace clippy clean.

## Changes Made

| File | Changes |
|------|---------|
| technique.rs | `#[must_use]` on `ease()`, `0.984375` → `0.984_375` |
| reveals.rs | Backticked doc names, `&'static str` on FadeIn/FadeUp `name()` |
| motion.rs | Backticked doc names, `&'static str` on Slide/Scale `name()` |
| staging.rs | Backticked doc names, `#[must_use]` on builders, `&'static str` on `name()`, `#[allow]` on casts |
| emphasis.rs | Backticked doc names |
| camera.rs | Backticked doc names |
| transitions.rs | Backticked doc names |
| data.rs | Backticked doc names, `#[must_use]` on `current_value()`, `&'static str` on `name()` |

## Verification

- `cargo clippy -p moron-techniques -- -W clippy::pedantic` — 0 warnings
- `cargo test -p moron-techniques` — 17 tests pass (14 unit + 3 integration)
- `cargo clippy` — workspace clean
