# T-009-01 Plan: Animation Execution Engine

## Step 1: Add AnimationRecord and storage to M

**File:** `moron-core/src/facade.rs`

1. Add `AnimationRecord` struct after `ElementRecord`:
   - `technique: Box<dyn moron_techniques::Technique>`
   - `target_ids: Vec<u64>`
   - `segment_index: usize`
2. Add `animations: Vec<AnimationRecord>` field to `M`.
3. Initialize `animations: Vec::new()` in `M::new()`.
4. Add `pub(crate) fn animations(&self) -> &[AnimationRecord]` accessor.

**Verify:** `cargo check` passes. Existing tests pass.

## Step 2: Modify play() to store technique and target

**File:** `moron-core/src/facade.rs`

1. Change `play()` signature to `impl moron_techniques::Technique + 'static`.
2. Before `self.timeline.add_segment()`:
   - Capture `segment_index = self.timeline.segments().len()`.
   - Determine `target_ids` from `self.elements.last()`.
3. After `self.timeline.add_segment()`:
   - Push `AnimationRecord { technique: Box::new(technique), target_ids, segment_index }`.

**Verify:** `cargo check` passes. `play_records_animation_segments` test still passes. Add a quick test that `m.animations()` is populated after `play()`.

## Step 3: Implement apply_animations() in frame.rs

**File:** `moron-core/src/frame.rs`

1. Add `use crate::facade::AnimationRecord;` (or access via M).
2. Add helper function:
   ```
   fn apply_animations(m: &M, time: f64, elements: &mut [ElementState])
   ```
3. Logic:
   - Build `HashMap<u64, usize>` mapping element ID → index in elements vec.
   - For each animation in `m.animations()`:
     - Compute segment start = sum of `m.timeline().segments()[..seg_idx]` durations.
     - Compute segment end = start + `m.timeline().segments()[seg_idx].duration()`.
     - Compute progress: before start → 0.0, after end → 1.0, during → (time - start) / (end - start).
     - Call `record.technique.apply(progress)`.
     - For each target_id, look up element. If visible, overwrite visual fields from TechniqueOutput.
4. Call `apply_animations(m, clamped_time, &mut elements)` in `compute_frame_state()` before building the return value.

**Verify:** `cargo check` passes. All existing frame tests pass (no animations → no changes to behavior).

## Step 4: Add animation execution tests

**File:** `moron-core/src/frame.rs` (in `mod tests`)

Tests:
1. `animation_fade_in_progress` — create element, play FadeIn, check opacity at t=0/mid/end of animation segment.
2. `animation_before_start_shows_initial_state` — check element has apply(0.0) output before animation segment begins.
3. `animation_after_end_shows_final_state` — check element has apply(1.0) output after animation segment ends.
4. `no_animation_retains_defaults` — element without play() keeps opacity=1.0.
5. `animation_fade_up_produces_translation` — FadeUp at midpoint has both opacity and translate_y.
6. `animation_not_applied_to_invisible_element` — element cleared before animation → stays at invisible defaults.
7. `animation_with_preceding_segments` — animation after narration+silence, verify correct time computation.

**Verify:** `cargo test` passes, `cargo clippy` clean.

## Step 5: Verify existing tests and e2e

Run full test suite:
- `cargo test` — all unit tests including facade, frame, technique, e2e.
- `cargo clippy` — no warnings.

Check that DemoScene still works correctly — it uses `play(FadeIn)` and `play(FadeUp)`, so now frame states will have actual animation values instead of hardcoded defaults. Existing e2e tests only check serialization and structure, not specific opacity values, so they should pass.

## Testing Strategy

- **Unit tests** in `frame.rs`: Core animation progress computation at boundary conditions (start, mid, end, before, after).
- **Existing tests** as regression: All ~170 existing tests must continue to pass.
- **No e2e changes needed:** E2e tests check JSON structure and pipeline integrity, not specific visual values.
