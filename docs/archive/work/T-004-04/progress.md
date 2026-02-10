# T-004-04 Progress: Frame Rendering Loop

## Status: Complete

All steps from the plan have been executed and verified.

## Completed Steps

### Step 1: Define RenderError
- Defined `RenderError` enum with 4 variants: `Bridge`, `OutputDir`, `Serialize`, `WriteFrame`
- Each variant captures contextual information (frame number, path, underlying error)
- `Bridge` variant uses `#[from] BridgeError` for ergonomic `?` usage
- Verified: `cargo check` passes

### Step 2: Define RenderProgress and RenderResult
- `RenderProgress { current_frame: u32, total_frames: u32 }`
- `RenderResult { total_frames: u32, output_dir: PathBuf }`
- Both are simple data structs with public fields
- Verified: `cargo check` passes

### Step 3: Define RenderConfig
- `RenderConfig { output_dir, bridge_config, progress }`
- `progress: Option<Box<dyn Fn(RenderProgress)>>` for optional callback
- Constructor `RenderConfig::new(output_dir, bridge_config)` with `progress: None` default
- Verified: `cargo check` passes

### Step 4: Implement frame_path helper
- `fn frame_path(output_dir: &Path, frame_num: u32) -> PathBuf`
- Uses `format!("frame_{:06}.png", frame_num)` for 6-digit zero-padding
- Unit tests added and passing
- Verified: `cargo test` passes

### Step 5: Implement the render function
- `pub async fn render(m: &M, config: RenderConfig) -> Result<RenderResult, RenderError>`
- Empty timeline early return (0 frames, no bridge launch, no directory creation)
- Creates output directory with `std::fs::create_dir_all`
- Launches ChromiumBridge from config
- Inner `render_frames` loop separated for clean bridge cleanup
- For each frame: compute_frame_state -> serde_json::to_string -> bridge.capture_frame -> fs::write
- Progress reported via callback or eprintln fallback
- Bridge closed in both success and error paths
- Verified: `cargo check` passes

### Step 6: Update lib.rs re-exports
- Added `pub use renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};`
- Added same to prelude module
- Verified: `cargo check` passes

### Step 7: Add unit tests
- 12 unit tests added:
  - `frame_path_formatting` -- zero-padded path generation
  - `frame_path_with_nested_dir` -- nested directory handling
  - `render_config_new` -- constructor defaults
  - `render_config_with_progress_callback` -- callback invocation
  - `render_progress_fields` -- struct field access
  - `render_result_fields` -- struct field access
  - `render_error_display_messages` -- error formatting
  - `empty_timeline_has_zero_frames` -- empty M produces 0 frames
  - `non_empty_timeline_frame_count` -- 1s at 30fps = 30 frames
  - `time_computation_matches_frame_at` -- time->frame consistency
  - `report_progress_with_callback` -- callback invoked correctly
  - `report_progress_without_callback` -- stderr fallback works
- Verified: all 12 tests pass

### Step 8: Update module doc comment
- Replaced stale "Bevy + wgpu + vello" doc comment with accurate description
  of the Chromium bridge rendering pipeline
- Verified: `cargo check` passes

### Step 9: Final verification
- `cargo check` passes
- `cargo test` passes (48 unit + 8 integration tests, all green)
- `cargo clippy` passes (no warnings)

### Step 10: Update ticket frontmatter
- Set `status: done`, `phase: done` in T-004-04.md

## Deviations from Plan

None. The plan was followed exactly as specified.

## Files Modified

1. `moron-core/src/renderer.rs` -- replaced stub with full implementation (~420 lines)
2. `moron-core/src/lib.rs` -- added re-exports for renderer types

## Files Created

1. `docs/active/work/T-004-04/research.md`
2. `docs/active/work/T-004-04/design.md`
3. `docs/active/work/T-004-04/structure.md`
4. `docs/active/work/T-004-04/plan.md`
5. `docs/active/work/T-004-04/progress.md`

## Test Results

```
running 48 tests  (moron-core unit tests)
running 5 tests   (moron-core integration tests)
running 14 tests  (moron-techniques unit tests)
running 5 tests   (moron-themes unit tests)
running 5 tests   (moron-voice unit tests)
running 3 tests   (moron-techniques composition tests)

All 80 tests passed. 0 failures.
```
