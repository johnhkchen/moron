# T-003-01 Design: Timeline Data Structure

## Chosen Approach: Simple Vec-Based Timeline

### Segment Enum

```rust
enum Segment {
    Narration { text: String, duration: f64 },
    Animation { name: String, duration: f64 },
    Silence { duration: f64 },
    Clip { path: PathBuf, duration: f64 },
}
```

All variants carry `duration: f64` (seconds). This makes total_duration() a simple sum.

`Animation.name` is a String (not a technique reference) because the timeline is a data record of what was sequenced, not an executor. The technique objects live in M/renderer scope.

### Timeline Struct

```rust
struct Timeline {
    segments: Vec<Segment>,
    fps: u32,
}
```

`fps` is stored on the timeline so frame calculations are consistent. Default 30fps.

### TimelineBuilder

Fluent builder pattern:
```rust
TimelineBuilder::new()
    .fps(60)
    .narration("Hello", 2.0)
    .silence(0.3)
    .animation("FadeIn", 0.5)
    .build() -> Timeline
```

Builder is the recommended way for tests and direct construction. M facade will use `Timeline::add_segment()` internally during scene building.

### Key Methods

- `add_segment(&mut self, segment: Segment)` — append a segment
- `total_duration(&self) -> f64` — sum of all segment durations
- `frame_at(&self, time: f64, fps: u32) -> u32` — floor(time * fps), clamped
- `total_frames(&self) -> u32` — frame count at timeline's fps
- `segments_in_range(&self, start: f64, end: f64) -> Vec<(f64, &Segment)>` — returns (start_time, segment) pairs for overlapping segments
- `segments(&self) -> &[Segment]` — direct access to segment list

### Alternatives Rejected

**Indexed segments with HashMap** — unnecessary complexity. Vec order is the timeline order.

**Segment trait object** — `Box<dyn Segment>`. Over-engineered. The enum is closed and known.

**Separate duration tracking** — caching total_duration. Premature optimization. Summing a Vec<f64> is trivial.

### frame_at() Semantics

- `frame_at(0.0) = 0` (first frame)
- `frame_at(t) = floor(t * fps)` for t in range
- Clamped: `frame_at(t >= total_duration) = total_frames - 1` (last valid frame, or 0 for empty)
- Negative time returns 0

### segments_in_range() Semantics

A segment overlaps [start, end) if its time range [seg_start, seg_start + seg_duration) intersects. Returns Vec of (segment_start_time, &Segment) tuples.
