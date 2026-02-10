# T-007-03 Structure: Theme CSS Integration

## 1. File Inventory

### New Files

**`packages/themes/src/light.css`**
Light theme CSS custom properties. Same structure as default.css (`:root` block with
56 `--moron-*` properties). Inverted color palette: light backgrounds, dark foreground
text, adjusted accent. Typography, spacing, timing, and shadows share structure but
shadows use lighter opacity values appropriate for a light background.

**`moron-themes/tests/css_sync.rs`**
Integration test file. Reads CSS files from `packages/themes/src/` at test time,
parses `--moron-*` property declarations, compares against Rust `Theme::default()`
and `Theme::light()` outputs. Catches drift between the two layers. Lives in
the `tests/` directory (integration test, not unit test) so it can access the
file system relative to the workspace root.

### Modified Files

**`moron-themes/src/defaults.rs`**
Add `Theme::light()` constructor returning a "moron-light" theme. Color tokens
are inverted (light bg, dark fg). Non-color tokens (typography, spacing, timing)
remain identical to the dark theme to keep the diff focused. Shadow tokens use
lower opacity since shadows on light backgrounds need less intensity.

**`moron-themes/src/lib.rs`**
No structural changes needed. The existing `pub use` re-exports cover `Theme`.
`Theme::light()` is an inherent method, not a trait impl, so it is available
through the existing `pub use theme::Theme` re-export. Add a unit test for the
light theme (non-empty values, property count, name check).

**`packages/themes/src/index.ts`**
Add "light" entry to the `themes` registry. Same shape as the "default" entry:
`{ name: "Light", stylesheet: new URL("./light.css", import.meta.url).href }`.

**`moron-core/src/frame.rs`** (tests only)
Add a test that creates two `M` instances (one with default theme, one with
`Theme::light()`), computes FrameState for each, and asserts that the theme
names differ and at least one CSS property value differs. This is the "two
themes produce different FrameState JSON" acceptance criterion.

### Unchanged Files (verified correct)

**`packages/ui/build.mjs`**
Already reads `packages/themes/src/default.css` and inlines it into the host
page HTML. The default theme CSS baseline is already wired. No changes needed.

**`packages/ui/src/host.tsx`**
The host page renders templates via the registry. Theme CSS is loaded via
build.mjs's HTML template, not via a React import. No changes needed.

**`packages/ui/src/MoronFrame.tsx`**
Inline style spreading of `state.theme.cssProperties` is correct. CSS custom
properties set as inline styles override `:root` values from the CSS file.
No changes needed.

## 2. Module Boundaries

### Rust side

```
moron-themes/
  src/
    theme.rs       -- Theme struct, to_css_properties() [unchanged]
    defaults.rs    -- Default for Theme + Theme::light() [add light()]
    lib.rs         -- Re-exports, unit tests [add light theme test]
  tests/
    css_sync.rs    -- Integration test: parse CSS, compare to Rust [NEW]
```

`Theme::light()` is an inherent method on `Theme`, not a new module. It returns
a fully populated `Theme` with all 56 properties. The method lives in
`defaults.rs` alongside the `Default` impl because both define concrete theme
values.

### TypeScript side

```
packages/themes/
  src/
    default.css    -- Dark theme CSS [unchanged]
    light.css      -- Light theme CSS [NEW]
    index.ts       -- Theme registry [add light entry]
```

No new TypeScript types or interfaces. The existing `MoronTheme` interface and
`themes` registry handle the light theme identically to the default.

### Integration test reads CSS files

The Rust integration test (`css_sync.rs`) reads CSS files using
`std::fs::read_to_string()` with paths relative to `CARGO_MANIFEST_DIR`.
The path to `packages/themes/src/` is `../../packages/themes/src/` from the
`moron-themes` crate root.

The CSS parser is minimal: scan for lines matching `--moron-*: *;` within a
`:root { }` block. No external CSS parsing dependency needed. A simple regex
or line-by-line scan suffices for the well-structured CSS files we control.

## 3. Public Interface Changes

### Rust

`Theme::light() -> Theme` — new public constructor. Returns a complete theme
with name "moron-light". All 56 properties populated.

No changes to `Theme::to_css_properties()`, `ThemeState`, `FrameState`, or
any existing public API.

### TypeScript

`themes` map in `packages/themes/src/index.ts` gains a `"light"` key.
The `ThemeName` type automatically includes it via `keyof typeof themes`.

## 4. Dependency Graph

```
moron-themes/tests/css_sync.rs
  reads -> packages/themes/src/default.css
  reads -> packages/themes/src/light.css
  calls -> moron_themes::Theme::default()
  calls -> moron_themes::Theme::light()

moron-core/src/frame.rs (test)
  calls -> M::new() [uses Theme::default()]
  calls -> M::theme(Theme::light())
  calls -> compute_frame_state()

packages/themes/src/index.ts
  references -> packages/themes/src/light.css (URL)

packages/ui/build.mjs
  reads -> packages/themes/src/default.css [already done]
```

No new crate dependencies. No new npm packages. The CSS sync test uses only
`std::fs` and string parsing from the standard library.

## 5. Ordering Constraints

1. `light.css` must exist before `css_sync.rs` can test it.
2. `Theme::light()` must exist before `css_sync.rs` and `frame.rs` tests use it.
3. `light.css` must exist before `index.ts` references it.
4. All of the above are independent of the host page (build.mjs already works).

Practical order: create light theme values (Rust + CSS) first, then tests, then
registry update. But all can be done in a single implementation pass since
there are no build-time code generation steps.
