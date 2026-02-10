# T-004-02 Progress: React Frame Component

## Status: Complete

All five RDSPI phases are done.

---

## Completed Steps

### Step 1: Create types.ts
- Created `packages/ui/src/types.ts`
- Defined `ElementKind`, `ElementState`, `ThemeState`, `FrameState`
- All field names match Rust serde camelCase output exactly
- `ElementKind` uses `type` as discriminant (matching `#[serde(tag = "type")]`)
- `activeNarration` is `string | null` (matching Rust `Option<String>`)
- `cssProperties` matches Rust `css_properties` via camelCase rename
- Verification: `tsc --noEmit` passes

### Step 2: Create MoronFrame.tsx
- Created `packages/ui/src/MoronFrame.tsx`
- Exports: `MoronFrame` component and `MoronFrameProps` type
- `buildTransform()`: constructs CSS transform string from translateX/Y, scale, rotation
- `renderContent()`: dispatches on `kind.type` to render five element types
  - `title` -> `<h1 data-moron="title">` with 4xl font
  - `section` -> `<h2 data-moron="title">` with 2xl font
  - `show` -> `<p data-moron="show">` with xl font
  - `metric` -> `<div data-moron="metric">` with value/label, parses "label: value" format
  - `steps` -> `<div data-moron="sequence">` with sequence-item children
- Container: explicit width/height (default 1920x1080), relative positioning,
  theme CSS custom properties spread as inline style
- Element wrappers: absolute inset-0, flexbox centered, opacity/transform/z-index
- Invisible elements (visible: false) are skipped entirely
- All styles are inline (no Tailwind)
- Pure component: same input = same output
- Verification: `tsc --noEmit` passes

### Step 3: Update index.ts
- Added exports for `MoronFrame`, `MoronFrameProps`
- Added type exports for `FrameState`, `ElementState`, `ElementKind`, `ThemeState`
- Existing exports untouched
- Verification: `tsc --noEmit` passes

### Step 4: Verify Rust side
- `cargo check` passes with no errors

### Step 5: Full verification
- `cd packages/ui && npx tsc --noEmit` — passes, zero errors
- `cargo check` — passes, zero errors

---

## Deviations from Design

The design.md assumed a nested structure with `content` as a discriminated union
and `visual` as a sub-object. The actual Rust `FrameState` (implemented in
T-004-01) uses flat fields on `ElementState`:
- `kind` (ElementKind tagged union) instead of nested `content`
- Visual fields (`opacity`, `translateX`, etc.) are flat on ElementState
- `visible` field exists on ElementState
- `items` field for steps content
- `ThemeState` has `name` + `cssProperties` (not just a flat Record)
- `FrameState` has `totalDuration` and `fps` (not `duration`)
- `activeNarration` field (Option<String> -> string | null)

All TypeScript types were written to match the actual Rust serde output, not
the design sketches. This is correct behavior per RDSPI: research and design
are exploratory, implementation follows the actual codebase state.

---

## Files Changed

| File | Action | Lines |
|------|--------|-------|
| packages/ui/src/types.ts | NEW | 98 |
| packages/ui/src/MoronFrame.tsx | NEW | 199 |
| packages/ui/src/index.ts | MODIFIED | +10 lines |
| docs/active/work/T-004-02/structure.md | NEW | ~150 |
| docs/active/work/T-004-02/plan.md | NEW | ~150 |
| docs/active/work/T-004-02/progress.md | NEW | this file |
| docs/active/tickets/T-004-02.md | MODIFIED | frontmatter update |
