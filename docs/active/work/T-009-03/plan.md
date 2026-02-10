# T-009-03 Plan: Element Layout System

## Step 1: Add `layout_y` to Rust `ElementState`

Add `pub layout_y: f64` field to `ElementState` in frame.rs. Default to `0.5` in all existing construction sites (`compute_frame_state`). Ensure serde serializes as `layoutY`.

**Verify:** `cargo check` passes. Existing tests pass (all elements get 0.5).

## Step 2: Implement `assign_layout_positions()`

Add a private function in frame.rs:

```
fn assign_layout_positions(elements: &mut [ElementState])
```

Logic:
1. Collect indices of visible elements.
2. If 0 or 1 visible → all get 0.5, return.
3. Classify each visible element: `is_header(kind)` returns true for Title/Section.
4. Sort visible elements: headers first, then bodies. Preserve relative order within each group.
5. Assign layout_y based on count:
   - 2 elements: 0.3 and 0.65
   - 3 elements: 0.2, 0.5, 0.8
   - N elements: evenly space from 0.2 to 0.8

Helper: `fn is_header(kind: &ElementKind) -> bool` — matches Title | Section.

**Verify:** Unit tests (next step).

## Step 3: Wire into `compute_frame_state()` and add tests

Call `assign_layout_positions(&mut elements)` in `compute_frame_state()` after building the element vec.

Add unit tests:
- `layout_single_element_centered` — Title alone → layout_y 0.5
- `layout_title_plus_show` — Title at 0.3, Show at 0.65
- `layout_section_plus_steps` — Section at 0.3, Steps at 0.65
- `layout_three_elements` — Section + Show + Steps → 0.2, 0.5, 0.8
- `layout_after_clear` — After clear, new solo element gets 0.5
- `layout_hidden_elements_ignored` — Hidden elements don't affect layout of visible ones
- `layout_two_body_elements` — Show + Steps → 0.35, 0.65

**Verify:** `cargo test` passes, `cargo clippy` clean.

## Step 4: Update TypeScript types

Add `layoutY: number` to `ElementState` in packages/ui/src/types.ts.

**Verify:** TypeScript types match Rust serde output.

## Step 5: Update React MoronFrame

Modify element wrapper in MoronFrame.tsx:
- Remove `inset: 0; display: flex; align-items: center; justify-content: center`
- Add `top: ${el.layoutY * 100}%; left: 50%; transform: translate(-50%, -50%) ${animTransform}`
- Add `maxWidth: "80%"` for text containment
- Combine centering transform with animation transform in `buildTransform` or at the call site

**Verify:** `npm run build` (or equivalent) in packages/ui succeeds.

## Step 6: Verify JSON round-trip and existing tests

Run full test suite. Check that:
- `frame_state_json_has_camel_case_keys` passes (layoutY in output)
- `frame_state_json_round_trip` passes
- `frame_state_serializes_to_json` passes
- All existing frame tests still pass
- WhatIsMoron scene tests pass

**Verify:** `cargo test` all green, `cargo clippy` clean.

## Testing Strategy

- **Unit tests** in frame.rs: Assert exact layout_y values for common compositions
- **Round-trip test**: Verify layoutY serializes/deserializes
- **Integration**: WhatIsMoron scene builds without panic (existing test)
- **Visual**: Manual check that React output shows non-overlapping elements (not automated)
