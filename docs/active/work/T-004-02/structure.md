# T-004-02 Structure: React Frame Component

## Overview

Three files are created or modified. The TypeScript types match the actual Rust
serde serialization from `moron-core/src/frame.rs`, not the preliminary shapes
sketched in the design phase. The design assumed nested `content` and `visual`
sub-objects; the actual Rust struct uses flat fields with `#[serde(rename_all = "camelCase")]`.

---

## File: packages/ui/src/types.ts (NEW)

Purpose: TypeScript types mirroring the Rust FrameState JSON contract.

### Type Inventory

```
ElementKind          — discriminated union matching Rust's #[serde(tag = "type")]
ElementState         — per-element snapshot, flat visual fields
ThemeState           — theme name + CSS custom properties map
FrameState           — top-level container for one frame's data
```

### ElementKind (discriminated union)

Mirrors `frame::ElementKind` with `tag = "type"`:

```typescript
type ElementKind =
  | { type: "title" }
  | { type: "show" }
  | { type: "section" }
  | { type: "metric"; direction: string }
  | { type: "steps"; count: number };
```

Rust uses `#[serde(tag = "type")]` on the enum, so the discriminant field is
literally `"type"`. Variants without data serialize as `{ "type": "title" }`.
Variants with fields include them at the same level: `{ "type": "metric", "direction": "up" }`.

### ElementState (interface)

Mirrors `frame::ElementState` with `rename_all = "camelCase"`:

```
id:         number     — u64 in Rust, safe as JS number for our ID range
kind:       ElementKind
content:    string
items:      string[]
visible:    boolean
opacity:    number     — 0.0..1.0
translateX: number     — pixels
translateY: number     — pixels
scale:      number     — 1.0 = normal
rotation:   number     — degrees
```

Key difference from design: visual fields are NOT nested under a `visual`
sub-object. They are flat on ElementState, matching the Rust struct.

### ThemeState (interface)

Mirrors `frame::ThemeState`:

```
name:           string
cssProperties:  Record<string, string>    — from Rust HashMap<String, String>
```

Note: Rust field `css_properties` becomes `cssProperties` via camelCase rename.

### FrameState (interface)

Mirrors `frame::FrameState`:

```
time:            number
frame:           number        — u32 in Rust
totalDuration:   number
fps:             number        — u32 in Rust
elements:        ElementState[]
activeNarration: string | null — Option<String> in Rust
theme:           ThemeState
```

Note: `active_narration` becomes `activeNarration`, `total_duration` becomes
`totalDuration`. Rust `Option<String>` serializes as `null` when `None`.

---

## File: packages/ui/src/MoronFrame.tsx (NEW)

Purpose: Pure React component that renders a single frame from FrameState.

### Exports

```
MoronFrame       — the component (named export)
MoronFrameProps  — prop type (type export)
```

### Props Interface

```typescript
interface MoronFrameProps {
  state: FrameState;
  width?: number;   // default 1920
  height?: number;  // default 1080
}
```

### Internal Structure

```
MoronFrame (function component)
├── buildTransform(element: ElementState): string
│   Computes CSS transform string from translateX/Y, scale, rotation.
│
├── renderContent(element: ElementState): ReactNode
│   Dispatches on element.kind.type to render the correct DOM subtree.
│   ├── "title"   → <h1 data-moron="title">
│   ├── "show"    → <p data-moron="show">
│   ├── "section" → <h2 data-moron="title">
│   ├── "metric"  → <div data-moron="metric"> with value/label sub-elements
│   └── "steps"   → <div data-moron="sequence"> with <div data-moron="sequence-item"> children
│
└── Render tree:
    <div data-moron="frame" style={container styles + theme CSS properties}>
      {elements.map((el, index) =>
        <div data-moron="element" data-element-id={el.id}
             style={absolute positioning + transform + opacity + z-index}>
          {renderContent(el)}
        </div>
      )}
    </div>
```

### Container Styles

The root `<div>` receives:
- Explicit `width` and `height` in pixels (from props, default 1920x1080)
- `position: relative`, `overflow: hidden`
- All theme CSS custom properties spread as inline style
- `background: var(--moron-bg-primary)`
- `color: var(--moron-fg-primary)`
- `fontFamily: var(--moron-font-sans)`

### Element Wrapper Styles

Each element `<div>` receives:
- `position: absolute`, `inset: 0` (fills the frame)
- `display: flex`, `alignItems: center`, `justifyContent: center`
- `opacity` from element state
- `transform` from buildTransform()
- `zIndex` from array index
- `pointerEvents: none` (non-interactive snapshots)

### Content Renderers

Each content type applies theme-based typography via CSS variables:

- **title**: `fontSize: var(--moron-text-4xl)`, `fontWeight: var(--moron-font-weight-bold)`
- **section**: `fontSize: var(--moron-text-2xl)`, `fontWeight: var(--moron-font-weight-semibold)`
- **show**: `fontSize: var(--moron-text-xl)`
- **metric**: direction indicator via `data-direction` attribute, value and label as sub-elements
- **steps**: vertical flex list, each item a `data-moron="sequence-item"` with index attribute

### Visibility Handling

Elements with `visible: false` are not rendered (early return `null` in the map).
This matches Rust's intent: invisible elements have `opacity: 0` and `scale: 0`
but we skip rendering entirely for cleanliness.

---

## File: packages/ui/src/index.ts (MODIFY)

Add three export blocks after the existing exports:

```typescript
// Frame rendering
export { MoronFrame } from "./MoronFrame";
export type { MoronFrameProps } from "./MoronFrame";

// Frame state types (Rust FrameState JSON contract)
export type {
  FrameState,
  ElementState,
  ElementKind,
  ThemeState,
} from "./types";
```

Existing exports remain untouched.

---

## Module Boundaries

- `types.ts` is the single source of truth for the Rust-React JSON contract.
  MoronFrame.tsx imports from it. External consumers import from index.ts.
- `MoronFrame.tsx` does NOT import from `./components/`. It renders its own DOM
  that follows the same `data-moron` attribute conventions.
- The existing components (Container, Title, Sequence, Metric) are unmodified.
  They serve the template system (future Q4 2026 work).

---

## Naming Conventions

- File: `MoronFrame.tsx` — PascalCase component file, matches existing pattern
- Types file: `types.ts` — lowercase, generic module name for shared types
- data attributes: `data-moron="frame"`, `data-moron="element"` — new attributes
  that extend the existing `data-moron` convention
- Props type: `MoronFrameProps` — matches `ContainerProps`, `TitleProps` pattern
