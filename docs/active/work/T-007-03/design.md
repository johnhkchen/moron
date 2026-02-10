# T-007-03 Design: Theme CSS Integration

## 1. Problem Statement

The theme pipeline from Rust to rendered output works functionally (inline styles
carry all 56 CSS custom properties), but the CSS layer (`default.css`) is not
wired into the host page, no second theme exists for testing theme switching, and
there is no verification that the Rust defaults and CSS defaults stay in sync.

This ticket must:
- Load default.css in the host page as a baseline
- Ensure FrameState inline styles correctly override CSS file defaults
- Create a second theme to verify theme switching
- Add an integration test proving two themes produce different output

## 2. Options Considered

### Option A: CSS File as Sole Source, Remove Inline Styles

Remove the inline-style injection in MoronFrame. Load theme CSS files via
`<link>` or `<style>` in the host page. Theme switching swaps the stylesheet.

**Pros:** Single source of truth in CSS. Cleaner DOM (no inline styles).
**Cons:** Breaks the pure-component contract (MoronFrame output depends on
external CSS state, not just its props). Theme values become invisible in
FrameState JSON, complicating debugging and testing. Requires synchronous
stylesheet swapping before screenshot capture. Fundamentally changes the
architecture.

**Rejected.** The FrameState-carries-everything design is a core architectural
decision. The rendering pipeline must be deterministic from FrameState alone.

### Option B: Inline Styles Only, CSS File for Tailwind/Dev Only

Keep the current inline-style approach as the sole rendering mechanism. Load
default.css only for Tailwind class generation and developer tooling. The CSS
file never affects rendering -- it is purely a build-time artifact.

**Pros:** Simplest. No cascade concerns. FrameState is the complete truth.
**Cons:** default.css in the host page serves no rendering purpose. If a
component omits a var() reference and no inline style defines it, the property
is undefined. No fallback safety net. Tailwind classes still need the CSS
variables defined somewhere at runtime if any template uses them.

**Viable but incomplete.** Templates from T-007-02 may use Tailwind classes,
which need CSS variables defined in the DOM, not just at build time.

### Option C: CSS File as Baseline + Inline Styles as Override (Chosen)

Load default.css in the host page to establish baseline CSS custom property
values on `:root`. MoronFrame continues to inject all theme properties as
inline styles on the frame container. Inline styles override `:root` values
due to CSS specificity (inline > any selector).

**Pros:**
- Fallback safety: if a property is somehow missing from FrameState, the CSS
  file provides a sensible default.
- Tailwind-ready: CSS variables are defined in the DOM for any Tailwind class
  usage by templates.
- No architecture change: FrameState remains the complete truth for rendering.
  The CSS file is defense-in-depth, not a primary mechanism.
- Dev-friendly: inspecting the page shows theme values even before JS runs.
- Theme switching: swap the CSS file reference to change the baseline, then
  FrameState overrides fill in the runtime values.

**Cons:**
- Two sources of truth for default values (Rust defaults.rs and CSS default.css).
  These must stay in sync. A sync-check test mitigates this risk.
- Slightly larger DOM (inline styles + CSS file), but negligible for a
  rendering pipeline that captures screenshots.

**Chosen.** This approach provides the most robustness with the least
architectural disruption.

## 3. Detailed Design

### 3.1 Load Theme CSS in Host Page

The host page (created by T-007-01) must import or inline the default theme CSS.
Two sub-options:

**3.1a: Static import at build time.** The host page entry point imports
`@moron/themes/src/default.css` directly. The bundler (likely Vite or esbuild,
decided by T-007-01) inlines it into the output HTML. Simple, zero-runtime cost.

**3.1b: Dynamic injection at runtime.** The host page reads the theme name from
FrameState and injects a `<style>` block with the corresponding CSS. More
flexible but adds complexity.

**Decision: 3.1a for the default theme.** Import default.css statically. For
theme switching, add a mechanism to swap or layer additional CSS. The static
import ensures the default is always available.

### 3.2 Theme Switching Mechanism

When `window.__moron_setFrame(frameState)` is called with a different theme name:

1. Inline styles on the frame container update automatically (MoronFrame
   re-renders with new cssProperties).
2. For the CSS file baseline, the host page should have a `<style id="moron-theme">`
   element whose content can be replaced.

In practice, since inline styles always override, the CSS baseline swap is
optional for correctness. But swapping it ensures that any element NOT covered
by inline styles (e.g., elements outside the MoronFrame tree, or Tailwind
utility classes resolved against `:root`) also picks up the new theme.

**Design: Theme CSS is injected as a `<style>` block in the host page. When
FrameState arrives with a different theme name, the host page can optionally
replace the style block. Inline styles handle the actual rendering.**

### 3.3 Second Theme for Testing

Create a "moron-light" theme as a minimal counterpart to "moron-dark":

- Rust: Add a `Theme::light()` constructor in `moron-themes/src/defaults.rs`
  with inverted color tokens (light bg, dark fg). Typography, spacing, timing,
  shadows can share values or differ slightly.
- CSS: Add `packages/themes/src/light.css` with the corresponding `:root` block.
- Registry: Add `"light"` entry in `packages/themes/src/index.ts`.

