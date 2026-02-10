# T-009-02 Structure: Per-Item Animation State

## Files Modified

### 1. `moron-techniques/src/technique.rs`

**Add `apply_items()` to `Technique` trait:**
```rust
fn apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput> {
    vec![self.apply(progress); count]
}
```
Default impl: all items get the same output. Object-safe, backwards compatible.

### 2. `moron-techniques/src/staging.rs`

**Override `apply_items()` in `Stagger<T>` impl:**
```rust
fn apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput> {
    (0..count).map(|i| self.apply_item_for_count(i, count, progress)).collect()
}
```
New helper `apply_item_for_count(index, count, progress)` uses the passed count instead of `self.count`, so it adapts to the actual number of items in the element.

### 3. `moron-core/src/frame.rs`

**New struct `ItemState`:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemState {
    pub text: String,
    pub opacity: f64,
    pub translate_x: f64,
    pub translate_y: f64,
    pub scale: f64,
    pub rotation: f64,
}
```

**Change `ElementState.items`:**
From: `pub items: Vec<String>`
To: `pub items: Vec<ItemState>`

**Update `compute_frame_state()`:**
- Build `ItemState` entries from `ElementRecord.items` with default transforms (opacity 1.0, scale 1.0, etc.)

**Update `apply_animations()`:**
- For Steps elements with non-empty items: call `technique.apply_items(items.len(), progress)` and write per-item transforms
- For Steps elements with Stagger: skip element-level transform write, only write per-item
- For non-Steps elements: keep current behavior (element-level transforms)

### 4. `packages/ui/src/types.ts`

**New interface `ItemState`:**
```ts
export interface ItemState {
  text: string;
  opacity: number;
  translateX: number;
  translateY: number;
  scale: number;
  rotation: number;
}
```

**Change `ElementState.items`:**
From: `items: string[]`
To: `items: ItemState[]`

### 5. `packages/ui/src/MoronFrame.tsx`

**Update Steps case in `renderContent()`:**
Each item div gets individual `opacity` and `transform` CSS from its ItemState:
```tsx
<div key={i} style={{
  opacity: item.opacity,
  transform: buildItemTransform(item),
}}>
  {item.text}
</div>
```

Add `buildItemTransform(item: ItemState)` helper, similar to `buildTransform(el)`.

## Files NOT Modified

- `moron-core/src/facade.rs` — No changes needed. `M::play()` continues to box the technique. Count sync handled by `apply_items(count)` parameter.
- `moron-core/src/timeline.rs` — No changes.
- Other technique files — They inherit the default `apply_items()`.

## Module Boundaries

- `moron-techniques` owns the Technique trait and Stagger per-item logic
- `moron-core/frame.rs` owns ItemState and the apply_animations integration
- `packages/ui` owns the React rendering of per-item transforms
- Serde JSON is the contract boundary between Rust and TypeScript

## Test Updates

- Existing tests in frame.rs that check `items: vec!["one".to_string(), ...]` must be updated to use `ItemState { text: "one".into(), ... }`
- New tests: Stagger on Steps at various progress points verifying per-item opacity/transforms
- Existing technique tests in moron-techniques unchanged (they test apply(), not apply_items())
