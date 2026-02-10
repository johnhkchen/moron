# T-004-02 Plan: React Frame Component

## Step Sequence

### Step 1: Create types.ts

Write `packages/ui/src/types.ts` with all TypeScript types matching the Rust
serde output from `moron-core/src/frame.rs`.

Types to define:
- `ElementKind` — discriminated union with `type` tag
- `ElementState` — flat interface with visual fields at top level
- `ThemeState` — name + cssProperties
- `FrameState` — top-level frame data

Verification: `npx tsc --noEmit` in packages/ui/ passes with zero errors.

Key details that must be exact:
- `ElementKind` uses `type` as the discriminant field (matching `#[serde(tag = "type")]`)
- `ElementState.kind` field name (not `content` or `type` — Rust field is `kind`)
- `activeNarration: string | null` (not `undefined`, serde serializes None as null)
- `cssProperties` (camelCase of `css_properties`)
- `totalDuration` (camelCase of `total_duration`)
- `translateX`, `translateY` (camelCase of `translate_x`, `translate_y`)

### Step 2: Create MoronFrame.tsx

Write `packages/ui/src/MoronFrame.tsx` with the component implementation.

Sub-tasks:
1. Define `MoronFrameProps` interface
2. Implement `buildTransform()` helper — constructs CSS transform string
3. Implement `renderContent()` helper — dispatches on `kind.type`
4. Implement `MoronFrame` component — container + element wrappers

Content rendering details:

**title**: `<h1 data-moron="title" style={titleStyles}>{el.content}</h1>`
- fontSize: `var(--moron-text-4xl)`
- fontWeight: `var(--moron-font-weight-bold)`
- margin: 0, padding: 0

**section**: `<h2 data-moron="title" style={sectionStyles}>{el.content}</h2>`
- fontSize: `var(--moron-text-2xl)`
- fontWeight: `var(--moron-font-weight-semibold)`
- margin: 0, padding: 0

**show**: `<p data-moron="show" style={showStyles}>{el.content}</p>`
- fontSize: `var(--moron-text-xl)`
- margin: 0, padding: 0

**metric**: Compound structure with value, label, direction
- Parse `el.content` which is `"label: value"` format (from Rust facade)
- `<div data-moron="metric" data-direction={kind.direction}>`
- `<span data-moron="metric-value">{value part}</span>`
- `<span data-moron="metric-label">{label part}</span>`
- Text alignment center, value font large, label font smaller

**steps**: Vertical list from `el.items`
- `<div data-moron="sequence" style={flexColumn}>`
- `{el.items.map((item, i) => <div data-moron="sequence-item" data-index={i}>{item}</div>)}`
- Gap between items via CSS gap property

Verification: `npx tsc --noEmit` passes.

### Step 3: Update index.ts

Add exports for `MoronFrame`, `MoronFrameProps`, and all types from `types.ts`.

Verification: `npx tsc --noEmit` passes. All new symbols are importable from
`@moron/ui`.

### Step 4: Verify Rust side compiles

Run `cargo check` to confirm the Rust codebase still compiles (no accidental
changes to Rust files).

### Step 5: Run full verification

- `cd packages/ui && npx tsc --noEmit` — TypeScript types check
- `cargo check` — Rust workspace compiles

---

## Testing Strategy

### TypeScript Type Checking

Primary verification is `tsc --noEmit`. Since MoronFrame is a pure render
component with no runtime logic beyond DOM construction, the type checker
catches the most likely errors:
- Mismatched field names between types.ts and Rust serde output
- Incorrect discriminated union handling
- Missing CSS property types

### Manual Verification Checklist

Since there is no test runner configured in packages/ui/ yet, verification is
structural:

1. types.ts field names match frame.rs serde output exactly:
   - FrameState: time, frame, totalDuration, fps, elements, activeNarration, theme
   - ElementState: id, kind, content, items, visible, opacity, translateX, translateY, scale, rotation
   - ElementKind: type="title"|"show"|"section"|"metric"|"steps", metric has direction, steps has count
   - ThemeState: name, cssProperties

2. MoronFrame renders correct HTML structure:
   - Root div with data-moron="frame"
   - Theme CSS properties on root div style
   - Element wrapper divs with absolute positioning
   - Content elements with correct data-moron attributes

3. buildTransform produces correct CSS:
   - translate(Xpx, Ypx) when translateX or translateY != 0
   - scale(N) when scale != 1
   - rotate(Ndeg) when rotation != 0
   - "none" when all transforms are identity

4. Element visibility:
   - visible: false elements are not rendered
   - visible: true elements are rendered with their opacity

5. Z-ordering:
   - zIndex = array index (0 = lowest, last = highest)

---

## Risk Mitigation

### Type Drift

The biggest risk is TypeScript types drifting from Rust serde output. Mitigation:
types.ts includes comments referencing the Rust source file and struct names.
The integration test in T-004-01 (`frame_state_json_has_camel_case_keys`) serves
as a cross-check.

### Metric Content Parsing

The Rust facade stores metric content as `"label: value"` (from the
`metric_element_preserves_direction` test: `"Revenue: $1M"`). The React side
must parse this. If the colon-split fails, fall back to rendering the full
content string.

---

## Commit Plan

Since commits are handled externally, the implementation produces these files
in the order listed. Each file is self-contained and can be verified
independently by the type checker.

1. types.ts — standalone, no imports from the project
2. MoronFrame.tsx — imports from types.ts and React
3. index.ts — adds re-exports
4. RDSPI artifacts — structure.md, plan.md, progress.md
5. Ticket update — T-004-02.md frontmatter
