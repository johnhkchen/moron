# T-007-02 Design: Default Explainer Template

## 1. Decision: Single Component with Kind-Specific Renderers

### Options Considered

**A. Monolithic component** -- One `ExplainerTemplate` function with a switch/case for
each element kind, similar to MoronFrame's `renderContent`.

**B. Kind-specific sub-components** -- `ExplainerTitle`, `ExplainerSection`, etc. as
separate components, orchestrated by a top-level `ExplainerTemplate`.

**C. Compose existing primitives** -- Use Container, Title, Sequence, Metric from
`components/` and add styling via wrapper/className.

### Decision: Option B (kind-specific sub-components)

Rationale:
- Each element kind has distinct layout needs (title card wants full-bleed centered,
  steps wants left-aligned content area, metrics want card-style treatment). Separate
  components keep each layout self-contained and reviewable.
- Option A would create a single 300+ line function -- hard to review and modify.
- Option C is appealing but the existing primitives are too bare. They lack the layout
  opinions an explainer template needs (background treatment, responsive sizing, content
  width constraints). Wrapping them would add a layer without reducing complexity.
- The sub-components are private to the template module. They are not exported or reused
  directly. The public API is `ExplainerTemplate` alone.

## 2. Layout Architecture

### Root Container

The template root mirrors MoronFrame's container: fixed pixel dimensions, `position:
relative`, `overflow: hidden`, theme CSS properties as inline styles. This is non-negotiable
-- the Chromium bridge captures this exact box.

### Responsive Sizing Strategy

**Problem:** rem-based font sizes from the theme are relative to the browser's root
font-size (typically 16px). At 1080p this produces reasonable sizes (3.5rem = 56px for
titles). At 4K (3840x2160) the same 56px title is proportionally half the visual size.

**Solution:** Set the root container's `fontSize` proportionally to the frame width.
At 1920px wide, 1rem = 16px (the CSS default). Scale linearly:

```
fontSize = (width / 1920) * 16  // e.g., 3840 -> 32px, 1280 -> ~10.67px
```

This means all `var(--moron-text-*)` values (in rem) scale automatically with frame
dimensions. Applied as inline style on the template root div, not on `:root`, so it
does not leak to other components.

### Element Wrapper

Each visible element gets an absolute-positioned wrapper, same as MoronFrame, with
opacity/transform/z-index from ElementState. This is shared infrastructure -- the
template must respect the FrameState contract.

The difference: inside the wrapper, the template applies kind-specific layout rather
than generic centering.

### Content Area

For body-level elements (show, steps), constrain content width to prevent edge-to-edge
text at large frame sizes:

```
maxWidth: 75%        // of frame width
margin: 0 auto       // center the constrained block
padding: 0 var(--moron-container-padding)
```

Title and section cards use the full frame width (centered text does not need a
width constraint).

## 3. Kind-Specific Rendering Designs

### Title (`ExplainerTitle`)

- Full-frame centered, both axes
- `fontSize: var(--moron-text-4xl)`, `fontWeight: var(--moron-font-weight-bold)`
- `lineHeight: var(--moron-leading-tight)`
- Subtle accent underline: a thin bar (`4px` height, `80px` width) below the title text,
  colored `var(--moron-accent)` with `border-radius: var(--moron-radius-full)`
- Optional: faint radial gradient background centered on text area using
  `var(--moron-accent-subtle)` fading to transparent -- adds depth without competing
  with text. Applied as a pseudo-element effect or background on the wrapper.

Implementation note: Since this is rendered as static frames (no CSS animation), the
gradient is a fixed visual treatment, not animated.

### Section (`ExplainerSection`)

- Full-frame centered horizontally, vertically centered
- `fontSize: var(--moron-text-3xl)`, `fontWeight: var(--moron-font-weight-semibold)`
- `lineHeight: var(--moron-leading-tight)`
- Accent left border: `4px` solid `var(--moron-accent)` on the left side of the text,
  with `padding-left: var(--moron-space-6)`. This creates a visual break from title
  cards and body text.
- `color: var(--moron-fg-primary)` for the heading text

Design note: Sections serve as transitions. The left-border accent differentiates them
from titles (which are centered with an underline) and body text (which has no decoration).

### Show (`ExplainerShow`)

- Centered in frame, both axes
- Content area constrained to `maxWidth: 75%`
- `fontSize: var(--moron-text-xl)`, `fontWeight: var(--moron-font-weight-normal)`
- `lineHeight: var(--moron-leading-normal)`
- `color: var(--moron-fg-secondary)` -- slightly muted to distinguish from headings
- `textAlign: center`

This is the simplest element type. The key improvement over MoronFrame is the width
constraint and the muted color to create hierarchy (headings are fg-primary, body is
fg-secondary).

### Metric (`ExplainerMetric`)

- Centered in frame, both axes
- Card-style container with:
  - `background: var(--moron-bg-secondary)`
  - `border-radius: var(--moron-radius-lg)`
  - `padding: var(--moron-space-8) var(--moron-space-12)`
  - `box-shadow: var(--moron-shadow-md)`
- Value display: `fontSize: var(--moron-text-4xl)`, bold, `color: var(--moron-fg-primary)`
- Direction indicator: color-coded based on `kind.direction`:
  - `"up"` -- `color: var(--moron-success)` + upward arrow character
  - `"down"` -- `color: var(--moron-error)` + downward arrow character
  - `"neutral"` -- `color: var(--moron-fg-muted)`, no arrow
- Label: `fontSize: var(--moron-text-lg)`, `color: var(--moron-fg-muted)`, below value
- Layout: flex column, `gap: var(--moron-space-4)`, items centered

Arrow characters: Use Unicode arrows (U+2191 upward, U+2193 downward) placed before
the value text. These render reliably in Chromium with the Inter font stack. No SVG or
icon font dependency.

### Steps (`ExplainerSteps`)

- Centered in frame vertically, content area constrained to `maxWidth: 75%`
- Each step item is a row with:
  - **Number indicator:** Circular badge with `var(--moron-accent)` background, white
    text, `border-radius: var(--moron-radius-full)`, sized to fit single/double digits
    (`width: 2em, height: 2em, fontSize: var(--moron-text-base)`)
  - **Step text:** `fontSize: var(--moron-text-xl)`, `color: var(--moron-fg-primary)`,
    `lineHeight: var(--moron-leading-normal)`
- Row layout: `display: flex`, `align-items: center`, `gap: var(--moron-space-4)`
- Step spacing: `gap: var(--moron-space-6)` between rows
- Left-aligned text within the centered content area

The numbered badges provide clear visual hierarchy and progression that the current
bare-div approach lacks.

## 4. Color Usage Map

| Theme Property          | Used By                                         |
|-------------------------|-------------------------------------------------|
| `--moron-bg-primary`    | Frame background                                |
| `--moron-bg-secondary`  | Metric card background                          |
| `--moron-fg-primary`    | Title, section, metric value, step text          |
| `--moron-fg-secondary`  | Show text                                        |
| `--moron-fg-muted`      | Metric label, neutral direction                  |
| `--moron-accent`        | Title underline, section left border, step badge |
| `--moron-accent-subtle` | Title background radial gradient                 |
| `--moron-success`       | Metric direction "up"                            |
| `--moron-error`         | Metric direction "down"                          |

No hardcoded colors. All values from `var(--moron-*)`. Theme switching changes the
entire visual identity.

## 5. File Structure

```
packages/ui/src/templates/
  ExplainerTemplate.tsx    # Main component + sub-components
  index.ts                 # Re-export (updated to export ExplainerTemplate)
