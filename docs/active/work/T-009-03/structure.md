# T-009-03 Structure: Element Layout System

## Files Modified

### 1. `moron-core/src/frame.rs`

**ElementState struct** — Add field:
```rust
pub layout_y: f64,  // 0.0=top, 0.5=center, 1.0=bottom
```

**New function: `assign_layout_positions()`**
- Input: `&[ElementState]` (after visibility is computed)
- Logic: filter visible elements, classify by kind (header/body), assign `layout_y` values
- Called from `compute_frame_state()` after building the element vec

**compute_frame_state()** — After building elements, call `assign_layout_positions()` to set `layout_y` on each visible element. Hidden elements get `layout_y: 0.5` (default, doesn't matter since they're not rendered).

**Tests** — New test functions:
- `single_element_centered()` — one Title, layout_y == 0.5
- `title_plus_steps_layout()` — Title at 0.3, Steps at 0.65
- `section_plus_show_layout()` — Section at 0.3, Show at 0.65
- `three_element_layout()` — distribute across 0.2, 0.5, 0.8
- `solo_after_clear_recenters()` — after clear + new element, layout resets

### 2. `packages/ui/src/types.ts`

**ElementState interface** — Add field:
```ts
layoutY: number;
```

### 3. `packages/ui/src/MoronFrame.tsx`

**Element wrapper style** — Replace:
```ts
position: "absolute",
inset: 0,
display: "flex",
alignItems: "center",
justifyContent: "center",
```

With:
```ts
position: "absolute",
top: `${el.layoutY * 100}%`,
left: "50%",
transform: `translate(-50%, -50%) ${buildTransform(el)}`,
```

**buildTransform()** — No changes needed. Its output (translate, scale, rotate) appends after the centering transform.

Note: The wrapper no longer uses flexbox for centering. The `translate(-50%, -50%)` handles centering relative to the anchor point. Content renderers don't change since they use text-align/flex internally.

We need to add a `width` constraint so text content doesn't overflow. Add `maxWidth: "80%"` or similar, and `textAlign: "center"` on the wrapper to keep content centered horizontally.

## Files NOT Modified

- `moron-core/src/facade.rs` — No changes. Layout is computed in frame.rs, not stored on ElementRecord.
- `moron-techniques/` — No changes. Animation transforms compose via CSS with layout positioning.
- `moron-voice/` — No changes.
- `moron-cli/` — No changes.

## Module Boundaries

Layout computation is entirely within `frame.rs`. It's a post-processing step on the already-built `Vec<ElementState>`. No new modules, no new crates, no new public APIs on `M`.

The layout function is a pure function: `fn assign_layout_positions(elements: &mut [ElementState])`. It reads `visible` and `kind` fields, writes `layout_y`. Testable in isolation.

## Ordering of Changes

1. Add `layout_y` to Rust `ElementState` (+ serde) and set default `0.5`
2. Add layout computation function
3. Wire into `compute_frame_state()`
4. Add Rust unit tests
5. Update TypeScript types
6. Update React component
7. Verify build
