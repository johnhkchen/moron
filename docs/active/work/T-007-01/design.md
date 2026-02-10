# T-007-01 Design: Template System Architecture

## Decision Summary

Build a template registry as a simple `Map<string, ComponentType<FrameState>>`, a host page (`host.tsx`) that wires React to `window.__moron_setFrame`, and a Node.js build script that uses esbuild's programmatic API to produce a self-contained `dist/index.html`.

## Template Registry

### Chosen Approach: Module-level Map with register/get functions

```typescript
type TemplateComponent = React.ComponentType<{ state: FrameState }>;

const registry = new Map<string, TemplateComponent>();

export function registerTemplate(name: string, component: TemplateComponent): void;
export function getTemplate(name: string): TemplateComponent;
export function listTemplates(): string[];
```

MoronFrame is pre-registered as "default" on module load.

### Why not a class-based registry?
A class adds indirection for no benefit. The registry is a singleton — there is exactly one template map per page. Module-level functions are simpler, tree-shakeable, and follow the pattern already used in `packages/themes/src/index.ts`.

### Why not React context for template selection?
Context is useful when template selection is dynamic and deeply nested. Here, the host page selects the template once per frame update based on FrameState. A simple function call is clearer than context indirection.

## Template Type Contract

Add a `template?: string` field to the TypeScript `FrameState` interface. This is backward-compatible: if Rust doesn't send it, it's `undefined`, and the host falls back to "default".

No Rust changes required for this ticket. When the Rust side wants to control templates, it adds the field to `frame::FrameState` with `#[serde(skip_serializing_if = "Option::is_none")]`.

Template components receive the same props as MoronFrame:
```typescript
interface TemplateProps {
  state: FrameState;
  width?: number;
  height?: number;
}
```

This mirrors `MoronFrameProps` exactly. Templates are drop-in replacements for MoronFrame with different layouts.

## Host Page (host.tsx)

The host page is the React entry point loaded by ChromiumBridge.

### Architecture:
1. Import React 19, createRoot from react-dom/client
2. Import template registry (which auto-registers MoronFrame as "default")
3. Create a root div, mount React
4. Expose `window.__moron_setFrame = (state: FrameState) => { ... }` that triggers re-render
5. The render function looks up `state.template` in the registry, falls back to "default"

### State management:
Use a module-level variable + root.render() call. No useState needed since the update comes from outside React (the Chromium bridge calling the global function). Each call to `__moron_setFrame` does a full `root.render(<Template state={state} />)`.

### Why not useState + useEffect?
The state update originates from JavaScript called by Chromium CDP, not from React events. Calling `root.render()` directly is simpler and avoids the need for a wrapper component that holds state. React 19's createRoot makes repeated render() calls efficient.

**Revised: use a wrapper App component with a ref-based update mechanism.** This is cleaner than repeated root.render() calls and plays better with React's concurrent features. The App component holds state, and `window.__moron_setFrame` updates it via a callback ref.

## Build System

### Chosen: Node.js build script using esbuild programmatic API

File: `packages/ui/build.mjs`

Steps:
1. Call `esbuild.build()` to bundle `src/host.tsx` into a single JS string (write: false)
2. Read the default CSS from `../themes/src/default.css`
3. Inject both into an HTML template string
4. Write to `dist/index.html`

### Why a script instead of CLI flags?
The CLI can't inline JS into HTML. We need programmatic access to the bundle output to embed it in the HTML template. A ~40 line build script handles this cleanly.

### HTML template structure:
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=1920,height=1080">
  <style>{DEFAULT_CSS}</style>
</head>
<body>
  <div id="root"></div>
  <script>{BUNDLED_JS}</script>
</body>
</html>
```

### npm scripts:
```json
{
  "build": "node build.mjs",
  "typecheck": "tsc --noEmit"
}
```

### esbuild config:
- bundle: true
- format: esm (for top-level await if needed, though not currently used)
- platform: browser
- target: es2022
- minify: false (readable output for debugging; minify can be added later)
- write: false (capture output in memory)
- jsx: automatic (matches tsconfig jsx: react-jsx)

## Rejected Alternatives

### esbuild HTML plugin
Adds a dependency for something achievable in ~10 lines of script. Not worth it.

### Vite / Rollup
Over-engineered for a single entry point that produces a static HTML file. esbuild is already the standard fast bundler.

### Inline everything in a single .html file manually
No bundler means no JSX, no TypeScript, no imports. Not viable.

## Template API Surface

Exported from `packages/ui/src/templates/registry.ts`:
- `registerTemplate(name, component)` — register a named template
- `getTemplate(name)` — get template by name, returns MoronFrame if not found
- `listTemplates()` — list all registered template names
- `TemplateComponent` type alias

Exported from `packages/ui/src/types.ts`:
- Updated `FrameState` with optional `template?: string`

Exported from `packages/ui/src/index.ts`:
- Re-exports registry functions and TemplateComponent type
