# T-006-02 Structure: Audio-Synced Timeline

## Overview

Two files modified, zero files created. All changes are additive to existing
modules. No new modules, no new crates, no public API signature changes to
existing functions.

## File Changes

### 1. `moron-core/src/timeline.rs` — Mutation API

**Add two methods to `Timeline`:**

```
pub fn update_segment_duration(&mut self, index: usize, duration: f64) -> bool
```
- Mutates `self.segments[index]` duration in place.
- Returns `false` if index is out of bounds. Returns `true` on success.
- Uses a match on the Segment enum to replace the duration field regardless
  of variant. This keeps it generic even though the caller only targets
  Narration segments today.

```
pub fn narration_indices(&self) -> Vec<usize>
```
- Iterates `self.segments` with `.enumerate()`.
- Filters for `Segment::Narration { .. }`.
- Returns collected indices.

**Add tests in the existing `mod tests` block:**
- `test_update_segment_duration` — create timeline, update a segment, verify
  `total_duration()` reflects the change.
- `test_update_segment_duration_out_of_bounds` — returns false.
- `test_narration_indices` — mixed segment types, verify correct indices.
- `test_narration_indices_empty` — no narration segments, returns empty vec.

Estimated: ~25 lines production, ~40 lines test.

### 2. `moron-core/src/facade.rs` — Resolution API

**Modify `ElementRecord` struct:**
- Add field `pub segments_at_creation: usize`.
- This records `self.timeline.segments().len()` at element creation time.

**Modify `mint_element_with_meta()`:**
- Capture `let segments_at_creation = self.timeline.segments().len();`
- Pass it into the `ElementRecord` initialization.

**Add error type:**
```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveDurationError {
    LengthMismatch { expected: usize, provided: usize },
}
impl std::fmt::Display for ResolveDurationError { ... }
impl std::error::Error for ResolveDurationError {}
```

**Add methods to `M`:**

```
pub fn narration_count(&self) -> usize
```
- Returns `self.timeline.narration_indices().len()`.
- Convenience for callers to know how many TTS durations to provide.

```
pub fn resolve_narration_durations(&mut self, durations: &[f64])
    -> Result<(), ResolveDurationError>
```
Steps:
1. `let indices = self.timeline.narration_indices();`
2. Validate `durations.len() == indices.len()`, return error if not.
3. For each `(i, &dur)` in `indices.iter().zip(durations.iter())`:
   call `self.timeline.update_segment_duration(*i, dur)`.
4. Recompute element timestamps: for each element in `self.elements`,
   compute the sum of durations of `self.timeline.segments()[0..rec.segments_at_creation]`
   and assign to `rec.created_at`.

**Update re-exports in `moron-core/src/lib.rs`:**
- Add `ResolveDurationError` to the `pub use facade::` line.
- Add `ResolveDurationError` to `prelude` module's facade re-export.

**Add tests in the existing `mod tests` block:**
- `test_segments_at_creation_tracked` — verify the field is set correctly
  as elements are interleaved with segments.
- `test_resolve_narration_durations` — build a scene with two narrations
  and elements after each, resolve with new durations, verify `created_at`
  and `total_duration` shift correctly.
- `test_resolve_narration_durations_length_mismatch` — wrong slice length
  returns `ResolveDurationError::LengthMismatch`.
- `test_resolve_preserves_non_narration_timing` — silence/animation
  durations remain unchanged after resolve.
- `test_fallback_without_resolve` — existing WPM estimates work without
  calling resolve (existing tests already cover this implicitly).

Estimated: ~65 lines production, ~60 lines test.

### 3. `moron-core/src/lib.rs` — Re-export update

One line change: add `ResolveDurationError` to the facade re-export.
One line change: add `ResolveDurationError` to the prelude re-export.

## Module Boundaries

- `Timeline` gains mutation API but remains dumb storage. It does not know
  about elements or timestamps.
- `M` owns the coordination logic: it knows about both timeline and elements,
  and handles the recomputation.
- `frame.rs`, `build.rs` — zero changes. They read from `M` and `Timeline`
  through existing APIs. The resolved data flows through the same interfaces.

## Public Interface Summary

| Type | New public items |
|------|------------------|
| `Timeline` | `update_segment_duration()`, `narration_indices()` |
| `M` | `resolve_narration_durations()`, `narration_count()` |
| `ElementRecord` | `segments_at_creation` field (pub(crate)) |
| `facade` module | `ResolveDurationError` enum |

## Ordering Constraints

1. Timeline methods must exist before facade methods (facade calls timeline).
2. `segments_at_creation` field must be added before `resolve_narration_durations`
   (the resolve method reads it).
3. Tests can be added alongside each change.

## What Is NOT Changed

- `Segment` enum — no new variants, no Option wrapping.
- `build_video()` signature — stays `&M`.
- `compute_frame_state()` — reads `created_at` as before, no changes needed.
- `TimelineBuilder` — no changes.
- No new files or modules.
