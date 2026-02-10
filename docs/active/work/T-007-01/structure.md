# T-007-01 Structure: Template System Architecture

## Files Created

### packages/ui/src/templates/registry.ts
Template registration system.

- `TemplateComponent` type: `React.ComponentType<{ state: FrameState; width?: number; height?: number }>`
- `registry`: Module-level `Map<string, TemplateComponent>`
- `registerTemplate(name: string, component: TemplateComponent): void` — adds to map, warns on overwrite
- `getTemplate(name: string): TemplateComponent` — returns component or MoronFrame default
- `listTemplates(): string[]` — returns array of registered names
- On module load: registers MoronFrame as "default"

### packages/ui/src/host.tsx
React entry point loaded by ChromiumBridge.

- Imports: React 19, createRoot, getTemplate, FrameState
- `App` component: holds FrameState in useState, renders looked-up template
- Module-level: creates root div, mounts App, exposes `window.__moron_setFrame`
- `window.__moron_setFrame(state)` triggers setState, which triggers re-render with correct template
- Declares global augmentation for `window.__moron_setFrame` type

### packages/ui/build.mjs
Node.js build script using esbuild programmatic API.

- Imports esbuild, fs, path
- Bundles `src/host.tsx` with esbuild (write: false, bundle: true, jsx: automatic)
- Reads `../themes/src/default.css`
- Constructs HTML string with inlined CSS and JS
- Writes to `dist/index.html`
- Logs success with output size

## Files Modified

### packages/ui/src/types.ts
- Add optional `template?: string` field to `FrameState` interface

### packages/ui/src/templates/index.ts
- Replace empty stub with re-exports from `./registry`
- Export `registerTemplate`, `getTemplate`, `listTemplates`, `TemplateComponent`

### packages/ui/src/index.ts
- Add re-exports for template registry: `registerTemplate`, `getTemplate`, `listTemplates`, `TemplateComponent`

### packages/ui/package.json
- Add `esbuild` to devDependencies
- Add `"build": "node build.mjs"` script

## Files NOT Modified

### moron-core/src/frame.rs
No Rust changes. The `template` field is TypeScript-only for now. When Rust adds it, it will be `Option<String>` with `skip_serializing_if`.

### packages/themes/*
No changes needed. The build script reads default.css directly.

### .gitignore
Already has `dist/` entry. No change needed.

### tsconfig.json
Already configured correctly (jsx: react-jsx, target: ES2022, moduleResolution: bundler). No change.

## Module Dependency Graph

```
host.tsx
  -> templates/registry.ts
    -> MoronFrame.tsx
    -> types.ts
  -> types.ts (FrameState)

build.mjs
  -> esbuild (dev dependency)
  -> reads ../themes/src/default.css
  -> bundles host.tsx -> dist/index.html
```

## Public API Surface

### From packages/ui/src/index.ts (library consumers):
- `registerTemplate(name, component)` — for plugins/extensions to add templates
- `getTemplate(name)` — for programmatic template lookup
- `listTemplates()` — for tooling/gallery
- `TemplateComponent` type — for typing template components

### From packages/ui/src/host.tsx (runtime, not exported):
- `window.__moron_setFrame(state: FrameState)` — ChromiumBridge contract

### From packages/ui/build.mjs (build time):
- `npm run build` — produces dist/index.html

## Directory Structure After Implementation

```
packages/ui/
  build.mjs                    # NEW — build script
  package.json                 # MODIFIED — add esbuild, build script
  tsconfig.json                # UNCHANGED
  src/
    index.ts                   # MODIFIED — add template re-exports
    types.ts                   # MODIFIED — add template field
    host.tsx                   # NEW — React entry point
    MoronFrame.tsx             # UNCHANGED
    components/                # UNCHANGED
      Container.tsx
      Title.tsx
      Sequence.tsx
      Metric.tsx
    templates/
      index.ts                 # MODIFIED — re-export registry
      registry.ts              # NEW — template registration system
  dist/                        # GENERATED (gitignored)
    index.html                 # Built output
```
