# T-005-03 Progress: moron-build-cli

## Completed Steps

### Step 1: Create moron-core/src/demo.rs
- Created `DemoScene` struct implementing `Scene` trait
- Uses existing techniques: `FadeIn`, `FadeUp`
- Uses existing facade methods: `title`, `narrate`, `play`, `beat`, `section`, `breath`, `show`
- Added 4 unit tests validating scene builds, duration, frame count, segment count

### Step 2: Create moron-core/src/build.rs
- Created `BuildError` enum wrapping `RenderError`, `FfmpegError`, `io::Error`, `Config`
- Created `BuildProgress` enum with phases: `SceneBuilt`, `RenderingFrame`, `Encoding`, `MuxingAudio`, `Complete`
- Created `BuildConfig` struct with `output_path`, `html_path`, `width`, `height`, `keep_frames`, `progress`
- Created `BuildResult` struct with `output_path`, `total_frames`, `duration`
- Implemented `build_video()` async function with full pipeline:
  1. Report scene stats
  2. Create temp dir
  3. Render frames via Chromium bridge
  4. Encode video-only .mp4 via FFmpeg
  5. Assemble audio track from timeline
  6. Mux audio + video into final .mp4
  7. Clean up temp dir (unless keep_frames)
  8. Report completion
- Progress callback uses `Arc<dyn Fn + Send + Sync>` for sharing between render forwarder and pipeline phases
- Added 6 unit tests for config defaults, error display, error conversions, result fields, report callback

### Step 3: Update moron-core/src/lib.rs
- Added `pub mod build;` and `pub mod demo;` declarations
- Added re-exports: `build_video`, `BuildConfig`, `BuildError`, `BuildProgress`, `BuildResult`, `DemoScene`
- Added same re-exports to prelude module

### Step 4: Update moron-cli/src/main.rs
- Changed `fn main()` to `#[tokio::main] async fn main()`
- Extended Build command with CLI args: `--output/-o`, `--html-path`, `--width`, `--height`, `--keep-frames`
- Implemented `run_build()` async function:
  - Resolves html_path (explicit flag or convention fallback)
  - Builds DemoScene via M facade
  - Creates progress callback with phased eprintln reporting
  - Calls `build_video()` and handles result/error
- Implemented `resolve_html_path()` with explicit/convention priority
- Implemented `format_build_error()` for user-friendly error messages
  - Chrome not found -> install instructions
  - FFmpeg not found -> install instructions with download URL
  - I/O and config errors -> clear messages

### Step 5: Verification
- `cargo check` passes (no errors)
- `cargo test` passes (130 tests, 0 failures)
- `cargo clippy` passes (0 warnings)

## Deviations from Plan

### Progress callback type changed to Arc
- **Plan:** `Option<Box<dyn Fn(BuildProgress)>>`
- **Actual:** `Option<Arc<dyn Fn(BuildProgress) + Send + Sync>>`
- **Reason:** The render progress forwarder closure needs to reference the build progress callback. Using `Box` creates a lifetime issue since the closure must be `'static` for `RenderConfig`. `Arc` with `Send + Sync` bounds allows safe sharing without lifetime constraints.

### No deviation in pipeline flow
The pipeline flow matches the plan exactly: render -> encode -> assemble audio -> mux -> cleanup.

## Files Changed

| File | Action | Lines |
|------|--------|-------|
| moron-core/src/demo.rs | Created | 63 |
| moron-core/src/build.rs | Created | 258 |
| moron-core/src/lib.rs | Modified | 44 (was 37) |
| moron-cli/src/main.rs | Modified | 235 (was 55) |
