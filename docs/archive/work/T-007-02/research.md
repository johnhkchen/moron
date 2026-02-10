# T-007-02 Research: Default Explainer Template

## 1. What Exists Today

### MoronFrame (packages/ui/src/MoronFrame.tsx)

The sole renderer. It receives a `FrameState` and produces a single React tree.

**Container setup:**
- Fixed pixel dimensions (default 1920x1080), `position: relative`, `overflow: hidden`
- Background: `var(--moron-bg-primary)`, color: `var(--moron-fg-primary)`, font: `var(--moron-font-sans)`
- Theme CSS custom properties are spread as inline styles on the root div, enabling
  CSS variable inheritance to all children

**Element wrapper (per-element):**
- `position: absolute; inset: 0` -- every element fills the full frame
- Flexbox centering: `align-items: center; justify-content: center`
- Opacity, transform (translate/scale/rotate), and z-index applied
- Invisible elements (`visible: false`) are skipped entirely

**Content renderers (switch on `kind.type`):**

| Kind      | Element  | Styling                                           | Notes                                     |
|-----------|----------|---------------------------------------------------|-------------------------------------------|
| `title`   | `<h1>`   | text-4xl, bold, lineHeight 1.2, center            | Bare text, no background treatment        |
| `section` | `<h2>`   | text-2xl, semibold, lineHeight 1.3, center        | data-moron="title" (bug: same as title)   |
| `show`    | `<p>`    | text-xl, lineHeight 1.5, center                   | No max-width constraint                   |
| `metric`  | `<div>`  | Flex column, value text-4xl bold, label text-lg   | Parses "label: value" from content string |
| `steps`   | `<div>`  | Flex column, gap space-4, items text-xl           | No numbering, no visual bullet/indicator  |

**Key observation:** MoronFrame is a functional baseline, not a polished template. Every
element occupies the same absolute-fill wrapper, content is centered without spatial
layout relationships between elements, and there is no background treatment, decoration,
or visual hierarchy beyond font size differences.

### Existing Components (packages/ui/src/components/)

Four reusable components exist but are NOT used by MoronFrame:

- **Container** -- full-size relative box, accepts children/className/style
- **Title** -- heading tag (h1/h2/h3), accepts level prop, unstyled
- **Sequence** -- flex column/row with gap, wraps children in sequence-item divs
- **Metric** -- value/label/unit display, no styling beyond structure

These are structural primitives. They have no visual polish -- no font sizes, colors,
spacing, or layout opinions. They are the building blocks a template should compose.

### CSS Custom Properties (packages/themes/src/default.css)

56 properties in 6 groups, all under `--moron-*` namespace:

