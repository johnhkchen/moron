# T-009-02 Design: Per-Item Animation State

## Problem

When a Stagger animation targets a Steps element, every item gets the same visual state because:
1. `apply_animations()` calls `technique.apply(progress)` which returns one `TechniqueOutput`
2. That output is applied to the element's top-level opacity/translate/scale/rotation
3. `ElementState.items` is `Vec<String>` — no per-item transform slots
4. React renders each item as a plain div inheriting the element's group transform

## Options Considered

### Option A: Extend Technique trait with `apply_items()`

Add a default method to the `Technique` trait:
```rust
fn apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput> {
    vec![self.apply(progress); count]
}
```
Stagger overrides to call `apply_item(i, progress)` for each index. The method is object-safe (no generics, concrete return type). `apply_animations()` detects Steps elements and calls `apply_items()` instead of `apply()`.

**Pros**: Clean, extensible, no downcasting, any future per-item technique benefits.
**Cons**: Adds a method to the public trait that most techniques don't use.

### Option B: Downcast via `Any`

Make `Technique: Any`, then `downcast_ref::<Stagger<T>>()` in `apply_animations()`.

**Pros**: No trait change.
**Cons**: Can't downcast without knowing `T`. Stagger<FadeIn>, Stagger<WithEase<FadeUp>>, etc. are all different types. Brittle, not extensible.

### Option C: Separate StaggerRecord

Store stagger metadata (count, delay, inner technique) in a separate struct alongside AnimationRecord. `apply_animations()` checks for stagger records.

**Pros**: No trait change.
**Cons**: Duplicates data, couples facade to stagger internals, complex bookkeeping.

## Decision: Option A — Extend Technique trait

The default method is cheap (one extra method with a sensible default). It's forward-compatible: if we add more per-item techniques later, they just override the method. No downcasting needed. The trait stays object-safe.

## Data Model Change

### New type: `ItemState` (in frame.rs)
```rust
pub struct ItemState {
    pub text: String,
    pub opacity: f64,
    pub translate_x: f64,
    pub translate_y: f64,
    pub scale: f64,
    pub rotation: f64,
}
```

`ElementState.items` changes from `Vec<String>` to `Vec<ItemState>`.

Default: each ItemState has `opacity: 1.0, scale: 1.0, translate_x/y: 0.0, rotation: 0.0` — same as element-level defaults. When a Stagger animation is active, `apply_animations()` writes per-item transforms from `apply_items()`.

### JSON contract change
Before: `"items": ["one", "two", "three"]`
After: `"items": [{"text": "one", "opacity": 1.0, ...}, ...]`

TypeScript `items: string[]` becomes `items: ItemState[]`.

### React rendering change
Each Steps item div gets individual `opacity` and `transform` CSS from its ItemState, composing with the element-level wrapper transform.

## Stagger Count Sync

When `M::play()` detects the technique is a Stagger (via `name() == "Stagger"`) and the target is a Steps element, it should auto-set the Stagger's count to match `items.len()`. This avoids requiring scene authors to manually keep counts in sync.

However, since `M::play()` takes the technique by value and stores it as `Box<dyn Technique>`, we can't mutate `Stagger.count` after boxing. Instead: `apply_items()` receives the `count` parameter from the caller — it's the element's item count, not the Stagger's internal count. Stagger's `apply_items()` uses the passed `count` directly, ignoring its own `self.count` for per-item computation. This makes the API self-correcting.

## Element-Level vs Item-Level Transforms

When a Stagger animation is active on a Steps element:
- Element-level transforms (ElementState.opacity, etc.) remain at defaults (1.0, 0, etc.)
- Per-item transforms carry the animation state
- React applies element-level transform on the wrapper, per-item on each child

This means `apply_animations()` should skip writing to the element-level fields for Stagger-on-Steps, and instead only write to `items[i]`.