The light theme needs only enough difference to be visually distinguishable:
different background colors, different foreground colors, different accent color.
Non-color tokens (typography, spacing, timing) can remain the same.

### 3.4 Sync Verification

The Rust defaults and CSS defaults can drift. Add a test that:

1. Parses `packages/themes/src/default.css` to extract all `--moron-*` property
   declarations and their values.
2. Calls `Theme::default().to_css_properties()` to get the Rust values.
3. Asserts they match exactly.

This is a Rust integration test that reads the CSS file at test time. It runs
during `cargo test` and catches any drift immediately.

A similar test for `light.css` vs `Theme::light()` ensures the second theme
also stays in sync.

### 3.5 Integration Test: Two Themes, Different Output

The acceptance criteria require: "render same frame with two themes, verify
different output." This needs the rendering pipeline (Chromium) which is gated
behind T-007-04's visual regression framework.

For this ticket, the integration test can be:
- **Rust-level:** Compute FrameState with Theme::default() and Theme::light().
  Assert the JSON output differs (different cssProperties values).
- **React-level:** Render MoronFrame with two different ThemeStates. Assert
  the rendered DOM has different inline style values.

Pixel-level comparison is T-007-04's responsibility.

### 3.6 MoronFrame Adjustments

MoronFrame currently works correctly. No changes are needed to the inline-style
spreading logic. The only consideration:

- The `containerStyle` hardcodes `background: "var(--moron-bg-primary)"` and
  similar references. These resolve against the inline styles spread via
  `...themeStyles`. This is correct and should remain.
- If the host page loads default.css, these var() references would also resolve
  against `:root` as a fallback. This is the desired behavior.

### 3.7 Cascade Order (Specificity)

The effective cascade for any `--moron-*` property:

1. **`:root` from CSS file** (lowest priority) -- baseline defaults.
2. **Inline style on frame container** (highest priority) -- FrameState values.

CSS custom properties follow normal cascade rules. Inline styles always win
over any selector-based rule. A `var(--moron-bg-primary)` reference on a child
element resolves by walking up the DOM: it finds the inline-style value on the
frame container first, shadowing the `:root` value.

If a property is defined in the CSS file but NOT in the inline styles (which
should not happen since Theme always produces all 56), the `:root` value is used.

### 3.8 File Changes Summary

**New files:**
- `packages/themes/src/light.css` -- Light theme CSS custom properties
- `moron-themes/src/light.rs` or extend `defaults.rs` -- Light theme Rust defaults

**Modified files:**
- `packages/themes/src/index.ts` -- Add "light" to themes registry
- Host page (created by T-007-01) -- Import default.css, add theme style element
- `moron-themes/src/defaults.rs` -- Add `Theme::light()` constructor
- `moron-themes/src/lib.rs` -- Possibly add light module or extend tests

**New test files:**
- `moron-themes/tests/css_sync.rs` -- Verify Rust defaults match CSS files
- Test additions in `moron-core/src/frame.rs` -- Two-theme FrameState comparison

### 3.9 Dependency on T-007-01

T-007-01 creates the host page. This ticket cannot wire CSS into a host page
that does not exist yet. The implementation plan must account for this:

- Research and Design (this document) can proceed now.
- Structure, Plan, and Implement must wait for T-007-01 to deliver the host page.
- The Rust-side work (light theme, sync tests) can proceed in parallel.

### 3.10 What This Ticket Does NOT Do

- Does not change MoronFrame's inline-style approach (that works correctly).
- Does not implement Tailwind class usage in components (that is T-007-02's
  concern, and templates decide their own styling approach).
- Does not implement pixel-level visual regression testing (that is T-007-04).
- Does not implement theme animation/transitions (out of scope).
- Does not change the FrameState JSON contract (ThemeState stays the same).

## 4. Risks

1. **CSS file / Rust defaults drift.** Mitigated by sync verification test.
2. **Host page structure unknown.** T-007-01 has not delivered yet. Design
   assumes a standard HTML page with a `<head>` where CSS can be loaded. If
   T-007-01 uses an unusual approach, the CSS loading mechanism may need
   adjustment. Low risk -- any HTML bundler supports CSS imports.
3. **Inline style key format.** CSS custom properties as inline style keys
   require the `--` prefix to work in React's style object. MoronFrame already
   handles this correctly (keys like `"--moron-bg-primary"` are spread directly).
4. **Theme completeness.** If a future theme omits a property, var() references
   fall back to the CSS file baseline. This is by design, but the sync test
   only verifies the default and light themes. Custom themes are the author's
   responsibility.

## 5. Decision Summary

- **Approach:** CSS file as baseline + inline styles as override (Option C).
- **Default CSS loading:** Static import in host page at build time.
- **Theme switching:** Inline styles handle it; CSS baseline swap is optional.
- **Second theme:** "moron-light" with inverted colors, same structural tokens.
- **Sync verification:** Rust integration test parses CSS and compares to defaults.
- **Integration test:** Rust-level FrameState comparison with two themes.
- **No architecture changes** to MoronFrame or the FrameState contract.