**Colors (12 properties):**
- Backgrounds: `bg-primary` (#0f172a), `bg-secondary` (#1e293b), `bg-tertiary` (#334155)
- Foregrounds: `fg-primary` (#f8fafc), `fg-secondary` (#cbd5e1), `fg-muted` (#64748b)
- Accent: `accent` (#3b82f6), `accent-hover` (#60a5fa), `accent-subtle` (rgba blue 15%)
- Semantic: `success` (#22c55e), `warning` (#eab308), `error` (#ef4444)

**Typography (17 properties):**
- Fonts: `font-sans` (Inter), `font-mono` (JetBrains Mono)
- Sizes: xs through 4xl (0.75rem to 3.5rem) -- 8 steps
- Line heights: tight (1.15), normal (1.5), relaxed (1.75)
- Weights: normal (400), medium (500), semibold (600), bold (700)

**Spacing (14 properties):**
- Scale: space-1 through space-24 (0.25rem to 6rem) -- 9 steps
- Container padding: 3rem (references space-12)
- Border radius: sm/md/lg/full

**Timing (10 properties):**
- Durations: instant/fast/normal/slow/slower (0ms to 800ms)
- Easing: default/in/out/in-out/spring

**Shadows (3 properties):**
- sm/md/lg with increasing blur and opacity

**Theme palette is dark-on-dark:** Dark navy backgrounds (#0f172a) with near-white
foreground text (#f8fafc) and blue accent (#3b82f6). The semantic colors (success/warning/error)
map to the metric direction indicator use case.

### FrameState Data Contract (packages/ui/src/types.ts, moron-core/src/frame.rs)

- `ElementState.kind` is a discriminated union with `type` field
- `metric` has `direction: string` ("up"/"down"/"neutral")
- `steps` has `count: number` (total items) and items in `ElementState.items`
- Content for metrics is "label: value" in the `content` string
- Transform properties: opacity, translateX/Y, scale, rotation -- all numbers
- `visible` boolean controls whether element renders at all

### Template System (T-007-01 dependency)

T-007-01 defines the architecture: named templates that register with a name-to-component
mapping, receive FrameState, and compose with the rendering pipeline. The host page loads
the template and calls `window.__moron_setFrame()`. The `packages/ui/src/templates/index.ts`
exists as an empty scaffold.

The explainer template will be the first concrete implementation of whatever interface
T-007-01 defines. The template likely receives the same `FrameState` props as MoronFrame
but has freedom over how it renders elements.

## 2. Gaps Between MoronFrame and "Polished"

1. **No spatial layout** -- All elements fill `inset: 0` and center. No content regions,
   no margin management, no relationship between simultaneous elements.

2. **No background treatment** -- Title cards and sections have no visual backdrop,
   gradient, or decorative element. Just text on the frame background color.

3. **No visual indicators for metrics** -- Direction (up/down/neutral) is stored as a
   data attribute but has no visual representation (arrow, color coding, icon).

4. **No numbering or progression for steps** -- Items are plain divs with no bullet,
   number, or progress indicator. No visual hierarchy between steps.

5. **No responsive sizing** -- Font sizes use rem units. At 1080p the base font size
   comes from the browser (16px). At 4K (3840x2160) the same rem values would produce
   text that is proportionally tiny unless the root font-size is scaled.

6. **No content width constraint** -- Show text can stretch edge-to-edge. Professional
   explainers limit body text width for readability (~60-70ch or percentage-based).

7. **No section transitions** -- Section headings render identically to body text layout,
   just with a different font size. No visual break or separator.

8. **No use of accent/semantic colors** -- The theme provides accent blue, success green,
   warning yellow, error red. MoronFrame uses none of them for content styling.

9. **No shadows or radius** -- Available in theme but unused in rendering.

## 3. Explainer Video Visual Patterns

Standard explainer videos (think 3Blue1Brown, Kurzgesagt, corporate product videos) share
common visual patterns:

- **Title cards:** Large centered text, often with a subtle gradient background or
  accent-colored underline/decoration. Takes the full frame. Brief duration.
- **Section transitions:** Clean break from previous content. Often a horizontal rule,
  color shift, or slide-in from an edge. Section text is prominent but smaller than title.
- **Body text (show):** Constrained width (60-75% of frame), vertically centered or
  placed in a content region. Readable line length. Often fades in.
- **Metrics/KPIs:** Large prominent number, small label. Directional indicators use
  color (green=up, red=down) and sometimes arrows. Often in a card/panel with background.
- **Steps/lists:** Numbered or bulleted, revealed one at a time with stagger. Visual
  hierarchy via numbering and indentation. Left-aligned within a centered content area.

## 4. Constraints

- Template must use ONLY `var(--moron-*)` properties -- no hardcoded colors
- Must respect FrameState transforms (opacity, translate, scale, rotation, z-order)
- Must render correctly in headless Chromium (no CSS features that require user interaction)
- Must work at 1080p (1920x1080) and 4K (3840x2160)
- Must compose with whatever template interface T-007-01 defines
- Output is static frames (no CSS animations -- all motion comes from FrameState changes)
