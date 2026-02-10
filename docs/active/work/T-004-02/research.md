# T-004-02 Research: React Frame Component

## Objective

Create a `<MoronFrame>` React component in `packages/ui/` that renders a single frame from serialized FrameState JSON. This is the React-side counterpart to the Rust FrameState struct defined in T-004-01.

## Current packages/ui/ Structure

### Package Configuration

`packages/ui/package.json` defines `@moron/ui` v0.1.0 as an ESM package (`"type": "module"`). Entry point is `src/index.ts`. Dependencies are React 19.2.x and React DOM 19.2.x. TypeScript 5.8.x with strict mode, JSX set to `react-jsx`, target ES2022, module resolution `bundler`.

### Existing Components

Four base components exist in `packages/ui/src/components/`:

| Component | File | data-moron attr | Purpose |
|-----------|------|-----------------|---------|
| Container | Container.tsx | `data-moron="container"` | Root layout wrapper, full viewport, `position: relative`, `overflow: hidden` |
| Title | Title.tsx | `data-moron="title"` | Heading text, supports level 1/2/3, renders as h1/h2/h3 |
| Sequence | Sequence.tsx | `data-moron="sequence"`, `data-moron="sequence-item"` | Flex container with individually animatable children |
| Metric | Metric.tsx | `data-moron="metric"`, `data-moron="metric-value"`, `data-moron="metric-label"` | KPI display with value, label, unit |

All components accept `className` and `style` props. None accept animation state directly -- they are static building blocks that rely on external CSS/style injection for animation.

### Index Exports

`src/index.ts` re-exports all four components and their prop types. Templates directory exists (`src/templates/`) but is empty (placeholder for Q4 2026).

### Key Pattern: data-moron Convention

Per specification section 6.3, elements use `data-moron` attributes for technique binding:
- `data-moron="container"` -- root layout wrapper
- `data-moron="title"` -- title text target
- `data-moron="sequence"` -- parent of staggerable items
- `data-moron="item"` (or `"sequence-item"` in current code) -- individual staggerable item

Techniques discover elements by these attributes. This convention means MoronFrame must either use these existing components or emit elements with the same `data-moron` attributes.

## Current packages/themes/ Structure

### CSS Custom Properties

`packages/themes/src/default.css` defines 56 CSS custom properties on `:root`, all prefixed `--moron-*`. Categories: colors (12), typography (17), spacing (14), timing (10), shadows (3).

### Tailwind Bridge

`packages/themes/tailwind.config.ts` maps `--moron-*` CSS custom properties to Tailwind utility classes. Content scans `../ui/src/**/*.{ts,tsx}`. This means MoronFrame code in `packages/ui/src/` will be picked up by Tailwind automatically.

### Theme Registry

`packages/themes/src/index.ts` exports a `themes` registry mapping theme names to CSS stylesheet paths. Currently only "default" is registered.

## Rust-Side Data Contract (T-004-01 Dependency)

T-004-01 defines `FrameState` -- the JSON data contract between Rust and React. T-004-01 is not yet implemented, but we can infer the shape from:

### TechniqueOutput (moron-techniques/src/technique.rs)

The visual transform state per element:

```
opacity: f64      (0.0 to 1.0)
translate_x: f64  (pixels)
translate_y: f64  (pixels)
scale: f64        (1.0 = normal)
rotation: f64     (degrees)
```

### Element Types from M Facade (moron-core/src/facade.rs)

The facade mints Element handles for content types:
- `m.show(text)` -- general text display
- `m.title(text)` -- title card
- `m.section(text)` -- section heading
- `m.metric(label, value, direction)` -- stat/KPI
- `m.steps(items)` -- sequential list

Each returns an opaque `Element(u64)` handle. Currently the facade does not store the text content or element type alongside the handle -- that is part of what T-004-01 must add.

### Theme as CSS Properties (moron-themes/src/theme.rs)

`Theme::to_css_properties()` produces `Vec<(String, String)>` of `("--moron-<token>", "<value>")` pairs. This is the mechanism for injecting theme values into the page. The Chromium bridge (T-004-03) will inject these as inline CSS custom properties.

### Expected FrameState Shape

Based on T-004-01's description and the existing types, the FrameState JSON will likely contain:

1. **elements**: array of visible elements, each with:
   - id (element handle)
   - type (title/section/show/metric/steps)
   - content (text, or structured data for metric/steps)
   - visual state (opacity, translate_x, translate_y, scale, rotation from TechniqueOutput)
2. **theme**: CSS custom property key-value pairs from `to_css_properties()`
3. **timeline metadata**: current_time, total_duration, frame_number

## Data Flow: Rust to React

The rendering pipeline per the specification (section 3.2):

1. Rust computes visual state at timestamp T
2. State serialized as JSON (FrameState)
3. JSON sent to headless Chromium via CDP (T-004-03)
4. React renders the frame using MoronFrame component
5. Chromium captures screenshot

MoronFrame sits at step 4. It receives the full FrameState as props, renders all visible elements with their CSS transforms, and applies theme custom properties to the container. The component does NOT animate -- animation is baked into the FrameState by Rust. Each frame is a static render.

## Relationship to Existing Components

The existing Container, Title, Sequence, and Metric components are building blocks for templates (higher-level compositions). MoronFrame is a lower-level renderer that sits between raw FrameState data and visual output. It can compose the existing components internally, or render elements directly using the `data-moron` attribute convention.

## Downstream Dependencies

- T-004-03 (Chromium bridge) needs MoronFrame to exist so it can load a page that renders frames.
- T-004-04 (frame rendering loop) iterates through the timeline and sends FrameState to the Chromium bridge.
- MoronFrame must be importable from `@moron/ui` and renderable in both browser and Node (SSR) environments.

## Constraints

- No runtime animation in MoronFrame. Every frame is a snapshot. CSS transitions/animations are not used.
- Component must be pure: same FrameState input always produces the same visual output.
- Must work in headless Chromium (no browser-specific APIs beyond standard DOM/CSS).
- Theme custom properties must be applied at the container level so all children inherit them.
- Z-ordering follows element array order (first element is lowest, last is highest).
