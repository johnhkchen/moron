# T-009-02 Research: Per-Item Animation State

## Current State

### Rust: `ElementState.items` (frame.rs:54)
```rust
pub items: Vec<String>,
```
Plain text only. Populated from `ElementRecord.items` which is set by `m.steps()`.
Steps elements carry items like `["one", "two", "three"]`. All other element kinds have empty `items`.

### Rust: `compute_frame_state()` / `apply_animations()` (frame.rs:191-233)
T-009-01 added `apply_animations()` which:
1. Iterates `m.animations()` (Vec<AnimationRecord>)
2. Computes progress within the animation segment's time window
3. Calls `record.technique.apply(progress)` to get a single `TechniqueOutput`
4. Applies that output to each target element's top-level visual state

The entire element gets one set of transforms (opacity, translate_x/y, scale, rotation).
When a Stagger targets a Steps element, all items get the same visual state.

### Rust: `AnimationRecord` (facade.rs:66-74)
```rust
pub(crate) struct AnimationRecord {
    pub technique: Box<dyn moron_techniques::Technique>,
    pub target_ids: Vec<u64>,
    pub segment_index: usize,
}
```
The technique is erased behind `Box<dyn Technique>`. No way to access `Stagger.apply_item()` through the trait interface.

### Rust: `Stagger<T>` (staging.rs)
- Wraps an inner technique with staggered delay
- `apply_item(index, progress) -> TechniqueOutput` computes per-item state
- `apply(progress)` delegates to `apply_item(0, progress)` (first item only)
- Total duration = inner.duration() + delay * (count - 1)
- Item progress normalized: each item starts at `delay * index / total_dur`

### Rust: `Technique` trait (technique.rs:108-117)
```rust
pub trait Technique {
    fn name(&self) -> &str;
    fn duration(&self) -> f64;
    fn apply(&self, progress: f64) -> TechniqueOutput;
}
```
No per-item method. Object-safe. Used as `Box<dyn Technique>` in AnimationRecord.

### TypeScript: `ElementState.items` (types.ts:47)
```ts
items: string[];
```
Plain string array.

### React: Steps rendering (MoronFrame.tsx:147-173)
```tsx
case "steps":
  el.items.map((item, i) => (
    <div key={i} data-moron="sequence-item" data-index={i}>
      {item}
    </div>
  ))
```
No per-item transforms. Each item renders with the same element-level opacity/transform.

### Rust: `M::play()` (facade.rs:274-290)
Records the technique + targets the most recently created element. A Stagger played after `m.steps()` would target the Steps element, but the pipeline only calls `apply()` (not `apply_item()`).

## Key Constraints

1. **Trait object erasure**: `Box<dyn Technique>` erases `Stagger<T>`. Can't call `apply_item()` through the trait without extending it.

2. **Object safety**: The Technique trait must remain object-safe for `Box<dyn Technique>` to work. Any new method must have a compatible signature (no `Self` in return, no generics).

3. **Serde contract**: `ElementState` serializes to JSON for React. Changing `items: Vec<String>` to a richer type changes the JSON shape. Both Rust and TypeScript must agree.

4. **Backwards compatibility**: Non-Steps elements have `items: []`. The new type must serialize cleanly for empty lists and for non-stagger animations.

5. **Steps count**: `ElementKind::Steps { count }` and `ElementRecord.items.len()` both know the item count. `Stagger.count` is set independently — must be kept in sync or derived.

## Relevant Files

| File | What changes |
|------|-------------|
| `moron-techniques/src/technique.rs` | Add per-item method to Technique trait |
| `moron-techniques/src/staging.rs` | Override per-item method in Stagger impl |
| `moron-core/src/frame.rs` | New ItemState type, update ElementState, update apply_animations() |
| `moron-core/src/facade.rs` | Possibly sync Stagger count with Steps item count |
| `packages/ui/src/types.ts` | Update items type from string[] to ItemState[] |
| `packages/ui/src/MoronFrame.tsx` | Apply per-item transforms when rendering Steps |

## Observations

- `Stagger.count` defaults to 1 and must be explicitly set via `.with_count()`. When targeting a Steps element, the count should match `items.len()`. This could be set automatically in `M::play()` or left to the scene author.
- The `Technique` trait can be extended with a default method `apply_items(count, progress) -> Vec<TechniqueOutput>` that falls back to calling `apply(progress)` for all items. Stagger overrides this.
- `ItemState` only needs text + the same 5 transform fields from TechniqueOutput (opacity, translate_x, translate_y, scale, rotation). Could reuse TechniqueOutput or define a new struct.
- The element-level transforms (on ElementState) still apply as a group transform on the wrapper div. Per-item transforms compose inside each item div.
