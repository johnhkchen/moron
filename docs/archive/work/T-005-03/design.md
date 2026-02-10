# T-005-03 Design: moron-build-cli

## Problem

Implement the `moron build` CLI command that runs the full rendering pipeline:
scene construction -> frame rendering -> FFmpeg encoding -> .mp4 output. For the
initial version, scenes are compiled into the binary as a built-in demo scene.

---

## Design Decisions

### 1. Pipeline Function vs. Inline in main.rs

**Options considered:**

A) **All logic in the Build match arm** -- everything in main.rs
B) **Separate `build_pipeline` function** -- extracted but still in moron-cli
C) **Pipeline function in moron-core** -- reusable from CLI and tests

**Decision: (C) Pipeline function in moron-core, thin CLI wrapper in moron-cli.**

Rationale: T-005-04 (end-to-end validation) needs to test the full pipeline
programmatically without going through the CLI binary. A `build_video` function in
moron-core accepts a scene, config, and output path, returning a result. The CLI
becomes a thin adapter: parse args, construct config, call `build_video`, report
results. This also keeps moron-cli small (< 100 lines of build logic).

```rust
// moron-core — new public function
pub async fn build_video(
    m: &M,
    config: BuildConfig,
) -> Result<BuildResult, BuildError>;
```

### 2. BuildConfig

**Decision: Flat config struct combining render + encode settings.**

```rust
pub struct BuildConfig {
    pub output_path: PathBuf,       // final .mp4 path
    pub html_path: PathBuf,         // React app index.html
    pub width: u32,                 // default 1920
    pub height: u32,                // default 1080
    pub fps: Option<u32>,           // override timeline FPS, or None to use timeline's
    pub keep_frames: bool,          // do not delete temp PNGs
    pub progress: Option<Box<dyn Fn(BuildProgress)>>,
}
```

The BuildConfig wraps both rendering and encoding concerns. Internally, `build_video`
constructs a `RenderConfig` and an FFmpeg encode config from it. This avoids exposing
intermediate config types to CLI callers.

### 3. Demo Scene

**Options considered:**

