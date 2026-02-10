# T-005-03 Research: moron-build-cli

## Objective

Wire the full rendering pipeline into the `moron build` CLI command. The command
takes a scene, builds it through the M facade, renders frames via the Chromium
bridge, encodes via FFmpeg, and outputs a playable .mp4 file.

## Current State of the CLI

### moron-cli/src/main.rs

The CLI is a `clap`-based binary with four subcommands: Build, Preview, Init, Gallery.
All four are stubs that print "not yet implemented". The `main` function is synchronous
(`fn main() -> anyhow::Result<()>`), not async.

```rust
Commands::Build { path } => {
    println!("moron build: not yet implemented (path: {path})");
}
```

The Build command accepts a single `path` argument (defaults to `"."`). There is no
`--output` / `-o` flag, no resolution flags, no FPS override, no verbose/quiet toggle.

### moron-cli/Cargo.toml

Dependencies: `moron-core` (workspace), `clap`, `tokio`, `anyhow`. The `tokio`
dependency is present but unused -- main is not `#[tokio::main]`. No direct
dependency on `moron-techniques`, `moron-voice`, or `moron-themes` (all accessed
transitively through `moron-core`).

## Pipeline Components (Upstream)

### M Facade (moron-core/src/facade.rs)

The `M` struct is the scene authoring entry point. Key characteristics:

- `M::new()` creates an instance with default theme (moron-dark) and voice (Kokoro).
- Scene trait: `fn build(m: &mut M)` -- scenes are built synchronously.
- `m.timeline()` returns `&Timeline` with `fps()`, `total_duration()`, `total_frames()`.
- Elements created via `m.title()`, `m.show()`, `m.section()`, `m.metric()`, `m.steps()`.
- Timing via `m.narrate()`, `m.beat()`, `m.breath()`, `m.wait()`, `m.play()`.
- All operations are synchronous -- no async in the facade.

### Scene Trait

```rust
pub trait Scene {
    fn build(m: &mut M);
}
```

Scenes are currently Rust structs compiled into the binary. The `examples/hello_world.rs`
uses `moron::prelude::*` and references techniques like `FadeIn`, `Stagger`, `CountUp`
that do not yet exist as concrete types (only `FadeIn`, `FadeUp`, `FadeOut` exist in
moron-techniques). The demo scene for T-005-03 must use only what exists.

### Frame Rendering (moron-core/src/renderer.rs)

Fully implemented. The `render` function is async:

```rust
pub async fn render(m: &M, config: RenderConfig) -> Result<RenderResult, RenderError>;
```

- Takes `&M` (built scene) and `RenderConfig` (output_dir, bridge_config, progress callback).
- Returns `RenderResult { total_frames, output_dir }`.
- Manages ChromiumBridge lifecycle: launches before rendering, closes after (even on error).
- Writes `frame_NNNNNN.png` files to output_dir.
- Reports progress via optional callback or eprintln fallback.
- Handles empty timelines: returns immediately with 0 frames, no Chrome launched.

### RenderConfig

```rust
pub struct RenderConfig {
    pub output_dir: PathBuf,
    pub bridge_config: BridgeConfig,
    pub progress: Option<Box<dyn Fn(RenderProgress)>>,
}
```

### BridgeConfig (moron-core/src/chromium.rs)

```rust
pub struct BridgeConfig {
    pub width: u32,              // 1920
    pub height: u32,             // 1080
    pub html_path: PathBuf,      // required: built React app index.html
    pub chrome_executable: Option<PathBuf>,
    pub headless: bool,          // true
    pub launch_timeout: Duration, // 20s
}
```

The `html_path` must point to a built React app that exposes `window.__moron_setFrame`.
This is `packages/ui`'s built output. The CLI needs to know where this HTML file lives.

### FFmpeg Encoding (moron-core/src/ffmpeg.rs)

Currently a **stub** -- single doc comment line. T-005-01 will replace this with:
- FFmpeg detection on PATH
- Directory-of-PNGs input mode
- H.264 encoding at configurable FPS, resolution
- Output to specified .mp4 path
- Error type for missing FFmpeg, encoding failures

