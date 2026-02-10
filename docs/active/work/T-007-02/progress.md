# T-007-02 Progress: Default Explainer Template

## Completed Steps

### Step 1: ExplainerTemplate.tsx — Skeleton + Root Container
- Created `packages/ui/src/templates/ExplainerTemplate.tsx`
- Implemented `buildTransform` helper (duplicated from MoronFrame)
- Implemented `ExplainerTemplate` main component with:
  - Responsive fontSize: `(width / 1920) * 16` px scaling
  - Theme CSS custom properties spread on root container
  - Container: fixed dimensions, position relative, overflow hidden
  - Element wrapper loop: skips invisible, applies opacity/transform/zIndex
  - `data-template="explainer"` attribute on root for identification
- Self-registration: `registerTemplate("explainer", ExplainerTemplate)`

### Step 2: ExplainerTitle
- Full-frame centered h1 with text-4xl, bold, leading-tight
- Accent underline bar: 80px wide, 4px tall, accent color, full radius
- Radial gradient background: accent-subtle fading to transparent
- Wrapped in flex column with space-6 gap

### Step 3: ExplainerSection
- Centered h2 with text-3xl, semibold, leading-tight
- Accent left border: 4px solid accent with space-6 padding-left
- Container padding from theme

### Step 4: ExplainerShow
- Centered paragraph with maxWidth 75% constraint
- text-xl, fg-secondary color, leading-normal, center aligned
- Container padding from theme

### Step 5: ExplainerMetric
- Card container: bg-secondary, radius-lg, padding space-8/space-12, shadow-md
- Value: text-4xl, bold, fg-primary, leading-tight
- Direction arrows: Unicode U+2191 (up) / U+2193 (down)
  - "up" -> success color (green)
  - "down" -> error color (red)
  - "neutral" -> fg-muted, no arrow
- Label: text-lg, fg-muted, below value
- Flex column layout, gap space-4, items centered
- Parses "label: value" from content string (matching MoronFrame convention)

### Step 6: ExplainerSteps
- Container with maxWidth 75%, centered via flexbox
- Each item: flex row with numbered badge + text, gap space-4
- Badge: circular (2em x 2em, radius-full), accent bg, fg-primary text,
  text-base font, bold weight, flex centered
- Text: text-xl, fg-primary, leading-normal
- Column gap: space-6 between rows

### Step 7: Updated templates/index.ts
- Added `export { ExplainerTemplate } from "./ExplainerTemplate";`

### Step 8: Updated host.tsx
- Added side-effect import: `import "./templates/ExplainerTemplate";`
- Ensures registration runs when host page loads

### Step 9: Updated index.ts
- Added `export { ExplainerTemplate } from "./templates/ExplainerTemplate";`
- Makes component accessible from package root

### Step 10: Full Verification
- `npm run typecheck` -- passed (zero errors)
- `npm run build` -- passed (dist/index.html produced at 1075.3 KB)

## Deviations from Plan

None. Implementation followed the plan exactly.

## Color Usage Verification

All colors reference `var(--moron-*)` properties:
- `--moron-bg-primary` -- frame background
- `--moron-bg-secondary` -- metric card background
- `--moron-fg-primary` -- title, section, metric value, step text, step badge text
- `--moron-fg-secondary` -- show text
- `--moron-fg-muted` -- metric label, neutral direction color
- `--moron-accent` -- title underline, section left border, step badge background
- `--moron-accent-subtle` -- title radial gradient background
- `--moron-success` -- metric "up" direction arrow
- `--moron-error` -- metric "down" direction arrow

No hardcoded color values in the template file.

## Files Changed

| File | Action |
|------|--------|
| `packages/ui/src/templates/ExplainerTemplate.tsx` | CREATED (278 lines) |
| `packages/ui/src/templates/index.ts` | MODIFIED (+3 lines) |
| `packages/ui/src/host.tsx` | MODIFIED (+3 lines) |
| `packages/ui/src/index.ts` | MODIFIED (+3 lines) |
