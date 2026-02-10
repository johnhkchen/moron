# T-009-01 Design: Animation Execution Engine

## Problem

`M::play()` discards technique objects. `compute_frame_state()` never applies animation transforms. Elements show opacity=1.0, scale=1.0, zero translation regardless of what techniques were played.

## Approaches Considered

### A: Store techniques on Timeline segments

Modify `Segment::Animation` to hold `Box<dyn Technique>` alongside name/duration. At frame computation time, walk segments to find overlapping animations and call `apply()`.

**Pros:** Minimal new types. Techniques live where they semantically belong (on the timeline).
**Cons:** `Segment` is `Clone` and `Debug` — `Box<dyn Technique>` breaks `Clone`. `Segment` is public API in `timeline.rs`. Would require `Technique: Clone` (not trivially dyn-safe) or removing `Clone` from `Segment` (breaking change). Also mixes targeting info (which elements?) into timeline segments, which is a separate concern.

**Rejected:** Too invasive to Segment's existing derive traits and public API.

### B: Separate animation registry on M

Add a `Vec<AnimationRecord>` field to `M`. Each record stores:
- `Box<dyn Technique>` (the technique object to call `apply()` on)
- Target element ID(s)
- Timeline segment index (to compute time window after duration resolution)

`play()` pushes both a `Segment::Animation` (for timeline pacing) and an `AnimationRecord` (for execution). `compute_frame_state()` queries the registry to find animations active at the current time and applies their output to elements.

**Pros:** No changes to `Segment` or `Timeline`. No new trait requirements on `Technique`. Animation targeting is explicit. Duration resolution works via segment index (same pattern as `ElementRecord.segments_at_creation`). Clean separation of pacing (timeline) from execution (registry).
**Cons:** Slight duplication — animation duration lives on both the segment and the technique object. But the segment is the source of truth after resolution, and the technique's `duration()` is only needed for `apply()` progress normalization (which uses the segment duration anyway).

**Chosen.** This is the minimal, non-breaking approach.

### C: Store animations on ElementRecord

Add an `animations: Vec<Box<dyn Technique>>` field to `ElementRecord`, binding techniques directly to their target elements.

**Pros:** Direct element-to-technique association.
**Cons:** Conflates element identity with animation behavior. An animation's time window comes from the timeline (segment position), not the element. Stagger (T-009-02) applies one technique to multiple elements, which doesn't fit a per-element ownership model. Also makes `ElementRecord` non-Clone.

**Rejected:** Wrong ownership model for the data flow.

## Chosen Design: Separate Animation Registry (B)

### New Type: `AnimationRecord`

```rust
pub(crate) struct AnimationRecord {
    pub technique: Box<dyn Technique>,
    pub target_ids: Vec<u64>,
    pub segment_index: usize,
}
```

- `technique`: The boxed technique object. Called at frame time via `technique.apply(progress)`.
- `target_ids`: Element IDs this animation applies to. For now, always a single element (the most recently created). T-009-02 will extend this for Stagger.
- `segment_index`: Index into `timeline.segments()`. Used to compute the animation's absolute time window: start = sum of durations of segments[..index], end = start + segments[index].duration(). This survives duration resolution because the segment index is stable.

### Changes to `M`

Add field: `animations: Vec<AnimationRecord>`.

Modify `play()`:
1. Determine target: `self.elements.last().map(|e| e.id)`. If no elements exist, record animation with empty targets (pacing-only).
2. Record the segment index: `self.timeline.segments().len()` (before adding the segment).
3. Add the animation segment to the timeline (as today).
4. Push an `AnimationRecord` with the boxed technique, target IDs, and segment index.

Signature change: `play()` takes `impl Technique + 'static` (needed for `Box<dyn Technique>`). All concrete techniques are `'static` already.

Add accessor: `pub(crate) fn animations(&self) -> &[AnimationRecord]`.

### Changes to `compute_frame_state()`

After building the initial `Vec<ElementState>` with default visuals, apply animations:

1. For each `AnimationRecord` in `m.animations()`:
   a. Compute the segment's start time: sum of `segments[..record.segment_index].duration()`.
   b. Compute end time: start + `segments[record.segment_index].duration()`.
   c. If `clamped_time` is within `[start, end)`:
      - Compute `progress = (clamped_time - start) / (end - start)`.
      - Call `record.technique.apply(progress)` to get `TechniqueOutput`.
      - For each target ID, find the matching `ElementState` and apply the output.
   d. If `clamped_time < start` (animation hasn't started):
      - Apply `technique.apply(0.0)` — the pre-animation state.
   e. If `clamped_time >= end` (animation completed):
      - Apply `technique.apply(1.0)` — the post-animation state.

"Apply the output" means overwriting the five visual fields:
```rust
elem.opacity = output.opacity;
elem.translate_x = output.translate_x;
elem.translate_y = output.translate_y;
elem.scale = output.scale;
elem.rotation = output.rotation;
```

### Pre/Post Animation Behavior

Key design decision: what happens to an element **before** its animation starts and **after** it ends?

- **Before animation:** Apply `technique.apply(0.0)`. For FadeIn this means opacity=0.0, which is correct — the element should be invisible before it fades in. The element is still `visible: true` (it was created), but has zero opacity.
- **After animation:** Apply `technique.apply(1.0)`. For FadeIn this means opacity=1.0, scale=1.0 — fully visible. This is the "resting state" after the animation completes.
- **No animation:** Elements without any animation record retain the current defaults (opacity=1.0 when visible).

This means the `visible` field remains a creation-time/end-time flag, while animation output controls the visual appearance. An element can be `visible: true` but `opacity: 0.0` if its FadeIn hasn't started.

### Applying only when element is visible

Animation output should only be applied when `elem.visible` is true. If an element isn't visible (hasn't been created yet or has been cleared), it keeps the invisible defaults (opacity=0, scale=0).

### Multiple animations on the same element

If a scene plays two animations targeting the same element (e.g., FadeIn then Slide), the later animation's output overwrites the earlier one's. This is correct — at any point in time, only one animation is "active" for a given element (they occupy different time windows on the timeline). If they overlap (unlikely but possible), last-write-wins is acceptable for now.

## What This Design Does NOT Do

- **Per-item Stagger animation:** T-009-02 scope. This ticket treats Stagger as a single whole-element animation.
- **Composite/layered animations:** Not needed yet. One active animation per element at a time.
- **Animation on narration segments:** Narrations aren't visual elements. Only elements with IDs get animation.
- **Changes to Segment enum:** No modifications to `Segment::Animation`'s fields.
- **Changes to Timeline:** No new methods needed on `Timeline`.