A) **DemoScene struct in moron-cli** -- only accessible from CLI
B) **DemoScene in moron-core** -- reusable, testable
C) **DemoScene in examples/** -- separate binary

**Decision: (B) DemoScene struct in moron-core, in a `demo` module.**

Rationale: The demo scene must be usable from both the CLI (moron build with no
scene file) and from T-005-04's end-to-end test. Placing it in moron-core makes it
importable from both. The demo module is small (~30 lines) and uses only existing
facade methods and techniques.

```rust
// moron-core/src/demo.rs
pub struct DemoScene;

impl Scene for DemoScene {
    fn build(m: &mut M) {
        m.title("moron Demo");
        m.narrate("This is a demo of the moron rendering pipeline.");
        m.play(FadeIn::default());
        m.beat();
        m.section("Pipeline");
        m.narrate("Scene to timeline to frames to video.");
        m.play(FadeUp::default());
        m.breath();
        m.show("Built with Rust.");
        m.play(FadeIn { duration: 0.5 });
    }
}
```

Uses only `FadeIn`, `FadeUp` (available techniques), facade methods that exist,
and produces a timeline of ~6-8 seconds -- enough to demonstrate the pipeline without
being wasteful.

### 4. Temp Directory Management

**Options considered:**

A) **`tempfile::TempDir`** -- auto-cleanup via Drop, new dependency
B) **`std::env::temp_dir()` + manual cleanup** -- no new dependency
C) **`.moron-build/` in project directory** -- visible, manually inspectable
D) **`std::env::temp_dir()` + unique suffix, manual cleanup, `keep_frames` flag**

**Decision: (D) System temp dir with manual cleanup and keep_frames escape hatch.**

Rationale: Adding the `tempfile` crate is unnecessary for a single temp directory.
Using `std::env::temp_dir()` with a unique subdirectory (e.g., `moron-build-{pid}`)
keeps intermediate frames invisible to the user by default. The `keep_frames` flag
preserves the directory for debugging. On error, frames are preserved automatically
(cleanup only runs on success, unless keep_frames is set).

```rust
let frames_dir = std::env::temp_dir().join(format!("moron-build-{}", std::process::id()));
// ... render frames to frames_dir ...
// ... encode frames_dir -> output.mp4 ...
if !config.keep_frames {
    std::fs::remove_dir_all(&frames_dir)?;
}
```

### 5. React App Location

**Options considered:**

A) **Hardcoded relative path** -- `packages/ui/dist/index.html`
B) **CLI flag** -- `--html-path`
C) **Embed HTML in binary** -- via `include_str!` or `rust-embed`
D) **Convention: look in known locations** -- project dir, then installed location

**Decision: (B) Required CLI flag with (D) as fallback convention.**

Rationale: For the initial version, the React app must be built separately. The CLI
cannot assume a fixed filesystem layout. A `--html-path` flag gives full control. As
a convenience, the CLI also checks `packages/ui/dist/index.html` relative to the
project path. If neither is available, a clear error message is emitted.

In the future, the React app could be embedded in the binary (option C), but that
is a separate ticket requiring build tooling changes.

### 6. CLI Argument Extensions

**Decision: Extend the Build command with output, resolution, and html-path flags.**

```rust
Build {
    #[arg(default_value = ".")]
    path: String,
    #[arg(short, long, default_value = "output.mp4")]
    output: String,
    #[arg(long)]
    html_path: Option<String>,
    #[arg(long, default_value = "1920")]
    width: u32,
    #[arg(long, default_value = "1080")]
    height: u32,
    #[arg(long)]
    keep_frames: bool,
}
```

The `--output` flag defaults to `output.mp4` in the current directory. Resolution
defaults to 1920x1080. These match BridgeConfig defaults. No FPS flag for now --
the timeline's FPS (30) is used.

### 7. Async Runtime

**Decision: Switch main to `#[tokio::main]`.**

The `render` function is async. The FFmpeg encoding will likely be async (spawning
a child process and waiting). `tokio` is already a dependency. The change is a
one-line annotation on main.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
```

### 8. Progress Reporting

**Decision: Phased progress with eprintln for CLI, callback for programmatic use.**

The build pipeline has distinct phases. The CLI reports each:

```
[1/4] Building scene...
[2/4] Rendering frames... (frame 1/90, 2/90, ... 90/90)
[3/4] Encoding video...
[4/4] Cleaning up...
Done: output.mp4 (90 frames, 3.0s)
```

The `BuildProgress` enum captures phase transitions:

```rust
pub enum BuildProgress {
    SceneBuilt { total_duration: f64, total_frames: u32 },
    RenderingFrame { current: u32, total: u32 },
    Encoding,
    Complete { output_path: PathBuf, total_frames: u32, duration: f64 },
}
```

The CLI provides a progress callback that formats these as the eprintln messages above.
The `build_video` function in moron-core calls the progress callback at each phase.

### 9. Error Handling

**Decision: BuildError enum wrapping component errors, CLI translates to user messages.**

```rust
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("render failed: {0}")]
    Render(#[from] RenderError),
    #[error("encoding failed: {0}")]
    Encode(#[from] EncodeError),  // from T-005-01
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(String),
}
```

The CLI catches BuildError and translates specific variants into user-friendly messages:

- `BuildError::Render(RenderError::Bridge(BridgeError::ChromeNotFound))` ->
  "Error: Chrome/Chromium not found. Install Chrome or set CHROME_PATH."
- `BuildError::Encode(EncodeError::FfmpegNotFound)` ->
  "Error: FFmpeg not found. Install FFmpeg or add it to PATH."

### 10. Pipeline Flow

The complete `build_video` flow:

1. **Scene build**: Call `Scene::build(&mut m)`. Record total_duration and total_frames.
   Report `BuildProgress::SceneBuilt`.
2. **Create temp dir**: `temp_dir/moron-build-{pid}/frames/`.
3. **Render frames**: Call `render(&m, render_config).await`. The render config uses a
   progress callback that forwards `RenderingFrame` to the outer callback.
4. **Encode video**: Call the FFmpeg module (T-005-01 API) with the frames directory,
   FPS, resolution, and output path. Report `BuildProgress::Encoding`.
5. **Audio mux**: If T-005-02 is available, generate audio track from timeline and mux
   with video. Otherwise, video-only output is acceptable for initial version.
6. **Cleanup**: Remove temp directory unless `keep_frames` is set.
7. **Report**: Return `BuildResult` with output path, frame count, duration.

---

## What Was Rejected

- **Dynamic scene loading from .rs files**: Requires a compilation step (rustc or
  dynamic linking). Out of scope -- the ticket explicitly states "compiled into the
  binary." Future work.

- **Embedded React app**: Using `include_str!` or `rust-embed` to bundle the React
  app HTML/JS into the binary. Good idea but requires build tooling changes and a
  built React app at compile time. Separate ticket.

- **Streaming frames to FFmpeg stdin**: Piping PNG bytes directly instead of writing
  to disk. Optimization that bypasses the disk I/O. Worth doing eventually but adds
  complexity to the initial version. The disk approach is simpler, debuggable (frames
  are inspectable), and sufficient for explainer-length videos.

- **tempfile crate**: Adds a dependency for one `TempDir`. Manual temp dir management
  with `std::env::temp_dir()` is straightforward and avoids dependency bloat.

- **Separate progress crate/trait**: Over-engineering for a callback closure. A
  `Box<dyn Fn(BuildProgress)>` is sufficient and matches the pattern already
  established in renderer.rs.

- **Configuration file (.moron.toml)**: Project-level configuration for resolution,
  FPS, theme, etc. Useful eventually but premature for the first working build
  command. CLI flags are sufficient.

---

## Module Placement Summary

| Component | Location | New/Modified |
|---|---|---|
| `build_video` function | moron-core/src/build.rs | New |
| `BuildConfig`, `BuildResult`, `BuildError`, `BuildProgress` | moron-core/src/build.rs | New |
| `DemoScene` | moron-core/src/demo.rs | New |
| Build command implementation | moron-cli/src/main.rs | Modified |
| Re-exports | moron-core/src/lib.rs | Modified |

The Build command in main.rs will be approximately 40-50 lines: parse CLI args,
construct BuildConfig, construct M and build demo scene, call `build_video`, handle
result/error.

## Assumptions About T-005-01 and T-005-02

This design assumes T-005-01 will provide:
```rust
pub async fn encode(config: EncodeConfig) -> Result<(), EncodeError>;

pub struct EncodeConfig {
    pub input_dir: PathBuf,         // directory of frame_NNNNNN.png
    pub input_pattern: String,      // "frame_%06d.png"
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub output_path: PathBuf,
}
```

And T-005-02 will provide audio assembly that integrates with the encode step.
If the actual APIs differ, the `build_video` function adapts accordingly -- the
CLI interface does not change.
