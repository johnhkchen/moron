# T-007-04 Structure: Visual Regression Tests

## Files Created

### `moron-core/tests/visual.rs`
New test file containing:
- Module-level doc comment with running instructions
- Helper functions:
  - `baselines_dir() -> PathBuf` -- returns path to baselines directory
  - `baseline_path(name: &str) -> PathBuf` -- returns path to a specific baseline
  - `compare_png_bytes(actual: &[u8], baseline: &[u8], tolerance: f64) -> bool` -- byte-level comparison
  - `check_or_create_baseline(name: &str, actual: &[u8], tolerance: f64)` -- core comparison logic
  - `launch_bridge() -> ChromiumBridge` -- create bridge from MORON_HTML_PATH env var
  - `render_frame_state(bridge: &ChromiumBridge, state: &FrameState) -> Vec<u8>` -- serialize + capture
- Test functions (all `#[test] #[ignore]`):
  - `visual_title_card_dark()` -- title element, dark theme
  - `visual_metric_display()` -- metric element with direction=up
  - `visual_steps_list()` -- steps with 3 items
  - `visual_theme_switching()` -- dark vs light comparison

### `moron-core/tests/baselines/` (directory)
- Created by `mkdir -p` or by tests on first run
- Will contain: `title_card_dark.png`, `metric_up.png`, `steps_list.png`, `theme_dark.png`, `theme_light.png`
- `.gitkeep` file to ensure directory exists in git

## Files Modified

### `docs/active/tickets/T-007-04.md`
- Update frontmatter: `status: done`, `phase: done`

## Module Boundaries

- `visual.rs` is a standalone integration test (in `tests/` directory)
- Imports from `moron_core::prelude::*` for facade, frame state, chromium bridge
- Imports from `moron_themes::Theme` for theme switching
- No new library code needed -- all functionality exists in moron-core already

## Public Interface

None -- this is a test file only. No library code changes.
