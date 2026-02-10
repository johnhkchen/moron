# T-004-02 Design: React Frame Component

## Approach: MoronFrame as a FrameState-to-DOM Renderer

MoronFrame receives a FrameState JSON object as props and renders a static DOM tree with inline CSS transforms. No animation, no state management, no side effects. Pure render function.

## TypeScript Types (mirroring Rust FrameState)

### ElementVisualState

Maps to TechniqueOutput from the Rust side:

```typescript
interface ElementVisualState {
  opacity: number;       // 0.0 to 1.0
  translateX: number;    // pixels
  translateY: number;    // pixels
  scale: number;         // 1.0 = normal
  rotation: number;      // degrees
}
```

### ElementContent (discriminated union)

```typescript
type ElementContent =
  | { type: "title"; text: string }
  | { type: "section"; text: string }
  | { type: "show"; text: string }
  | { type: "metric"; label: string; value: string; unit?: string; direction: "up" | "down" | "neutral" }
  | { type: "steps"; items: string[] };
```

### FrameElement

```typescript
interface FrameElement {
  id: number;
  content: ElementContent;
  visual: ElementVisualState;
}
```

### FrameState (top-level props)

```typescript
interface FrameState {
  elements: FrameElement[];
  theme: Record<string, string>;  // {"--moron-bg-primary": "#0f172a", ...}
  time: number;                    // current time in seconds
  duration: number;                // total timeline duration
  frame: number;                   // frame number
}
```

The `theme` field is a flat key-value map of CSS custom properties, matching the output of `Theme::to_css_properties()`. This avoids duplicating the structured Theme type in TypeScript -- the React side only needs the CSS properties.

## Component Design

### MoronFrame Props

```typescript
interface MoronFrameProps {
  state: FrameState;
  width?: number;    // default 1920
  height?: number;   // default 1080
}
```

Width/height default to 1080p. These set the container's explicit dimensions so Chromium captures at the correct resolution.

### HTML Structure

```html
<div data-moron="frame"
     style="width: 1920px; height: 1080px; position: relative;
            overflow: hidden; --moron-bg-primary: ...; --moron-fg-primary: ...;
            background: var(--moron-bg-primary); color: var(--moron-fg-primary);
            font-family: var(--moron-font-sans);">

  <!-- Element 0 (lowest z-index) -->
  <div data-moron="element" data-element-id="0"
       style="position: absolute; inset: 0;
              display: flex; align-items: center; justify-content: center;
              opacity: 0.8;
              transform: translateX(10px) translateY(20px) scale(1.0) rotate(5deg);
              z-index: 0;">
    <!-- content rendered by type -->
  </div>

  <!-- Element 1 (higher z-index) -->
  <div data-moron="element" data-element-id="1"
       style="position: absolute; inset: 0; ...;
              z-index: 1;">
    ...
  </div>
</div>
```

### Key Design Decisions

**1. Theme properties as inline style on the root container.**

The theme `Record<string, string>` is spread onto the container's `style` as CSS custom properties. This means all children can use `var(--moron-*)` without a separate `<style>` block. This is simpler than injecting a `<style>` element and works in both SSR and CSR contexts.

**2. Each element gets a full-viewport positioned wrapper.**

Elements use `position: absolute; inset: 0` so they fill the entire frame. Content is centered within this wrapper using flexbox. CSS transforms are applied to this wrapper. This matches how motion graphics work: elements occupy the full canvas and are positioned/transformed within it. The z-index follows array order (element index = z-index).

**3. Transform string built from visual state.**

The five TechniqueOutput properties map to a single CSS `transform` string:

```typescript
function buildTransform(visual: ElementVisualState): string {
  const parts: string[] = [];
  if (visual.translateX !== 0 || visual.translateY !== 0) {
    parts.push(`translate(${visual.translateX}px, ${visual.translateY}px)`);
  }
  if (visual.scale !== 1) {
    parts.push(`scale(${visual.scale})`);
  }
  if (visual.rotation !== 0) {
    parts.push(`rotate(${visual.rotation}deg)`);
  }
  return parts.join(" ") || "none";
}
```

