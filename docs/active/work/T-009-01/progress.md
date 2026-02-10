# T-009-01 Progress: Animation Execution Engine

## Completed

### Step 1: AnimationRecord type and storage
- Added `AnimationRecord` struct in `facade.rs` with `technique: Box<dyn Technique>`, `target_ids: Vec<u64>`, `segment_index: usize`
- Added `animations: Vec<AnimationRecord>` field to `M`
- Added `pub(crate) fn animations()` accessor

### Step 2: Modified play() to store technique and target
- Changed signature to `impl Technique + 'static`
- Captures segment index before adding segment
- Determines target from most recently created element
- Pushes AnimationRecord after adding timeline segment

### Step 3: Implemented apply_animations() in frame.rs
- Added `apply_animations(m, time, elements)` helper function
- Builds ID→index HashMap for O(1) element lookup
- Computes per-animation segment start/end times from timeline
- Computes progress (0.0 before, linear during, 1.0 after)
- Applies TechniqueOutput to visible target elements
- Called in `compute_frame_state()` before layout assignment

### Step 4: Animation execution tests (7 tests)
- `animation_fade_in_progress` — FadeIn at start/mid/end
- `animation_before_start_shows_initial_state` — pending animation shows apply(0.0)
- `animation_after_end_shows_final_state` — completed animation shows apply(1.0)
- `no_animation_retains_defaults` — no play() → opacity 1.0
- `animation_fade_up_produces_translation` — FadeUp opacity + translate_y
- `animation_not_applied_to_invisible_element` — cleared element ignores animation
- `animation_with_preceding_segments` — correct time computation after narration

### Step 5: Verification
- `cargo test` (excluding pre-existing ffmpeg failures): 123+ tests pass
- `cargo clippy`: clean (0 warnings for new code)
- All 32 frame tests pass (26 existing + 6 new)
- All facade, demo, timeline, e2e tests pass unchanged

## Deviations from Plan
- None. All steps executed as planned.

## Files Modified
- `moron-core/src/facade.rs`: Added AnimationRecord, animations field/accessor, modified play()
- `moron-core/src/frame.rs`: Added apply_animations() function, 7 new tests, updated comment
