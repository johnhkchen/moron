# T-007-03 Plan: Theme CSS Integration

## Step 1: Create light theme CSS file

**File:** `packages/themes/src/light.css`

Create the moron-light theme CSS file with all 56 `--moron-*` custom properties
on `:root`. Color tokens are inverted from the dark theme:
- Backgrounds: light grays/whites (#ffffff, #f8fafc, #e2e8f0)
- Foregrounds: dark grays (#0f172a, #334155, #64748b)
- Accent: different hue to be visually distinct (#2563eb, #3b82f6)
- Semantic colors: same or slightly adjusted
- Shadows: lower opacity (0.1, 0.15, 0.2 instead of 0.3, 0.3, 0.4)

Typography, spacing, timing tokens remain identical to the dark theme.

**Verification:** File exists, has 56 properties, well-formed CSS.

## Step 2: Add Theme::light() in Rust

**File:** `moron-themes/src/defaults.rs`

Add `impl Theme { pub fn light() -> Self }` method. Values must match
`packages/themes/src/light.css` exactly. Color tokens are the light theme
counterparts. Non-color tokens call the same Default impls.

The method constructs `Theme` directly (not via Default + overrides) so every
value is explicit and auditable.

**Verification:** `cargo check` passes.

## Step 3: Add light theme unit tests

**File:** `moron-themes/src/lib.rs`

Add tests:
- `light_theme_has_non_empty_values` — mirrors existing default theme test
- `light_theme_has_correct_name` — asserts name is "moron-light"
- `light_theme_differs_from_default` — at least colors differ
- `light_theme_to_css_properties_count` — produces 56 properties

**Verification:** `cargo test -p moron-themes` passes.

## Step 4: Create CSS sync integration test

**File:** `moron-themes/tests/css_sync.rs`

Write a minimal CSS property parser:
- Read file contents with `std::fs::read_to_string`
- Path: `concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/themes/src/default.css")`
- Find lines matching `--moron-<name>: <value>;` pattern
- Trim whitespace, extract key-value pairs into a HashMap

Test functions:
- `default_css_matches_rust_defaults` — parse default.css, compare to
  `Theme::default().to_css_properties()`. Assert same keys and same values.
  Special handling for `--moron-container-padding` which uses `var()` reference
  in CSS but resolved value in Rust.
- `light_css_matches_rust_light` — same comparison for light.css vs
  `Theme::light().to_css_properties()`.
- `css_files_have_same_property_count` — both CSS files define exactly 56 props.

**Verification:** `cargo test -p moron-themes` passes.

## Step 5: Add two-theme FrameState integration test

**File:** `moron-core/src/frame.rs` (in existing `tests` module)

Add test `two_themes_produce_different_frame_state`:
- Create `M::new()` (default theme), compute FrameState at t=0.
- Create another `M::new()`, call `m.theme(Theme::light())`, compute FrameState.
- Assert theme names differ ("moron-dark" vs "moron-light").
- Assert at least one CSS property value differs (e.g., `--moron-bg-primary`).
- Assert both have 56 CSS properties.

**Verification:** `cargo test -p moron-core` passes.

## Step 6: Register light theme in TypeScript

**File:** `packages/themes/src/index.ts`

Add `light` entry to the `themes` record:
```typescript
light: {
  name: "Light",
  stylesheet: new URL("./light.css", import.meta.url).href,
},
```

**Verification:** `npm run typecheck` in `packages/themes/` passes.

## Step 7: Verify host page CSS integration

No code changes needed. Verify by reading `packages/ui/build.mjs`:
- Lines 46-49 read `default.css` and inline it into the HTML `<style>` block.
- This means the default theme CSS is already loaded as a baseline.
- FrameState inline styles (from MoronFrame) override these `:root` values.

Document this verification in progress.md.

## Step 8: Run all checks

Run:
- `cargo check` — full workspace compiles
- `cargo test` — all tests pass (including new sync and integration tests)
- `cargo clippy` — no warnings
- `npm run typecheck` in `packages/themes/` — TypeScript compiles
- `npm run typecheck` in `packages/ui/` — TypeScript compiles

## Testing Strategy

| Test | Location | What it verifies |
|------|----------|-----------------|
| Light theme non-empty values | moron-themes unit | Theme::light() is complete |
| Light theme name | moron-themes unit | Name is "moron-light" |
| Light differs from default | moron-themes unit | Color tokens are different |
| CSS properties count | moron-themes unit | 56 properties produced |
| Default CSS sync | moron-themes integration | default.css == Rust defaults |
| Light CSS sync | moron-themes integration | light.css == Rust light |
| CSS file property count | moron-themes integration | Both files have 56 props |
| Two-theme FrameState | moron-core unit | Different themes -> different JSON |

## Rollback

All changes are additive. No existing code is modified in a breaking way.
Removing the light theme and tests returns to the exact prior state.

## Notes on container-padding

`default.css` uses `--moron-container-padding: var(--moron-space-12);` (a CSS
variable reference), while Rust uses the resolved value `"3rem"`. The sync test
must handle this: either skip this property, or resolve the reference during
comparison. Choosing to skip this single property in the sync test and document
why, since the CSS variable reference is intentional (allows spacing scale
changes to cascade) while Rust uses the resolved value (no variable resolution
in Rust).
