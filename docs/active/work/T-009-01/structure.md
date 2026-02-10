# T-009-01 Structure: Animation Execution Engine

## Files Modified

### `moron-core/src/facade.rs`

**New type: `AnimationRecord`** (after `ElementRecord`)
```rust
pub(crate) struct AnimationRecord {
    pub technique: Box<dyn moron_techniques::Technique>,
    pub target_ids: Vec<u64>,
    pub segment_index: usize,
}
```

**New field on `M`:**
```rust
animations: Vec<AnimationRecord>,
```

Initialize to `Vec::new()` in `M::new()`.

**Modified method: `play()`**
- Signature: `pub fn play(&mut self, technique: impl moron_techniques::Technique + 'static)`
- Before adding segment: capture `segment_index = self.timeline.segments().len()`
- Determine target: `self.elements.last().map(|e| vec![e.id]).unwrap_or_default()`
- After adding segment: push `AnimationRecord { technique: Box::new(technique), target_ids, segment_index }`

**New accessor:**
```rust
pub(crate) fn animations(&self) -> &[AnimationRecord] {
    &self.animations
}
```

### `moron-core/src/frame.rs`

**Modified function: `compute_frame_state()`**

After the initial `elements` vec is built (line ~149), add animation application:

1. Build an element ID → index lookup map for O(1) access.
2. For each animation record from `m.animations()`:
   - Compute segment start time (sum of preceding segment durations).
   - Compute segment end time (start + segment duration).
   - Compute progress based on clamped_time relative to the segment window.
   - Call `record.technique.apply(progress)`.
   - For each target ID, if the element is visible, overwrite its visual fields.

Extract this into a helper function:
```rust
fn apply_animations(m: &M, time: f64, elements: &mut [ElementState])
```

### No files created or deleted

All changes are modifications to existing files.

## Module Boundaries

- `AnimationRecord` is `pub(crate)` — only `facade.rs` and `frame.rs` need it.
- The `animations()` accessor is `pub(crate)` — same as `elements()`.
- `Box<dyn Technique>` crosses the crate boundary (moron-techniques → moron-core), but `moron-core` already depends on `moron-techniques`.
- No changes to public API. `play()` signature change (`+ 'static`) is invisible to callers since all concrete techniques are `'static`.

## Interface Summary

```
M::play(technique)
  → stores Box<dyn Technique> + target_ids + segment_index in animations vec
  → stores Segment::Animation { name, duration } on timeline (unchanged)

M::animations() -> &[AnimationRecord]
  → read-only access for compute_frame_state()

compute_frame_state(&M, time)
  → builds elements with default visuals (unchanged)
  → calls apply_animations() to overlay technique output
  → returns FrameState with animated visual state
```

## Test Locations

New tests in `moron-core/src/frame.rs::tests`:
- `animation_fade_in_at_start` — FadeIn at progress 0.0 → opacity 0.0
- `animation_fade_in_at_mid` — FadeIn at progress 0.5 → opacity 0.5
- `animation_fade_in_at_end` — FadeIn at progress 1.0 → opacity 1.0
- `animation_before_start` — element with pending animation shows apply(0.0) state
- `animation_after_end` — element with completed animation shows apply(1.0) state
- `no_animation_default_visible` — element without animation retains opacity 1.0
- `animation_fade_up_translation` — FadeUp produces both opacity and translate_y changes
- `animation_not_applied_to_invisible` — cleared element ignores animation output

Existing tests in `facade.rs::tests::play_records_animation_segments` — should continue to pass since timeline behavior is unchanged.
