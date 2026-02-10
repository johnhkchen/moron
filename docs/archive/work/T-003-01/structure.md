# T-003-01 Structure: Timeline Data Structure

## Files Modified

### moron-core/src/timeline.rs (rewrite from stub)

Module structure:
```
timeline.rs
├── Segment enum (4 variants)
├── impl Segment { fn duration() }
├── Timeline struct { segments, fps }
├── impl Timeline
│   ├── new(fps)
│   ├── add_segment()
│   ├── segments() -> &[Segment]
│   ├── total_duration() -> f64
│   ├── total_frames() -> u32
│   ├── frame_at(time) -> u32
│   └── segments_in_range(start, end) -> Vec<(f64, &Segment)>
├── TimelineBuilder struct
├── impl TimelineBuilder
│   ├── new()
│   ├── fps()
│   ├── narration()
│   ├── animation()
│   ├── silence()
│   ├── clip()
│   └── build() -> Timeline
└── #[cfg(test)] mod tests (8+ tests)
```

### moron-core/src/lib.rs

Add `pub use timeline::{Segment, Timeline, TimelineBuilder};` to root re-exports.
Add Timeline/Segment/TimelineBuilder to prelude.

## Public API

| Type | Kind | Key Methods |
|------|------|-------------|
| Segment | enum | duration() |
| Timeline | struct | new, add_segment, total_duration, total_frames, frame_at, segments_in_range, segments |
| TimelineBuilder | struct | new, fps, narration, animation, silence, clip, build |

## Dependencies

Uses `std::path::PathBuf` for Clip variant. No new crate dependencies.
