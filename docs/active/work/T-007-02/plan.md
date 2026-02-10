# T-007-02 Plan: Default Explainer Template

## Implementation Steps

### Step 1: Create ExplainerTemplate.tsx — Skeleton + Root Container

Create the file with:
- All imports (react, types, registry)
- `buildTransform` helper (duplicated from MoronFrame)
- `ExplainerTemplate` main component with responsive fontSize, theme spreading,
  container div, element wrapper loop (skip invisible, apply transforms)
- Placeholder `renderExplainerContent` that returns `null` for all kinds
- Self-registration: `registerTemplate("explainer", ExplainerTemplate)`

**Verify:** `npm run typecheck` passes. The component compiles and matches
TemplateProps interface.

### Step 2: Implement ExplainerTitle Sub-Component

Add `ExplainerTitle` function:
- Centered h1 with text-4xl, bold, leading-tight
- Accent underline bar: div below text, 4px height, 80px width, accent color,
  radius-full, centered via margin auto
- Radial gradient background on wrapper using accent-subtle
- Wire into `renderExplainerContent` switch case for "title"

**Verify:** TypeScript compiles. Title rendering path is complete.

### Step 3: Implement ExplainerSection Sub-Component

Add `ExplainerSection` function:
- Centered h2 with text-3xl, semibold, leading-tight
- Accent left border: 4px solid accent, padding-left space-6
- Wire into switch case for "section"

**Verify:** TypeScript compiles.

### Step 4: Implement ExplainerShow Sub-Component

Add `ExplainerShow` function:
- Centered paragraph with maxWidth 75%
- text-xl, fg-secondary, leading-normal, center aligned
- Wire into switch case for "show"

**Verify:** TypeScript compiles.

### Step 5: Implement ExplainerMetric Sub-Component

Add `ExplainerMetric` function:
- Parse "label: value" from content string (same logic as MoronFrame)
- Card container: bg-secondary, radius-lg, padding space-8/space-12, shadow-md
- Value: text-4xl, bold, fg-primary
- Direction arrow: Unicode U+2191 (up) / U+2193 (down), colored success/error/fg-muted
- Label: text-lg, fg-muted, below value
- Flex column layout, gap space-4, items centered
- Wire into switch case for "metric"

**Verify:** TypeScript compiles. Metric kind type narrowing works correctly
(accessing `kind.direction` requires type guard).

### Step 6: Implement ExplainerSteps Sub-Component

Add `ExplainerSteps` function:
- Container with maxWidth 75%, centered via margin auto
- Each item: flex row with numbered badge + text
- Badge: circular (width/height 2em, radius-full), accent bg, white text,
  flex centered, text-base font size
- Text: text-xl, fg-primary, leading-normal
- Row gap: space-4. Column gap: space-6
- Wire into switch case for "steps"

**Verify:** TypeScript compiles. Steps kind type narrowing works correctly
(accessing `kind.count` if needed).

### Step 7: Update templates/index.ts

Add re-export:
```typescript
export { ExplainerTemplate } from "./ExplainerTemplate";
```

**Verify:** TypeScript compiles. Export is accessible.

### Step 8: Update host.tsx

Add side-effect import:
```typescript
import "./templates/ExplainerTemplate";
```

This ensures registration runs when the host page loads.

**Verify:** TypeScript compiles. Build produces working output.

### Step 9: Update index.ts

Add package export:
```typescript
export { ExplainerTemplate } from "./templates/ExplainerTemplate";
```

**Verify:** TypeScript compiles. Symbol is accessible from package root.

### Step 10: Full Verification

Run both verification commands:
- `npm run typecheck` — strict TypeScript compilation with no errors
- `npm run build` — esbuild bundles host.tsx and all dependencies into
  dist/index.html without errors

Check that the build output size is reasonable (should increase by a few KB
from the new template code).

## Testing Strategy

### Automated Verification
- `npm run typecheck` validates all type contracts (TemplateProps, ElementState,
  ElementKind discriminated union narrowing)
- `npm run build` validates the bundling pipeline (no import errors, no missing
  dependencies, esbuild resolves all modules)

### Manual Verification (not in scope for this ticket)
- Visual inspection requires the full Rust pipeline (ChromiumBridge + frame
  generation) which is not yet integrated
- Screenshot testing is a future ticket concern

### Type Safety Checks
- ExplainerTemplate matches TemplateProps (enforced by registerTemplate call)
- ElementKind narrowing in switch cases (TypeScript exhaustiveness)
- CSSProperties type compliance for all inline styles
- No `any` types used

## Risk Mitigation

- **buildTransform duplication:** Accepted tech debt. Extracting it to a shared
  module would require modifying MoronFrame, which is outside scope. The function
  is 12 lines and stable.

- **Side-effect registration:** The import in host.tsx is the reliability
  mechanism. Even if tree-shaking removes the re-export from index.ts, the
  explicit side-effect import ensures registration.

- **CSS var() type casting:** TypeScript's CSSProperties does not understand
  `var()` strings for properties like `fontWeight`. Use `as CSSProperties["fontWeight"]`
  cast following MoronFrame's existing pattern.
