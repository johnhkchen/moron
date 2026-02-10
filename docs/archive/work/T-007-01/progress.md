# T-007-01 Progress: Template System Architecture

## Status: Complete

## Completed Steps

- [x] Step 1: Add template field to FrameState — added `template?: string` to `FrameState` in types.ts
- [x] Step 2: Create template registry — `src/templates/registry.ts` with registerTemplate/getTemplate/listTemplates, MoronFrame as "default"
- [x] Step 3: Update templates index — re-exports from registry
- [x] Step 4: Update package exports — added template re-exports to `src/index.ts`
- [x] Step 5: Create host page — `src/host.tsx` with App component, window.__moron_setFrame, React 19 createRoot
- [x] Step 6: Add esbuild dependency — added esbuild ^0.25.0 to devDependencies, added build script
- [x] Step 7: Create build script — `build.mjs` bundles host.tsx + default.css into dist/index.html
- [x] Step 8: Verify everything — typecheck passes, build produces 1065 KB self-contained index.html

## Verification Results

- `npm run typecheck` — passes (0 errors)
- `npm run build` — produces dist/index.html (1065.2 KB)
- dist/index.html contains inlined JS bundle and default theme CSS
- dist/index.html defines window.__moron_setFrame
- dist/index.html has proper HTML structure (#root div, module script, CSS)

## Deviations from Plan

### useRef initial value (Step 5)
React 19 types require an explicit initial value for `useRef`. Changed from `useRef<(state: FrameState) => void>()` to `useRef<((state: FrameState) => void) | undefined>(undefined)`. This is a minor type-level adjustment, not a design change.
