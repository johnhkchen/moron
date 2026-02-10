# T-007-01 Research: Template System Architecture

## Current Codebase State

### packages/ui module structure
- `src/index.ts` — re-exports base components (Container, Title, Sequence, Metric), MoronFrame, and types (FrameState, ElementState, ElementKind, ThemeState)
- `src/MoronFrame.tsx` — pure component that renders a single frame from FrameState JSON. Handles all element kinds inline (title, section, show, metric, steps). Applies theme CSS custom properties as inline styles.
- `src/types.ts` — TypeScript types mirroring Rust `frame::FrameState` serde output. FrameState contains time, frame, totalDuration, fps, elements[], activeNarration, theme.
- `src/components/` — four base components: Container, Title, Sequence, Metric. These are compositional building blocks, not used by MoronFrame directly.
- `src/templates/index.ts` — empty stub with comment "Templates will be added starting in Q4 2026 per the roadmap."
- `package.json` — React 19.2, TypeScript 5.8. Scripts: only `typecheck`. No build script. No bundler.
- `tsconfig.json` — jsx: react-jsx, target: ES2022, module: ESNext, moduleResolution: bundler

### packages/themes module
- `src/default.css` — 56 CSS custom properties under `:root` (colors, typography, spacing, radius, animation, shadows)
- `src/index.ts` — MoronTheme interface, themes registry (Record<string, MoronTheme>), only "default" theme registered
- Theme properties follow `--moron-*` naming convention

### moron-core/src/chromium.rs — ChromiumBridge
- `BridgeConfig` takes `html_path: PathBuf` pointing to built React app's `index.html`
- `launch()` builds `file://` URL from html_path, navigates Chrome to it
- Verifies `typeof window.__moron_setFrame === 'function'` after page load
- `capture_frame()` calls `window.__moron_setFrame(frameJson)` then double-rAF then screenshot
- The contract: page must expose `window.__moron_setFrame` as a global function that accepts FrameState JSON

### FrameState contract (Rust -> JS)
```typescript
interface FrameState {
  time: number;
  frame: number;
  totalDuration: number;
  fps: number;
  elements: ElementState[];
  activeNarration: string | null;
  theme: ThemeState;
}
```
No `template` field exists yet. Templates would need either a field on FrameState or a separate config mechanism.

### What does NOT exist yet
- No host HTML page (the entry point Chromium loads)
- No build script or bundler configuration
- No template registry (name -> component mapping)
- No `window.__moron_setFrame` implementation
- No esbuild dependency
- `packages/ui/dist/` directory doesn't exist (already in .gitignore as `dist/`)

## Key Constraints

1. **Self-contained HTML**: ChromiumBridge loads a `file://` URL. The built output must be a single HTML file with all JS/CSS inlined. No external CDN or network dependencies.
2. **React 19**: Uses createRoot API (react-dom/client). No legacy ReactDOM.render.
3. **FrameState-driven**: Templates receive the same FrameState. The template is a different visual layout, not different data.
4. **No template field on FrameState yet**: Adding a template field to FrameState requires changing the Rust struct too. This ticket should handle template selection without requiring Rust changes, or add the field minimally.
5. **Air-gapped**: Everything bundled. No network at render time.
6. **TypeScript strict mode**: tsconfig has `strict: true`.
7. **MoronFrame is the default**: Existing MoronFrame handles all element kinds. It should be the default/fallback template.

## Bundler Options

### esbuild
- Fast, zero-config for React/TypeScript
- Single binary, no plugin ecosystem issues
- Can bundle to single JS file. HTML wrapping needs a small script or plugin.
- `esbuild-plugin-html` or manual HTML template approach
- Already well-established, stable

### Alternatives (not recommended per design guidance)
- Vite: heavier, dev-server focused, overkill for this
- webpack: complex config, slow
- Rollup: more config than esbuild
- parcel: auto-config but heavier

Task guidance specifies esbuild. No need to evaluate further.

## HTML Inlining Strategy

esbuild outputs JS bundles. To produce a self-contained HTML file, we need to either:
1. Use a build script that bundles JS, then wraps it in an HTML template
2. Use an esbuild plugin for HTML generation
3. Use a simple Node.js build script: bundle with esbuild API, read output, inject into HTML template string, write index.html

Option 3 is the simplest and most maintainable for this use case.

## Template Selection Mechanism

Three approaches:
1. Add `template?: string` field to FrameState (requires Rust change)
2. Pass template name via a separate `window.__moron_setTemplate()` call
3. Have the host page determine template at build time (baked in)

The simplest is adding an optional `template` field to the TypeScript FrameState type. The Rust side can add it later; serde's `skip_serializing_if` means missing fields deserialize as the default. For now, TypeScript can treat it as optional and default to "default" (MoronFrame).
