# T-003-01 Research: Timeline Data Structure

## Objective

Implement the timeline data structure in moron-core/src/timeline.rs. The timeline is the backbone of video sequencing — it tracks ordered segments (narration, animation, silence, clip) and maps time to frame numbers.

## Current State

`moron-core/src/timeline.rs` is a stub with only a module doc comment.

## Downstream Consumers

### T-003-03 (Pacing Primitives)
- `m.beat()` → adds Silence { duration: 0.3 } to timeline
- `m.breath()` → adds Silence { duration: 0.8 }
- `m.wait(d)` → adds Silence { duration: d }
- Needs: `Timeline::add_segment(Segment::Silence { duration })`

### T-003-04 (Integration Tests)
- Tests frame mapping at 30fps and 60fps
- Tests that scenes produce valid timelines with expected segments
- Needs: `frame_at()`, `total_duration()`, iteration over segments

### M Facade (facade.rs)
- M will eventually hold a Timeline to record what scene methods produce
- Methods like `narrate()`, `play()`, `show()` will push segments
- Not wired in this ticket — just the data structure

## Required API (from ticket)

```
Timeline struct: ordered list of segments, total duration, current position
Segment enum: Narration { text, duration }, Animation { technique_id, duration }, Silence { duration }, Clip { path, duration }
TimelineBuilder: fluent API for constructing timelines
Methods: add_segment(), total_duration(), frame_at(time, fps), segments_in_range(start, end)
```

## Key Design Considerations

1. **Segment identification**: Segments need an index or id for technique correlation. A simple index-in-vec approach works. `technique_id` in Animation could be a string name since techniques have `.name()`.

2. **Time model**: All durations in f64 seconds. Frame mapping via `frame_at(time, fps)` → integer frame number. Standard: `floor(time * fps)` with clamp to valid range.

3. **Segment ranges**: `segments_in_range(start, end)` returns segments that overlap the time window. Requires computing cumulative start times from segment order.

4. **No audio data**: Timeline doesn't store audio bytes — just metadata. Audio synthesis is moron-voice's job. Narration segments record text + expected duration.

5. **Immutability after build**: TimelineBuilder produces a Timeline. Timeline itself can be append-only during scene construction, then frozen for rendering.

## Existing Patterns

- Other moron types use `#[derive(Debug, Clone)]` consistently
- Serde derives where serialization matters (Theme has it)
- Tests co-located in same file via `#[cfg(test)] mod tests`
