# T-007-03 Progress: Theme CSS Integration

## Completed Steps

### Step 1: Created light.css -- DONE
- Created `packages/themes/src/light.css` with all 56 `--moron-*` CSS custom
  properties on `:root`.
- Color tokens inverted: white/light-gray backgrounds, dark foreground text.
- Accent color shifted from `#3b82f6` to `#2563eb`.
- Semantic colors adjusted to darker variants for light backgrounds.
- Shadow opacity significantly reduced (0.05/0.07/0.10 vs 0.3/0.3/0.4).
- Typography, spacing, and timing tokens identical to dark theme.

### Step 2: Added Theme::light() in Rust -- DONE
- Added `impl Theme { pub fn light() -> Self }` in `moron-themes/src/defaults.rs`.
- All values match `packages/themes/src/light.css` exactly (verified by sync test).
- Non-color tokens reuse `Default` impls for typography, spacing, and timing.
- Color and shadow tokens are explicit, matching the light.css values.

### Step 3: Added light theme unit tests -- DONE
- `light_theme_has_correct_name` -- name is "moron-light"
- `light_theme_has_non_empty_values` -- all fields populated
- `light_theme_differs_from_default` -- colors and shadows differ
- `light_theme_to_css_properties_count` -- exactly 56 properties
- `light_theme_serde_round_trip` -- serialize/deserialize preserves equality
- All 5 tests pass.

### Step 4: Created CSS sync integration test -- DONE
- Created `moron-themes/tests/css_sync.rs` with a minimal CSS property parser.
- Path: `CARGO_MANIFEST_DIR/../packages/themes/src/{filename}`.
- Tests:
  - `default_css_matches_rust_defaults` -- keys and values match
  - `light_css_matches_rust_light` -- keys and values match
  - `css_files_have_56_properties` -- both files define exactly 56 properties
  - `both_css_files_define_same_property_keys` -- same key set
  - `parser_handles_css_comments_and_blank_lines` -- parser robustness
- Special handling: `--moron-container-padding` uses `var(--moron-space-12)` in
  CSS but `"3rem"` in Rust. Skipped in value comparison (documented in code).
- All 5 tests pass.

### Step 5: Added two-theme FrameState integration test -- DONE
- Added `two_themes_produce_different_frame_state` in `moron-core/src/frame.rs`.
- Creates M with default theme and M with light theme.
- Asserts: different theme names, different bg-primary, different fg-primary,
  both have 56 properties, ThemeState structs differ, JSON output differs.
- Test passes.

### Step 6: Registered light theme in TypeScript -- DONE
- Added `light` entry to `packages/themes/src/index.ts` themes registry.
- Uses same `MoronTheme` shape with `new URL("./light.css", import.meta.url).href`.

### Step 7: Verified host page CSS integration -- DONE
- `packages/ui/build.mjs` already reads `packages/themes/src/default.css` and
  inlines it into the `<style>` block of `dist/index.html`.
- The default theme CSS is loaded as a baseline in the host page.
- MoronFrame's inline styles override `:root` values due to CSS specificity.
- No changes were needed -- T-007-01 already completed this wiring.

### Step 8: All checks pass -- DONE
- `cargo check` -- workspace compiles cleanly
- `cargo test` -- 168 tests pass (12 ignored for environment-specific reasons)
- `cargo clippy` -- no warnings
- `npm run typecheck` (packages/themes) -- passes
- `npm run typecheck` (packages/ui) -- passes

## Deviations from Plan

### Path resolution fix
The initial path in `css_sync.rs` used `../../packages/themes/src/` relative to
`CARGO_MANIFEST_DIR`. Since `CARGO_MANIFEST_DIR` for `moron-themes` is the
`moron-themes/` directory (one level below workspace root), the correct
relative path is `../packages/themes/src/`. Fixed during implementation.

No other deviations from the plan.

## Summary

All acceptance criteria met:
1. Default theme CSS loaded in host page (via build.mjs inlining)
2. Theme properties from FrameState override defaults correctly (CSS specificity)
3. "moron-light" theme exists with different colors
4. Switching themes produces different FrameState JSON (test verified)
5. Rust test verifies CSS and Rust defaults don't drift (sync tests)
6. `cargo check`, `cargo test`, `npm run typecheck` all pass
