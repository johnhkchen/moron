# T-007-03 Research: Theme CSS Integration

## 1. Current Architecture Overview

The theme pipeline spans three layers: Rust (source of truth), JSON (transport),
and React (rendering). Each layer has its own representation of theme data.

### Rust Layer (moron-themes)

**`moron-themes/src/theme.rs`** defines the `Theme` struct with five sub-structs:
`ThemeColors` (12 tokens), `ThemeTypography` (17 tokens), `ThemeSpacing` (14 tokens),
`ThemeTiming` (10 tokens), `ThemeShadows` (3 tokens). Total: 56 design tokens.

**`moron-themes/src/defaults.rs`** implements `Default for Theme` with the name
"moron-dark". Every value here is hardcoded to match `packages/themes/src/default.css`.
This is a manual mirror -- no codegen, no verification that the two stay in sync.

**`Theme::to_css_properties()`** converts the struct into a `Vec<(String, String)>`
of 56 `--moron-*` key-value pairs. This is a flat, manual mapping -- each field is
pushed individually with its corresponding CSS custom property name.

### JSON Transport Layer (moron-core)

**`moron-core/src/frame.rs`** defines `ThemeState { name: String, css_properties: HashMap<String, String> }`.
The `compute_frame_state()` function calls `m.current_theme().to_css_properties()`
and collects it into a HashMap, then embeds it in `FrameState.theme`.

The `M` facade (`moron-core/src/facade.rs`) holds `current_theme: Theme` and
exposes `theme(&mut self, theme: Theme)` for switching and `current_theme(&self) -> &Theme`
for reading. Theme is set once and remains static for the entire scene unless
explicitly changed via `m.theme(...)`.

### React Rendering Layer (packages/ui)

**`packages/ui/src/types.ts`** mirrors the Rust types: `ThemeState { name: string; cssProperties: Record<string, string> }`.

**`packages/ui/src/MoronFrame.tsx`** applies theme CSS properties as **inline styles**
on the root container div. It iterates `state.theme.cssProperties` and spreads them
into the container's `style` attribute. Child elements reference these via
`var(--moron-*)` in their own inline styles (e.g., `fontSize: "var(--moron-text-4xl)"`).

This works because CSS custom properties set as inline styles on a parent element
cascade to children via CSS variable inheritance -- `var()` references resolve
against the nearest ancestor that defines the property.

### CSS Layer (packages/themes)

**`packages/themes/src/default.css`** defines the same 56 properties on `:root`.
This file is NOT currently imported or loaded anywhere in the rendering pipeline.
It exists as a standalone CSS file.

**`packages/themes/src/index.ts`** defines a `MoronTheme` interface with `name` and
`stylesheet` (URL path). The `themes` registry maps `"default"` to the CSS file URL.
This registry is not consumed by any component or host page currently.

**`packages/themes/tailwind.config.ts`** bridges CSS custom properties to Tailwind
utility classes. Every Tailwind token (colors, fonts, spacing, etc.) maps to a
`var(--moron-*)` reference. This enables Tailwind classes like `bg-moron-bg-primary`
to resolve through CSS variables. The `content` key scans `../ui/src/**/*.{ts,tsx}`.

## 2. Data Flow Analysis

### Current flow (working):

```
Rust Theme::default()
  -> Theme::to_css_properties() -> Vec<(String, String)>
    -> compute_frame_state() -> ThemeState { css_properties: HashMap }
      -> JSON serialization (camelCase: cssProperties)
        -> MoronFrame reads state.theme.cssProperties
          -> Spread as inline styles on root div
            -> Children use var(--moron-*) references
```

This flow is complete and functional. The 56 CSS custom properties are set as
inline styles on the `<div data-moron="frame">` element and cascade to all
children. Every `var(--moron-*)` reference in child inline styles resolves
correctly.

### What default.css would provide (not currently wired):

```
default.css loaded via <link> or <style> in host page
  -> :root { --moron-*: values }
    -> Available globally to all elements via CSS inheritance
```

