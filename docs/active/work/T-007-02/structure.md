# T-007-02 Structure: Default Explainer Template

## 1. File Changes Overview

| File | Action | Purpose |
|------|--------|---------|
| `packages/ui/src/templates/ExplainerTemplate.tsx` | CREATE | Main template component + 5 kind sub-components |
| `packages/ui/src/templates/index.ts` | MODIFY | Add ExplainerTemplate re-export |
| `packages/ui/src/host.tsx` | MODIFY | Import ExplainerTemplate to trigger registration |
| `packages/ui/src/index.ts` | MODIFY | Add ExplainerTemplate to package exports |

No files deleted. No changes outside `packages/ui/src/`.

## 2. ExplainerTemplate.tsx — Module Structure

Single file, ~250 lines. Contains:

### Imports

```
react (CSSProperties, ReactNode)
../types (ElementState, FrameState)
./registry (registerTemplate, TemplateProps)
```

### Internal Helpers

**`buildTransform(el: ElementState): string`**
Duplicated from MoronFrame. Identical logic: combines translateX/Y, scale, rotation
into a CSS transform string. Not extracting to shared utility — that is outside this
ticket's scope and would require modifying MoronFrame.

### Sub-Components (not exported)

Five internal functions, each takes `{ el: ElementState }` and returns ReactNode:

1. **`ExplainerTitle`** — Full-frame centered h1 with accent underline bar and
   radial gradient background. Uses text-4xl, bold, leading-tight.

2. **`ExplainerSection`** — Full-frame centered h2 with accent left border.
   Uses text-3xl, semibold, leading-tight. padding-left for border offset.

3. **`ExplainerShow`** — Centered paragraph with maxWidth 75% constraint.
   Uses text-xl, fg-secondary color, leading-normal.

4. **`ExplainerMetric`** — Card with bg-secondary background, shadow-md, radius-lg.
   Value in text-4xl bold. Direction arrows (Unicode U+2191/U+2193) color-coded
   via success/error/fg-muted. Label in text-lg fg-muted below value.

5. **`ExplainerSteps`** — Content area maxWidth 75%. Each item is a flex row with
   numbered circular accent badge (radius-full, accent bg, white text) and step
   text in text-xl fg-primary.

### Content Router

**`renderExplainerContent(el: ElementState): ReactNode`**
Switch on `el.kind.type`, dispatches to the appropriate sub-component.

### Main Component

**`ExplainerTemplate({ state, width, height }: TemplateProps)`**
- Computes responsive fontSize: `(width / 1920) * 16` px
- Spreads theme cssProperties as inline styles on root div
- Root div: fixed dimensions, position relative, overflow hidden, bg-primary,
  fg-primary, font-sans, responsive fontSize
- Maps over `state.elements`, skips invisible, wraps each in absolute-positioned
  div with opacity/transform/zIndex from ElementState
- Calls `renderExplainerContent` for inner content

### Self-Registration

Bottom of file:
```typescript
registerTemplate("explainer", ExplainerTemplate);
```

This executes at module load time. The host page must import this module (directly
or transitively) for registration to occur.

## 3. templates/index.ts — Modifications

Add one line to existing exports:

```typescript
export { ExplainerTemplate } from "./ExplainerTemplate";
```

This goes after the existing registry re-exports. It both re-exports the component
and triggers the module's side-effect registration.

## 4. host.tsx — Modifications

Add one import line near the top imports:

```typescript
import "./templates/ExplainerTemplate";
```

This is a side-effect import. It ensures ExplainerTemplate's module executes during
host page initialization, which triggers `registerTemplate("explainer", ...)`.
Without this import, the template is only registered if something else imports it.

The host already imports `getTemplate` from `./templates`, but the index.ts barrel
does not re-export ExplainerTemplate at module level in a way that triggers the
side effect (it only re-exports the symbol). The explicit import in host.tsx
guarantees registration regardless of tree-shaking.

## 5. index.ts — Modifications

Add ExplainerTemplate to package exports:

```typescript
export { ExplainerTemplate } from "./templates/ExplainerTemplate";
```

This allows external consumers to import the component directly if needed (e.g.,
for testing or custom host pages).

## 6. Component Interface Compliance

ExplainerTemplate implements `TemplateProps` from `registry.ts`:

```typescript
interface TemplateProps {
  state: FrameState;
  width?: number;
  height?: number;
}
```

Width defaults to 1920, height defaults to 1080 — matching MoronFrame's defaults.

## 7. Styling Architecture

All styling is inline `CSSProperties` objects. No CSS files, no Tailwind classes,
no styled-components. This matches MoronFrame's approach and works reliably in
the headless Chromium capture pipeline.

Every color value is a `var(--moron-*)` reference. No hex codes, no rgb(), no
hardcoded values in the template file.

Font sizes, spacing, radii, shadows, and weights all reference `var(--moron-*)`
properties. The only numeric values are:
- Layout constants (maxWidth percentages, badge dimensions in em units)
- The responsive fontSize calculation
- Zero values for margins/padding resets

## 8. Data Attribute Conventions

Following MoronFrame's `data-moron` pattern:
- Root: `data-moron="frame"`
- Element wrappers: `data-moron="element"` with `data-element-id`
- Content elements: `data-moron="explainer-title"`, `data-moron="explainer-section"`, etc.

The "explainer-" prefix distinguishes template-specific markup from MoronFrame's
generic markup, useful for testing and debugging.

## 9. Dependency Graph

```
host.tsx
  -> ./templates/ExplainerTemplate  (side-effect import for registration)
  -> ./templates (getTemplate)
       -> ./registry (getTemplate, registerTemplate)
       -> ./ExplainerTemplate (re-export)
            -> ./registry (registerTemplate, TemplateProps)
            -> ../types (ElementState, FrameState)
```

No circular dependencies. ExplainerTemplate depends on registry and types.
Registry depends on MoronFrame. No cross-dependency between ExplainerTemplate
and MoronFrame.
