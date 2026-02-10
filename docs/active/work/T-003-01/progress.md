# T-003-01 Progress: Timeline Data Structure

## Completed

- [x] Step 1: Segment enum (4 variants) and Timeline struct with core methods
- [x] Step 2: TimelineBuilder with fluent API
- [x] Step 3: Updated lib.rs re-exports and prelude
- [x] Step 4: 10 unit tests covering all methods
- [x] Step 5: cargo check, cargo test (30 pass), cargo clippy — all green
- [x] Step 6: Ready for commit

## Changes Made

### moron-core/src/timeline.rs (rewritten from stub)
- Segment enum: Narration, Animation, Silence, Clip — all with duration
- Timeline struct: segments vec, fps, add_segment, total_duration, total_frames, frame_at, segments_in_range
- TimelineBuilder: fluent builder with fps, narration, animation, silence, clip, build
- Default and Default impls for both
- 10 unit tests

### moron-core/src/lib.rs
- Added `pub use timeline::{Segment, Timeline, TimelineBuilder}` to root
- Added same to prelude module

## Deviations from Plan

None.