If default.css were loaded, its `:root` properties would be overridden by the
inline styles from FrameState, because inline styles have higher specificity
than `:root` declarations. This means default.css would serve as a fallback --
properties defined in FrameState win, properties missing from FrameState fall
back to the CSS file.

## 3. Redundancy Analysis

The Rust `defaults.rs` and the CSS `default.css` contain **identical values**.
Both define the same 56 tokens with the same values. This is intentional: the
Rust side is the source of truth for the rendering pipeline, and the CSS side
is the source of truth for Tailwind class generation and potential standalone
CSS usage.

Current inline-style approach makes default.css **entirely redundant** for
rendering. All 56 properties are always present in FrameState.theme.cssProperties,
so var() references always resolve from the inline styles.

However, default.css becomes relevant when:
1. The host page needs theme values before FrameState is injected (initial render)
2. Tailwind classes are used (they need CSS variables defined somewhere)
3. Templates want to use CSS classes instead of inline styles
4. A developer inspects the page and wants sensible defaults without JS

## 4. Theme Switching

Current `m.theme(theme: Theme)` on the facade replaces the entire theme.
`compute_frame_state` always reads the current theme, so switching themes
between frames works at the Rust level.

On the React side, MoronFrame re-spreads the new theme's cssProperties as
inline styles on every render. Since the component is pure (same input = same
output), theme switching is already supported -- a new FrameState with different
theme values produces different visual output.

What is missing for robust theme switching:
- No second theme exists (only "moron-dark" default is implemented)
- No CSS file for alternate themes exists
- The themes registry in `index.ts` only has one entry
- No test verifies that different theme values produce different renders

## 5. Host Page (T-007-01 Dependency)

T-007-01 is creating the host HTML page (`packages/ui/src/host.tsx`) that:
1. Bundles React + MoronFrame into a single HTML file
2. Exposes `window.__moron_setFrame(frameState)` for the Chromium bridge
3. Is buildable via `npm run build`

The host page is where default.css should be imported. Without the host page,
there is no HTML document to attach a `<style>` or `<link>` element to.
This is why T-007-03 depends on T-007-01.

## 6. Component Usage of Theme Properties

MoronFrame's `renderContent()` uses these CSS custom properties in inline styles:
- `--moron-text-4xl`, `--moron-text-2xl`, `--moron-text-xl`, `--moron-text-lg`
- `--moron-font-weight-bold`, `--moron-font-weight-semibold`
- `--moron-space-4`

The root container uses:
- `--moron-bg-primary` (background)
- `--moron-fg-primary` (color)
- `--moron-font-sans` (fontFamily)

The base components (Container, Title, Sequence, Metric) do NOT reference
any theme properties -- they accept arbitrary `style` and `className` props.
Theme integration happens entirely in MoronFrame.

## 7. Tailwind Integration Status

The Tailwind config bridges all 56 CSS variables to utility classes, but:
- No component currently uses Tailwind classes
- No build pipeline processes Tailwind (no PostCSS config in packages/ui)
- The `content` array scans UI components, but there is nothing to extract
- Tailwind is a devDependency of @moron/themes, not @moron/ui

Tailwind integration is architecturally prepared but not yet active.

## 8. Key Constraints

- T-007-01 must deliver the host page before CSS can be loaded into it
- The inline-style approach works now and must not break
- CSS custom properties from FrameState must override CSS file defaults
- All 56 properties must be present (no partial themes)
- Theme name is carried through the pipeline for identification
- The rendering pipeline is pure: same FrameState = same visual output

## 9. Open Questions

1. Should default.css be the fallback or should FrameState always carry all 56?
   Currently FrameState always carries all 56 (Theme struct has no Option fields).
2. Should theme switching happen mid-scene or only at scene boundaries?
   The facade supports mid-scene switching via `m.theme()`, but compute_frame_state
   uses a single theme for the entire frame.
3. Should Tailwind class usage be in scope or deferred?
   Templates (T-007-02) may want Tailwind classes, but that is their decision.
4. Should a second theme be created as part of this ticket for testing?
   The acceptance criteria say "switching themes produces visually different output"
   and "render same frame with two themes, verify different output".