```

Single file for all sub-components. They are private implementation details -- no need
for separate files when the total is ~200-250 lines. If a sub-component exceeds ~80
lines, extract it to a separate file.

## 6. Component Interface

The template must conform to whatever interface T-007-01 defines. Based on MoronFrame's
existing props and the T-007-01 ticket description, the likely interface is:

```typescript
interface TemplateProps {
  state: FrameState;
  width?: number;
  height?: number;
}
```

`ExplainerTemplate` accepts these props and renders the full frame. It reuses
MoronFrame's `buildTransform` logic for element wrappers (or imports it if T-007-01
extracts it as shared utility).

## 7. Rejected Alternatives

**Tailwind classes instead of inline styles:** The Tailwind config maps to `var(--moron-*)`
properties, so `className="text-4xl"` resolves to `font-size: var(--moron-text-4xl)`.
However, MoronFrame uses inline styles consistently, the template renders in headless
Chromium where Tailwind must be pre-compiled, and inline styles are more explicit for
a rendering pipeline. Stay with inline styles for consistency.

**SVG arrows for metric direction:** More visually refined but adds SVG management,
sizing concerns, and Chromium rendering edge cases. Unicode arrows are sufficient for
v1 and render identically across platforms.

**CSS Grid for element layout:** Grid would allow named regions (title area, content
area, footer), but MoronFrame's absolute-positioning model means elements float
independently. Grid would fight the existing transform/position model. Keep absolute
positioning per element.

**Separate background component for title cards:** A dedicated background element would
allow animated gradients via FrameState. But this adds complexity to the Rust layer (a
new element kind). Inline background treatment on the title wrapper is simpler for v1.

## 8. Open Questions for T-007-01 Dependency

These will be resolved when T-007-01 completes:

1. What is the exact template registration interface? (name string -> component)
2. Is `buildTransform` extracted as shared utility, or must the template duplicate it?
3. Does the host page handle template selection, or does FrameState include a template name?
4. Are there lifecycle hooks (mount/unmount) or is it purely render-per-frame?

The explainer template design above is compatible with any reasonable answer to these
questions. The component accepts FrameState, renders elements, respects transforms.
The registration mechanism is orthogonal to the rendering logic.
