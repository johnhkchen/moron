# T-006-02 Plan: Audio-Synced Timeline

## Step 1: Add Timeline Mutation Methods

**File:** `moron-core/src/timeline.rs`

Add `update_segment_duration(index: usize, duration: f64) -> bool`:
- Match on `&mut self.segments[index]` (after bounds check).
- For each variant, replace the duration field.
- Return true on success, false if out of bounds.

Add `narration_indices() -> Vec<usize>`:
- `self.segments.iter().enumerate().filter_map(...)`.
- Filter for `Segment::Narration { .. }`.

**Tests:**
- `test_update_segment_duration`: build timeline with 3 segments, update
  middle one, verify `total_duration()` changes.
- `test_update_segment_duration_out_of_bounds`: index beyond length returns
  false, timeline unchanged.
- `test_narration_indices`: timeline with narration-silence-narration-animation,
  returns `[0, 2]`.
- `test_narration_indices_empty`: timeline with only silence, returns `[]`.

**Verification:** `cargo test -p moron-core timeline::tests`

## Step 2: Add `segments_at_creation` to ElementRecord

**File:** `moron-core/src/facade.rs`

Add `pub segments_at_creation: usize` field to `ElementRecord`.

Update `mint_element_with_meta()`:
- Before pushing the record, capture `self.timeline.segments().len()`.
- Include it in the `ElementRecord` initializer.

**Verification:** `cargo check -p moron-core` (no test needed yet — the field
is exercised in step 4).

## Step 3: Add ResolveDurationError and M Methods

**File:** `moron-core/src/facade.rs`

Add the error type:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveDurationError {
    LengthMismatch { expected: usize, provided: usize },
}
```
Implement `Display` and `Error` for it.

Add `narration_count(&self) -> usize`:
- Delegates to `self.timeline.narration_indices().len()`.

Add `resolve_narration_durations(&mut self, durations: &[f64]) -> Result<(), ResolveDurationError>`:
1. Get narration indices from timeline.
2. Validate length match.
3. Update each narration segment duration via `timeline.update_segment_duration()`.
4. Recompute `created_at` for every element:
   ```rust
   for rec in &mut self.elements {
       rec.created_at = self.timeline.segments()[..rec.segments_at_creation]
           .iter()
           .map(|s| s.duration())
           .sum();
   }
   ```

**Verification:** `cargo check -p moron-core`

## Step 4: Update Re-exports

**File:** `moron-core/src/lib.rs`

Add `ResolveDurationError` to `pub use facade::` line.
Add `ResolveDurationError` to `prelude` module.

**Verification:** `cargo check -p moron-core`

## Step 5: Add Tests for Resolution Logic

**File:** `moron-core/src/facade.rs` (existing `mod tests` block)

Tests to add:

`test_segments_at_creation_tracked`:
- Create M, add title (0 segments), narrate (adds segment), add show
  (1 segment), narrate again, add section (3 segments).
- Verify each element's `segments_at_creation` matches expected count.

`test_resolve_narration_durations`:
- Build scene: title, narrate("A"), show("B"), narrate("C"), section("D").
- Initial: title.created_at=0, show.created_at=0.4 (WPM), section.created_at=0.8.
- Resolve with [1.0, 2.0].
- After: title.created_at=0, show.created_at=1.0, section.created_at=3.0.
- Total duration shifts from WPM sum to 3.0 + any silence.

`test_resolve_length_mismatch`:
- Two narrations, provide 3 durations -> `LengthMismatch { expected: 2, provided: 3 }`.

`test_resolve_preserves_non_narration`:
- Scene with narration + silence + narration. Resolve narrations.
- Silence duration unchanged. Total = narr1 + silence + narr2.

`test_narration_count`:
- Build scene with known narration count, verify `narration_count()`.

**Verification:** `cargo test -p moron-core facade::tests`

## Step 6: Full Verification

Run the complete test suite:
- `cargo check` — full workspace compiles.
- `cargo test` — all existing + new tests pass.
- `cargo clippy` — no warnings.

## Summary

| Step | File(s) | Lines (est.) | Depends on |
|------|---------|-------------|------------|
| 1 | timeline.rs | ~25 prod, ~40 test | None |
| 2 | facade.rs | ~5 prod | None |
| 3 | facade.rs | ~40 prod | Steps 1, 2 |
| 4 | lib.rs | ~2 | Step 3 |
| 5 | facade.rs | ~60 test | Steps 1-3 |
| 6 | (verification) | 0 | Steps 1-5 |

Total: ~70 lines production code, ~100 lines tests.
