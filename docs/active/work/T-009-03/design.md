# T-009-03 Design: Element Layout System

## Problem

Multiple visible elements stack dead-center on top of each other. Need automatic layout positioning based on element kinds and co-visibility.

## Approach A: Slot-based layout with `layout_offset_y`

Add a `layout_offset_y: f64` field to `ElementState`. Rust computes vertical offset in pixels based on element kind and how many elements are co-visible. React applies this offset separately from `translate_y` (animation).

**Layout algorithm:**
1. Collect visible elements at current time.
2. Classify each: "header" (Title, Section) or "body" (Show, Steps, Metric).
3. If 1 element → offset 0 (centered).
4. If 2 elements (header + body) → header at -20% of frame height, body at +10%.
5. If 3+ elements → distribute evenly across vertical space.

**React changes:** Replace `inset: 0; align-items: center` with `top: calc(50% + layout_offset_y)` or similar.

**Pros:** Simple. Pixel offsets are deterministic. Easy to test.
**Cons:** Hardcoded pixel math for 1920x1080. Doesn't adapt to content size. New field on the wire format that animation must ignore.

## Approach B: Region/slot enum with flexbox

Add a `layout_region: "top" | "center" | "bottom"` enum to `ElementState`. React uses this to place elements in CSS grid or flex regions.

**Layout algorithm:**
1. Collect visible elements, classify as header/body.
2. Assign regions: headers → "top", single body → "center", multiple bodies → "center" then "bottom".

**React changes:** Replace absolute positioning with a 3-row CSS grid. Each element goes into its assigned row.

**Pros:** Responsive to content. Clean separation. No pixel math.
**Cons:** More React complexity. Layout logic split between Rust (assignment) and React (grid sizing). Harder to test from Rust side since visual result depends on CSS.

## Approach C: Percentage-based `layout_y` with CSS composition

Add `layout_y: f64` to `ElementState` — a percentage (0.0 = top, 0.5 = center, 1.0 = bottom). React uses `top: {layout_y * 100}%` with a centering transform. Animation `translate_y` composes via CSS `transform`.

**Layout algorithm:**
1. Collect visible elements, classify as header/body.
2. Single element → `layout_y: 0.5`.
3. Two elements → header at 0.3, body at 0.65.
4. Three elements → distribute: 0.2, 0.5, 0.8.

**React changes:** Element wrapper uses `position: absolute; top: {layout_y * 100}%; left: 50%; transform: translate(-50%, -50%) {animation_transforms}`.

**Pros:** Resolution-independent. Clean composition with animation transforms. Fully testable from Rust (just check numeric values). No CSS grid complexity. Animation translate_y composes naturally via CSS transform stack.
**Cons:** Percentage-based might not give pixel-perfect control for edge cases.

## Decision: Approach C — Percentage-based `layout_y`

**Rationale:**

1. **Clean composition with animations.** CSS `top` positions the element, `transform: translate()` animates it. These compose naturally in CSS. FadeUp's `translate_y: 40px` moves the element 40px from its layout position — exactly correct behavior.

2. **Resolution-independent.** `layout_y: 0.3` means "30% from top" regardless of whether the frame is 1080p or 4K. No hardcoded pixel values.

3. **Fully testable from Rust.** Tests just assert `element.layout_y == 0.3`. No need to reason about CSS grid rows or pixel arithmetic.

4. **Minimal React changes.** Replace `inset: 0; display: flex; align-items: center; justify-content: center` with `top: {y}%; left: 50%; transform: translate(-50%, -50%)`. The animation transforms already compose via `buildTransform()`.

5. **Forward-compatible.** Can later add `layout_x` for horizontal positioning without changing the architecture.

### Layout Rules

Element kind classification:
- **Header**: Title, Section
- **Body**: Show, Steps, Metric

Position assignment by composition:

| Visible Count | Composition | Positions (layout_y) |
|---------------|-------------|---------------------|
| 1 | Any solo | 0.5 |
| 2 | Header + Body | Header: 0.3, Body: 0.65 |
| 2 | Body + Body | 0.35, 0.65 |
| 2 | Header + Header | 0.35, 0.65 |
| 3 | Header + 2 Body | Header: 0.2, Body1: 0.5, Body2: 0.8 |
| 3+ | General | Evenly spaced from 0.2 to 0.8 |

Headers always sort before bodies. Within the same class, preserve creation order.

### Wire format change

```rust
pub struct ElementState {
    // ... existing fields ...
    pub layout_y: f64,  // 0.0=top, 0.5=center, 1.0=bottom
}
```

TypeScript mirror:
```ts
interface ElementState {
    // ... existing fields ...
    layoutY: number;
}
```

### React positioning change

From:
```css
position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
```

To:
```css
position: absolute; top: {layout_y * 100}%; left: 50%; transform: translate(-50%, -50%) {animation_transforms};
```

The `buildTransform()` function already builds a transform string from translate_x/y, scale, rotation. We prepend `translate(-50%, -50%)` for centering, then append the animation transforms.

## Rejected Alternatives

- **Approach A (pixel offsets)**: Hardcoded to resolution. Awkward interaction with animation translate_y (same coordinate space, need additive composition in Rust).
- **Approach B (region enum)**: Splits layout logic between Rust and React. Harder to test. Over-engineered for v1.
