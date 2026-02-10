# T-006-02 Design: Audio-Synced Timeline

## Problem Statement

Narration durations are estimated at 150 WPM. Real TTS produces audio whose
duration differs from the estimate. The timeline, element timestamps, and frame
state must reflect actual TTS durations when available, while preserving the
WPM fallback when TTS is not.

## Option A: Mutate Timeline Segments In-Place

Add a method to `Timeline` that replaces a narration segment's duration. After
scene build, walk the segments, call TTS for each narration, and update
durations. Then recompute element `created_at` timestamps.

**Pros:** Minimal API surface change. Timeline stays as a `Vec<Segment>`.
**Cons:** Requires mutable access to segment durations (new API on Timeline).
Requires a separate pass to recompute element `created_at` values, which means
`M` needs a `resolve_durations()` method that walks both segments and elements.
The coupling between segment order and element timestamps is implicit.

## Option B: Two-Phase Build with Deferred Duration Marker

Change `Segment::Narration` to carry an `Option<f64>` or a separate
`DeferredDuration` wrapper. During scene build, narrate() records `None` as the
duration. A resolution pass fills in the real value. Element timestamps are
computed only after resolution.

**Pros:** Makes the deferred nature explicit in the type system.
**Cons:** Breaks every piece of code that reads `duration` from Narration
segments -- `total_duration()`, `frame_at()`, `segments_in_range()`, and all
tests. Requires pervasive `Option` handling or a "resolved" vs "unresolved"
timeline distinction. Much larger change surface.

## Option C: Resolution Method on M (Chosen)

Keep `Segment::Narration` with a concrete `f64` duration (WPM estimate as
default). Add a `resolve_narration_durations()` method on `M` that:

1. Accepts a slice of replacement durations for narration segments
2. Walks the timeline segments, updating narration durations
3. Recomputes all element `created_at` timestamps based on the new timeline

The caller (build pipeline in T-006-03) is responsible for calling TTS and
collecting the durations. This ticket provides the mechanism; T-006-03 wires
it to the actual backend.

**Pros:**
- No type-level changes to `Segment` -- backward compatible
- Narration always has a valid duration (WPM estimate) so all existing code
  works without resolution
- Clean separation: T-006-02 provides duration resolution, T-006-03 provides
  TTS synthesis
- Element timestamps are recomputed atomically in one pass
- `build_video` can remain `&M` if resolution happens before it is called
  (i.e., the caller resolves, then passes immutable M to build)

**Cons:**
- The slice of durations must be in narration-segment order, which is an
  implicit contract. Mitigated by returning narration count for validation.

## Why Not Option B

Option B is the "purest" design -- making deferred duration a first-class
concept. But it violates the project constraint of <15K lines and solo
maintainability. Every consumer of `Segment::duration()` would need to handle
the unresolved case. The benefit (type safety for deferred durations) does not
justify the cost in a codebase where the resolution is always a single well-
defined step in the pipeline.

## Detailed Design

### 1. Timeline: Add `update_segment_duration(index, new_duration)`

A new method on `Timeline`:

```rust
pub fn update_segment_duration(&mut self, index: usize, duration: f64) -> bool
```

Returns `true` if the index was valid and the duration was updated. This is the
minimal mutation API -- no full segment replacement, just duration adjustment.

### 2. Timeline: Add `narration_indices()` Helper

```rust
pub fn narration_indices(&self) -> Vec<usize>
```

Returns the indices of all `Segment::Narration` variants, in order. This lets
the caller match a list of TTS durations to the correct segments without
manually walking the segment vec.

### 3. M: Add `resolve_narration_durations(durations: &[f64])`

```rust
pub fn resolve_narration_durations(&mut self, durations: &[f64]) -> Result<(), ResolveDurationError>
```

Steps:
1. Call `self.timeline.narration_indices()` to get narration positions
2. Validate `durations.len() == narration_indices.len()`
3. For each (index, duration) pair, call `timeline.update_segment_duration()`
4. Recompute all element `created_at` timestamps (see below)

Error type covers: length mismatch.

### 4. M: Recompute Element Timestamps

After updating narration durations, element `created_at` values are stale.
The fix: replay the element creation order against the new timeline.

Elements are stored in creation order in `M.elements`. Each element was created
at a point when the timeline had a certain cumulative duration. We need to
reconstruct which segments had been added at each element's creation point.

**Key insight:** Elements are interleaved with segments. The ordering is:
```
m.title("A");          // element 0 created, timeline has N segments
m.narrate("text");     // segment added
m.show("B");           // element 1 created, timeline has N+1 segments
```

The element's `created_at` equals `timeline.total_duration()` *at the time of
creation* -- meaning the sum of segments added *before* this element.

To reconstruct this, we need to know how many segments existed when each
element was created. Two approaches:

**Approach 4a: Record segment count at element creation.** Add a
`segments_at_creation: usize` field to `ElementRecord`. When minting an element,
store `self.timeline.segments().len()`. During resolution, recompute
`created_at` as the sum of durations of `segments[0..segments_at_creation]`.

**Approach 4b: Record based on old created_at.** Walk segments to find the
cumulative duration that matches each element's old `created_at`, determine
which segments preceded it, then recompute.

Approach 4a is simpler and exact. Approach 4b is fragile with floating point.

**Decision: Approach 4a.** Add `segments_at_creation: usize` to `ElementRecord`.

### 5. Fallback Behavior

When TTS is unavailable, `resolve_narration_durations()` is never called.
The timeline retains WPM-estimated durations. All existing behavior is
preserved. No code path requires resolution to have happened.

### 6. Error Type

```rust
pub enum ResolveDurationError {
    LengthMismatch { expected: usize, provided: usize },
}
```

Minimal. Only one thing can go wrong at this level (wrong number of durations).

### 7. Public API Changes Summary

| Location | Change |
|---|---|
| `Timeline` | Add `update_segment_duration(index, f64) -> bool` |
| `Timeline` | Add `narration_indices() -> Vec<usize>` |
| `ElementRecord` | Add `segments_at_creation: usize` field |
| `M` | Add `resolve_narration_durations(&mut self, &[f64]) -> Result<()>` |
| `M` | Add `narration_count() -> usize` convenience method |
| `facade` | Add `ResolveDurationError` enum |

### 8. Interaction with T-006-03

T-006-03's pipeline flow becomes:
1. `Scene::build(&mut m)` -- scene records narrations with WPM estimates
2. Collect narration texts: `m.timeline().segments().iter().filter_map(narration)`
3. For each text, call `backend.synthesize(text)` -> `AudioClip`
4. Extract durations: `clips.iter().map(|c| c.duration()).collect::<Vec<f64>>()`
5. `m.resolve_narration_durations(&durations)?` -- timeline + elements updated
6. `build_video(&m, config)` -- renders with correct timing
7. Audio assembly uses the real clips (not silence)

The `build_video` signature stays `&M`. Resolution happens before the call.

### 9. Test Strategy

- `update_segment_duration`: unit test on Timeline directly
- `narration_indices`: unit test with mixed segment types
- `resolve_narration_durations`: test with known durations, verify timeline
  total and element `created_at` values shift correctly
- Length mismatch error: test with wrong-size slice
- Fallback: existing tests pass without calling resolve (WPM path)
- Frame state: test that `compute_frame_state` after resolution produces
  correct element visibility at new timestamps

### 10. Lines-of-Code Estimate

- `timeline.rs`: ~30 lines (two new methods + tests)
- `facade.rs`: ~60 lines (resolve method, narration_count, error type,
  segments_at_creation field, updated mint method, tests)
- Total new/modified: ~90 lines of production code, ~100 lines of tests