Opacity is set separately via the CSS `opacity` property, not inside `transform`.

**4. Content rendering by element type.**

Each ElementContent type maps to a rendering strategy:

- **title**: `<h1 data-moron="title">` with `--moron-text-4xl` size, `--moron-font-weight-bold`
- **section**: `<h2 data-moron="title">` with `--moron-text-2xl` size, `--moron-font-weight-semibold`
- **show**: `<p data-moron="show">` with `--moron-text-xl` size
- **metric**: Uses the existing Metric component pattern -- value displayed prominently, label below, direction as accent color
- **steps**: Uses the existing Sequence component pattern -- vertical list with `data-moron="sequence"` and `data-moron="sequence-item"` on each child

Content renderers use `var(--moron-*)` for all styling. No hardcoded colors or sizes.

**5. No Tailwind classes in MoronFrame itself.**

MoronFrame uses inline styles exclusively. Rationale: the component runs inside headless Chromium where Tailwind may or may not be set up. Inline styles are guaranteed to work. The existing base components (Container, Title, etc.) also use inline styles for their core layout.

Templates (future, Q4 2026) will use Tailwind classes. MoronFrame is infrastructure, not a template.

## Rejected Alternatives

### A. Reuse existing Container/Title/Metric components directly

Considered composing MoronFrame from the existing four components. Rejected because:
- The existing components are designed for manual composition in templates, not for data-driven rendering from FrameState JSON.
- They accept `children` and `className` props, not typed content objects.
- MoronFrame needs to map FrameState elements generically; hardwiring to existing components creates coupling.
- Instead, MoronFrame renders elements that follow the same `data-moron` attribute conventions. This preserves the convention contract without importing the components.

### B. Inject theme via <style> block in <head>

Considered using `document.head.appendChild(styleEl)` to inject `:root { --moron-*: ... }`. Rejected because:
- Requires DOM manipulation outside the React tree (side effect).
- Breaks pure rendering -- same input would not always produce the same DOM tree.
- Inline CSS custom properties on the container achieve the same cascade effect and are simpler.

### C. Use CSS class-based transforms instead of inline style

Considered generating CSS classes for transform states. Rejected because:
- Every frame has different transform values. Generating CSS classes per frame is wasteful.
- Inline styles are the natural choice for computed, per-frame values.
- CSS classes make sense for static designs, not per-frame animation snapshots.

### D. Use a React context for theme propagation

Considered wrapping MoronFrame in a ThemeProvider context. Rejected because:
- CSS custom properties already cascade through the DOM natively.
- A React context adds complexity with no benefit when CSS variables do the job.
- The theme data is flat key-value pairs, not structured React state.

## File Plan

- `packages/ui/src/MoronFrame.tsx` -- new file, the component
- `packages/ui/src/types.ts` -- new file, TypeScript types for FrameState (shared between MoronFrame and any future consumers)
- `packages/ui/src/index.ts` -- add exports for MoronFrame and types

## Boundary with T-004-01

MoronFrame defines the TypeScript side of the FrameState contract. T-004-01 defines the Rust side. The JSON serialization format is the shared boundary. The TypeScript types in `types.ts` must match the serde-serialized shape of the Rust `FrameState` struct. If T-004-01 changes the Rust struct shape, `types.ts` must be updated to match.

For implementation, the TypeScript types can be written first (from this design) and the Rust types in T-004-01 written to match, or vice versa. The contract is:
- `elements` is an array of objects with `id`, `content` (discriminated by `type`), and `visual` (five numeric fields).
- `theme` is a flat object mapping `--moron-*` CSS property names to string values.
- `time`, `duration`, `frame` are numeric metadata.
