# T-004-04 Research: Frame Rendering Loop

## Objective

Implement the frame rendering loop in `moron-core/src/renderer.rs`. Given a built scene
(an `M` with a recorded timeline), the renderer iterates through the timeline at the target
FPS, renders each frame via the Chromium bridge, and saves numbered PNG files to disk.

## Current State

### Stub File

`moron-core/src/renderer.rs` contains a single doc comment:

```rust
//! Rendering pipeline: Bevy + wgpu + vello integration.
```

The module is declared as `pub mod renderer` in `moron-core/src/lib.rs`. It compiles as an
empty module. No types or functions are re-exported from it.

### Module Doc Comment

The existing doc comment references "Bevy + wgpu + vello" which reflects an earlier design
direction. The actual rendering approach uses headless Chromium + React (via ChromiumBridge).
The doc comment should be updated to reflect the current architecture.

## Upstream Dependencies

### ChromiumBridge (T-004-03) -- Primary Dependency

Located in `moron-core/src/chromium.rs`. Provides the full frame capture pipeline:

```rust
pub struct ChromiumBridge { ... }

impl ChromiumBridge {
    pub async fn launch(config: BridgeConfig) -> Result<Self, BridgeError>;
    pub async fn capture_frame(&self, frame_json: &str) -> Result<Vec<u8>, BridgeError>;
    pub async fn close(mut self) -> Result<(), BridgeError>;
}
```

Key observations:
- `launch` takes a `BridgeConfig` (width, height, html_path, chrome_executable, headless,
  launch_timeout). Returns `Result<Self, BridgeError>`.
- `capture_frame` takes a JSON string (`&str`) and returns PNG bytes (`Vec<u8>`).
- `close` takes `self` by value (consuming the bridge).
- All methods are async.
- `BridgeError` is a `thiserror` enum with variants: LaunchFailed, ChromeNotFound,
  PageLoadFailed, JsEvalFailed, ScreenshotFailed, RenderTimeout, AlreadyClosed.

### BridgeConfig

```rust
pub struct BridgeConfig {
    pub width: u32,                          // default 1920
    pub height: u32,                         // default 1080
    pub html_path: PathBuf,                  // required
    pub chrome_executable: Option<PathBuf>,  // default None (auto-detect)
    pub headless: bool,                      // default true
    pub launch_timeout: Duration,            // default 20s
}
```

Constructor: `BridgeConfig::new(html_path)` sets sensible defaults.

### compute_frame_state (T-004-01)

Located in `moron-core/src/frame.rs`:

```rust
pub fn compute_frame_state(m: &M, time: f64) -> FrameState;
```

Takes an `M` reference and a timestamp. Returns a `FrameState` containing:
- `time`, `frame`, `total_duration`, `fps`
- `elements: Vec<ElementState>`
- `active_narration: Option<String>`
- `theme: ThemeState`

`FrameState` derives `Serialize` so it can be serialized to JSON via `serde_json::to_string`.

### Timeline

Located in `moron-core/src/timeline.rs`:

```rust
impl Timeline {
    pub fn fps(&self) -> u32;
    pub fn total_duration(&self) -> f64;
    pub fn total_frames(&self) -> u32;     // ceil(duration * fps)
    pub fn frame_at(&self, time: f64) -> u32;
}
```

`total_frames()` returns `(duration * fps).ceil() as u32`, or 0 for empty timelines.
Empty timelines (no segments) have `total_duration() == 0.0` and `total_frames() == 0`.

### M (Facade)

Located in `moron-core/src/facade.rs`:

```rust
impl M {
    pub fn timeline(&self) -> &Timeline;
    pub fn current_theme(&self) -> &Theme;
    // ... element accessors are pub(crate)
}
```

The renderer needs `m.timeline()` to get FPS, total_frames, total_duration.
The renderer calls `compute_frame_state(&m, time)` which accesses M internals.

## Available Dependencies in moron-core

From `Cargo.toml`:
- `serde` + `serde_json` -- for serializing FrameState to JSON
- `tokio` (full features) -- async runtime, filesystem operations
- `anyhow` -- error handling
- `thiserror` -- custom error types
- `chromiumoxide` -- headless Chrome (used via ChromiumBridge)

All needed dependencies are already available. No new dependencies required.

### Filesystem Operations

`tokio::fs` provides async filesystem operations:
- `tokio::fs::create_dir_all(path)` -- create output directory
- `tokio::fs::write(path, bytes)` -- write PNG bytes to file

Standard `std::fs` is also available for synchronous operations if preferred.

## Re-export Considerations

`moron-core/src/lib.rs` currently re-exports key types. The renderer module should
export its public types, which lib.rs can then re-export. Currently the lib.rs does
not re-export anything from `renderer` since it's a stub.

## Error Handling Patterns

The codebase uses two patterns:
1. `thiserror` for specific error enums (e.g., `BridgeError` in chromium.rs)
2. `anyhow::Error` for general error propagation

The renderer should define its own `RenderError` enum via `thiserror` since the caller
needs to distinguish between different failure modes (bridge errors, I/O errors,
serialization errors).

## Progress Reporting Patterns

The codebase has no existing progress reporting mechanism. The ticket requires "progress
reporting (frame count, optional callback)." Options:
- Simple `println!` to stderr
- Callback function/closure
- Both (callback with println as fallback)

## File Naming Convention

The ticket specifies: `frame_000000.png`, `frame_000001.png`, etc.
This uses 6-digit zero-padded frame numbers, which supports up to 999,999 frames
(~9.2 hours at 30fps). Adequate for explainer videos.

## Downstream Consumer

### T-005-01 (FFmpeg Encoding)

The FFmpeg pipeline (S-005) will consume the PNG frame sequence produced by the renderer.
FFmpeg accepts numbered image sequences via `ffmpeg -i frame_%06d.png`. The 6-digit
zero-padded naming convention matches FFmpeg's `%06d` pattern specifier.

The renderer output directory becomes the FFmpeg input directory.

## Testing Considerations

The renderer's core logic is async and depends on ChromiumBridge, which requires a real
Chrome instance. Unit testing the render loop without Chrome requires either:
1. Mocking the bridge (not trait-based, so would need a test double approach)
2. Testing only the non-bridge parts (frame count computation, path generation, config)
3. Integration tests that require Chrome installed

For this ticket, we should test:
- RenderConfig construction and defaults
- Frame count computation from timeline
- Output path formatting
- Empty timeline handling (0 frames = no output, no bridge launch)
- The render function signature compiles

Full integration tests with Chrome belong in a separate test suite.
