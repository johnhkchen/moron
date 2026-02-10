# T-009-02 Progress: Per-Item Animation State

## Completed

### Step 1: Extend Technique trait
- Added `apply_items(&self, count, progress) -> Vec<TechniqueOutput>` with default impl
- Updated `WithEase<T>` to delegate `apply_items()` through easing

### Step 2: Override apply_items() in Stagger
- Added `apply_item_for_count(index, count, progress)` helper using passed count
- Added `duration_for_count(count)` helper
- Refactored `apply_item()` to delegate to `apply_item_for_count()`
- Overrode `apply_items()` in `Technique for Stagger<T>`

### Step 3: Add ItemState and update ElementState
- Defined `ItemState` struct with text + 5 transform fields (serde camelCase)
- Changed `ElementState.items` from `Vec<String>` to `Vec<ItemState>`
- Updated `compute_frame_state()` to build `ItemState` entries with default transforms
- Fixed existing test `steps_element_preserves_items` to use new type

### Step 4: Wire apply_items() into apply_animations()
- Steps elements (items.len() > 0): calls `technique.apply_items()` for per-item transforms
- Non-Steps elements: keeps element-level transform behavior
- Element-level transforms stay at defaults when per-item transforms are active
- Added 7 new tests covering per-item animation behavior

### Step 5: Update TypeScript types
- Added `ItemState` interface matching Rust serde output
- Changed `ElementState.items` from `string[]` to `ItemState[]`

### Step 6: Update React rendering
- Added `buildItemTransform()` helper
- Updated Steps case to use `item.text` for content, `item.opacity` + `buildItemTransform()` for styles

### Step 7: Final verification
- 133 tests pass (was 126, +7 new)
- clippy clean
- Added `ItemState` to crate root and prelude exports

## Deviations from Plan
None.
