# T-003-01 Plan: Timeline Data Structure

## Steps

### Step 1: Implement Segment enum and Timeline struct
- Define Segment with 4 variants, each carrying duration
- Implement `Segment::duration()` method
- Define Timeline with segments vec and fps
- Implement core methods: new, add_segment, segments, total_duration, total_frames, frame_at, segments_in_range
- Verify: `cargo check -p moron-core`

### Step 2: Implement TimelineBuilder
- Fluent builder with fps, narration, animation, silence, clip, build methods
- Verify: `cargo check -p moron-core`

### Step 3: Update lib.rs re-exports and prelude
- Add Timeline, Segment, TimelineBuilder to root re-exports
- Add to prelude module
- Verify: `cargo check`

### Step 4: Write unit tests
- empty_timeline_has_zero_duration
- add_segments_and_check_duration
- frame_at_basic_mapping
- frame_at_clamps_out_of_range
- segments_in_range_overlap
- builder_produces_correct_timeline
- builder_default_fps
- segment_duration_method
- Verify: `cargo test -p moron-core`

### Step 5: Full workspace verification
- `cargo check`, `cargo test`, `cargo clippy`

### Step 6: Commit and update ticket
