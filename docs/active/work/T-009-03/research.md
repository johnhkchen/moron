# T-009-03 Research: Element Layout System

## Current State

### How elements are created and tracked

`M` (facade.rs) mints elements via `mint_element_with_meta()`. Each element gets:
- `id: u64` — unique identifier
- `kind: ElementKind` — Title, Show, Section, Metric{direction}, Steps{count}
- `content: String` — text content
- `items: Vec<String>` — list items (Steps only)
- `created_at: f64` — timeline position at creation
- `ended_at: Option<f64>` — set by `clear()`, None means visible until end

Elements are stored in `M.elements: Vec<ElementRecord>` (crate-internal).

### How slides are bounded

`m.clear()` (facade.rs:239-248) sets `ended_at = now` on all elements without one. This is the slide boundary. A "slide" is the set of elements created between consecutive `clear()` calls.

The WhatIsMoronScene demonstrates typical slide patterns:
1. **Title only**: `title("What is moron?")`
2. **Section + Show**: `section("The Problem")` + `show("Complex tools...")`
3. **Section + Steps**: `section("A Better Way")` + `steps([...])`
4. **Section + Metric**: `section("Lean and Mean")` + `metric(...)`
5. **Title + Show**: `title("moron")` + `show("Offline. Fast. Professional.")`

### How compute_frame_state() works (frame.rs:121-176)

Iterates `m.elements()`, checks visibility (`created_at <= time && ended_at not crossed`), and produces `ElementState` with hardcoded visuals:
- `opacity: 1.0` if visible, `0.0` if not
- `translate_x: 0.0, translate_y: 0.0`
- `scale: 1.0` if visible, `0.0` if not
- `rotation: 0.0`

No layout positioning. No awareness of co-visible elements.

### How React renders elements (MoronFrame.tsx:221-231)

Every visible element gets an absolutely-positioned wrapper:
```css
position: absolute;
inset: 0;          /* fills entire 1920x1080 frame */
display: flex;
align-items: center;
justify-content: center;
```

Result: every element occupies the full frame and is dead-centered. Multiple visible elements stack directly on top of each other.

### ElementState fields (frame.rs:46-67, types.ts:39-60)

Current fields: `id, kind, content, items, visible, opacity, translate_x, translate_y, scale, rotation`. No layout/position fields. The Rust struct and TS interface are in sync.

### Content rendering by kind (MoronFrame.tsx:50-173)

- **Title**: `<h1>` with `text-4xl`, centered
- **Section**: `<h2>` with `text-2xl`, centered
- **Show**: `<p>` with `text-xl`, centered
- **Metric**: flex-column with large value + label below
- **Steps**: flex-column, left-aligned items with `space-4` gap

All content renderers assume they're centered in their wrapper. They set text alignment/flex alignment but not position.

## Slide Composition Patterns

Based on the WhatIsMoron scene and the acceptance criteria, common compositions:

| Pattern | Elements | Expected Layout |
|---------|----------|-----------------|
| Solo title | Title | Centered vertically |
| Solo section | Section | Centered vertically |
| Title + subtitle | Title + Show | Title upper, Show center-lower |
| Section + content | Section + Show | Section upper, Show center |
| Section + steps | Section + Steps | Section upper, Steps center |
| Section + metric | Section + Metric | Section upper, Metric center |
| Three elements | Section + Show + Steps | Distribute vertically |

## Key Constraints

1. **Rust computes layout**: FrameState JSON must contain position data. React should not compute layout logic — it just places elements where Rust says.
2. **Convention over configuration**: Scene authors don't specify positions. Layout is inferred from element kinds and co-visibility.
3. **Element kind semantics**: Title/Section are "header" types (favor top). Show/Steps/Metric are "body" types (favor center/below).
4. **translateY already exists**: ElementState already has `translate_y` — layout could use this field, OR a new dedicated layout field could be added. Using translate_y would conflict with animation techniques that also use translate_y (FadeUp translates Y).
5. **Animation interaction**: T-009-01 will wire technique output into element state. Layout offsets and animation offsets must compose, not conflict. If layout uses `translate_y`, animation FadeUp's translate_y would be relative to... what? This is the critical design question.

## The translateY Conflict

FadeUp applies `translate_y: 40.0 * (1.0 - progress)` — starts 40px below and moves to 0. If layout sets `translate_y: -200` to position a title in the upper third, and FadeUp sets `translate_y: 40`, the two values would fight.

Possible resolutions:
- **Separate fields**: Add `layout_offset_y` (or `slot`/`region`) to ElementState, distinct from `translate_y`. React combines them.
- **Additive composition**: Rust computes `base_translate_y` (from layout) + `anim_translate_y` (from technique) and writes the sum to `translate_y`.
- **CSS composition**: Use different CSS properties — layout uses `top`/`bottom`/flexbox, animation uses `transform: translate()`.

## Integration with T-009-01/T-009-02

T-009-03 is independent (no dependency). But T-009-04 (showcase) depends on both. The layout system and animation system must compose cleanly. The design must anticipate how layout + animation combine even though T-009-01 hasn't shipped yet.

## Summary of What Needs to Change

1. **frame.rs**: `ElementState` needs layout positioning data. `compute_frame_state()` needs to group visible elements by slide and assign positions.
2. **types.ts**: Mirror new fields.
3. **MoronFrame.tsx**: Use layout data for positioning instead of `inset: 0; align-items: center`.
4. **Tests**: Verify layout for common compositions.
