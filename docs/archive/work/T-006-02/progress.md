# T-006-02 Progress: Audio-Synced Timeline

## Status: Complete

All steps from the plan have been executed and verified.

## Steps Completed

### Step 1: Timeline Mutation Methods
- Added `Timeline::update_segment_duration(index, duration) -> bool`
- Added `Timeline::narration_indices() -> Vec<usize>`
- Added 4 unit tests: `update_segment_duration`, `update_segment_duration_out_of_bounds`,
  `narration_indices_mixed`, `narration_indices_empty`
- File: `moron-core/src/timeline.rs`

### Step 2: ElementRecord `segments_at_creation` Field
- Added `pub segments_at_creation: usize` to `ElementRecord`
- Updated `mint_element_with_meta()` to capture segment count at creation
- File: `moron-core/src/facade.rs`

### Step 3: ResolveDurationError and M Methods
- Added `ResolveDurationError` enum with `LengthMismatch` variant
- Implemented `Display` and `Error` traits
- Added `M::narration_count() -> usize`
- Added `M::resolve_narration_durations(&mut self, &[f64]) -> Result<(), ResolveDurationError>`
- Resolution updates narration segment durations then recomputes all element
  `created_at` timestamps using `segments_at_creation`
- File: `moron-core/src/facade.rs`

### Step 4: Re-exports
- Added `ResolveDurationError` to `pub use facade::` in lib.rs
- Added `ResolveDurationError` to `prelude` module
- File: `moron-core/src/lib.rs`

### Step 5: Tests
- `segments_at_creation_tracked` — verifies field tracks segment count
- `narration_count` — verifies convenience method
- `resolve_narration_durations` — full resolution with timestamp recomputation
- `resolve_narration_durations_length_mismatch` — error on wrong slice length
- `resolve_preserves_non_narration_timing` — silence durations unchanged
- `resolve_duration_error_display` — error message formatting
- `resolve_with_zero_narrations_succeeds` — empty slice on no-narration scene
- File: `moron-core/src/facade.rs`

### Step 6: Verification
- `cargo check` — passes (full workspace)
- `cargo test` — 98 core tests pass, all integration/e2e tests pass
- `cargo clippy` — no warnings

## Deviations from Plan

- Test `resolve_narration_durations` initially used "one word" (2 words) but
  expected 0.4s (1-word estimate). Fixed to use single-word narrations.
- No other deviations.

## Lines Added

- `timeline.rs`: ~30 lines production, ~40 lines test
- `facade.rs`: ~65 lines production, ~75 lines test
- `lib.rs`: 2 lines (re-exports)
- Total: ~97 lines production, ~115 lines test
