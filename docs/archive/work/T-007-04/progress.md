# T-007-04 Progress: Visual Regression Tests

## Completed

### Step 1: Created baselines directory
- `moron-core/tests/baselines/` with `.gitkeep`

### Step 2: Wrote visual.rs test file
- Module doc with running instructions
- Helper functions: baselines_dir, baseline_path, compare_png_bytes, check_or_create_baseline
- Bridge helpers: bridge_config_from_env, capture_frame_png
- 4 visual regression tests (all #[ignore]):
  - visual_title_card_dark -- title element, dark theme
  - visual_metric_display -- metric with direction=up
  - visual_steps_list -- steps with 3 items
  - visual_theme_switching -- dark vs light, verifies they differ
- 8 non-ignored unit tests for comparison helpers (comparison_tests module)

### Step 3: Updated ticket frontmatter
- status: done, phase: done

### Step 4: Verification
- `cargo check` passes
- `cargo test` passes: 8 new non-ignored tests pass, 4 visual tests correctly ignored
- All 106 existing moron-core unit tests pass
- All e2e, integration, and other crate tests unaffected

## Deviations from Plan
None. Implementation followed the plan exactly.

## Files Created
- `moron-core/tests/visual.rs` -- visual regression test suite
- `moron-core/tests/baselines/.gitkeep` -- baseline directory placeholder
- `docs/active/work/T-007-04/research.md`
- `docs/active/work/T-007-04/design.md`
- `docs/active/work/T-007-04/structure.md`
- `docs/active/work/T-007-04/plan.md`
- `docs/active/work/T-007-04/progress.md`

## Files Modified
- `docs/active/tickets/T-007-04.md` -- status: done, phase: done