### Audio Track Assembly (T-005-02)

T-005-02 will extend ffmpeg.rs with audio muxing:
- Walk timeline segments to generate silence-duration audio track
- FFmpeg muxes audio + video into single .mp4
- Both video and audio streams in output

## Re-exports (moron-core/src/lib.rs)

All key types are re-exported at the crate root:

```rust
pub use facade::{Direction, Element, M, Scene, BEAT_DURATION, BREATH_DURATION};
pub use renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
pub use timeline::{Segment, Timeline, TimelineBuilder};
```

The prelude includes everything a scene author needs. The CLI can use `moron_core::*`
or `moron_core::prelude::*`.

## Available Techniques

From moron-techniques: `FadeIn`, `FadeUp`, `FadeOut` with configurable duration.
The `Technique` trait provides `name()` and `duration()`. `TechniqueExt` provides
`.with_ease()` and `.with_duration()`. Easing: `Ease::Linear`, `OutCubic`, `OutBack`,
`InOutCubic`.

## What the Build Command Needs

### 1. Scene Construction

For the initial version, a built-in demo scene compiled into the binary. The scene
uses only existing API: `M::new()`, facade methods, available techniques (FadeIn,
FadeUp, FadeOut). No dynamic loading of .rs scene files.

### 2. Frame Rendering

Call `moron_core::render(m, config).await`. Requires:
- An output directory for PNG frames (temporary)
- A BridgeConfig pointing to the React app's index.html
- A progress callback (or use eprintln default)

### 3. FFmpeg Encoding

After rendering completes, call the FFmpeg module (T-005-01 API, not yet defined) to
encode the PNG directory into .mp4. Will need:
- Input: directory of PNGs, frame pattern `frame_%06d.png`
- FPS from timeline
- Resolution from BridgeConfig
- Output path for .mp4

### 4. Audio Muxing

After video encoding, mux the audio track (T-005-02). Or this may be combined with
video encoding in a single FFmpeg pass.

### 5. Temp Directory Management

Frame PNGs are intermediate artifacts. The CLI needs a temp directory for frames,
cleaned up after successful encoding. Options: `std::env::temp_dir()`, `tempfile` crate,
or a `.moron-build/` directory in the project.

### 6. React App Location

The BridgeConfig requires `html_path` pointing to the built React UI. The CLI must
locate `packages/ui/dist/index.html` (or equivalent). For initial version, this could
be a hardcoded path relative to the binary, or a CLI flag.

### 7. Async Runtime

The render function is async. The CLI's `main` must become `#[tokio::main]` (tokio
is already a dependency).

### 8. Progress Reporting

The ticket requires progress reporting: "rendering frame N/M, encoding, done."
The renderer already has progress for frames. The CLI adds encoding progress and
overall pipeline progress.

### 9. Error Handling

The ticket requires graceful handling of:
- Missing Chrome/Chromium (BridgeError::ChromeNotFound / LaunchFailed)
- Missing FFmpeg (from T-005-01's error type)
- General I/O errors, permission errors

Errors should produce user-friendly messages, not raw Rust debug output.

### 10. CLI Arguments

The current Build command only has `path`. Likely additions:
- `--output` / `-o`: output .mp4 path (default: `output.mp4` or `<scene>.mp4`)
- `--width` / `--height`: resolution override
- `--fps`: FPS override
- `--html-path`: React app location override
- `--keep-frames`: do not clean up intermediate PNGs

## Dependencies Not Yet Available

T-005-03 depends on T-005-01 (FFmpeg encoding) and T-005-02 (audio assembly). Both
are currently stubs. The Build command design must account for the FFmpeg API that
T-005-01 will define. The design should assume reasonable APIs based on the tickets'
acceptance criteria.

## Downstream Consumer

T-005-04 (end-to-end validation) depends on T-005-03. It will test the full pipeline
from scene construction through to .mp4 output. The Build command's internal pipeline
should be factored so it can be tested both via the CLI binary and programmatically.
