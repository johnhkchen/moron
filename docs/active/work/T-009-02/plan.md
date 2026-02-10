# T-009-02 Plan: Per-Item Animation State

## Step 1: Extend Technique trait with `apply_items()`

**File:** `moron-techniques/src/technique.rs`

- Add `apply_items(&self, count: usize, progress: f64) -> Vec<TechniqueOutput>` to the `Technique` trait with default implementation
- Default: `(0..count).map(|_| self.apply(progress)).collect()`

**Verify:** `cargo check` passes.

## Step 2: Override `apply_items()` in Stagger

**File:** `moron-techniques/src/staging.rs`

- Add `apply_item_for_count(index, count, progress)` method that works like `apply_item()` but uses the passed `count` parameter instead of `self.count`
- Override `apply_items()` in `Technique for Stagger<T>` to call `apply_item_for_count()` for each index
- Also override in `Technique for WithEase<T>` to delegate to inner

**Verify:** `cargo test -p moron-techniques` passes. Add a test for `apply_items()` on a Stagger.

## Step 3: Add `ItemState` and update `ElementState`

**File:** `moron-core/src/frame.rs`

- Define `ItemState` struct with `text`, `opacity`, `translate_x`, `translate_y`, `scale`, `rotation` (serde camelCase)
- Change `ElementState.items` from `Vec<String>` to `Vec<ItemState>`
- Update `compute_frame_state()` to build `ItemState` entries from `ElementRecord.items` with default transforms
- Fix all existing tests that reference `.items` (they check `vec!["one".to_string(), ...]` → now check `ItemState { text: ... }`)

**Verify:** `cargo test -p moron-core` passes (existing tests updated).

## Step 4: Wire `apply_items()` into `apply_animations()`

**File:** `moron-core/src/frame.rs`

- In `apply_animations()`, after computing progress, check if the target element has non-empty items (is a Steps element)
- If yes: call `technique.apply_items(items.len(), progress)` and write per-item transforms to each `ItemState`
- If yes: keep element-level transforms at defaults (opacity 1.0, scale 1.0) so the wrapper div doesn't double-apply
- If no: keep current behavior (write element-level transforms)

**Verify:** `cargo test -p moron-core` passes. Add new tests:
- Stagger + Steps: per-item opacity varies at midpoint
- Stagger + Steps: first item animated, last item still at initial state
- Non-Stagger + Steps: all items get same transforms
- `cargo clippy` clean

## Step 5: Update TypeScript types

**File:** `packages/ui/src/types.ts`

- Add `ItemState` interface with `text`, `opacity`, `translateX`, `translateY`, `scale`, `rotation`
- Change `ElementState.items` from `string[]` to `ItemState[]`

**Verify:** TypeScript builds (or manual review; no TS build step configured).

## Step 6: Update React rendering

**File:** `packages/ui/src/MoronFrame.tsx`

- Add `buildItemTransform(item: ItemState)` helper
- Update Steps case in `renderContent()` to use `item.text` for content, `item.opacity` and `buildItemTransform()` for per-item styles

**Verify:** Manual review. `cargo test` still passes (React changes are rendering-only).

## Step 7: Final verification

- `cargo test` — all tests pass
- `cargo clippy` — clean
- Verify JSON round-trip test still passes with new ItemState shape
- Update ticket phase to `done`
