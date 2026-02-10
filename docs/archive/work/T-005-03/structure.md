# T-005-03 Structure: moron-build-cli

## New Files

### moron-core/src/build.rs

Public module containing the pipeline orchestration function and its types.

**Types:**

```rust
pub struct BuildConfig {
    pub output_path: PathBuf,
    pub html_path: PathBuf,
    pub width: u32,           // default 1920
    pub height: u32,          // default 1080
    pub keep_frames: bool,    // default false
    pub progress: Option<Box<dyn Fn(BuildProgress)>>,
}

pub enum BuildProgress {
    SceneBuilt { total_duration: f64, total_frames: u32 },
    RenderingFrame { current: u32, total: u32 },
    Encoding,
    MuxingAudio,
    Complete { output_path: PathBuf, total_frames: u32, duration: f64 },
}

pub struct BuildResult {
    pub output_path: PathBuf,
    pub total_frames: u32,
    pub duration: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    Render(#[from] RenderError),
    Ffmpeg(#[from] FfmpegError),
    Io(#[from] std::io::Error),
    Config(String),
}
```

**Functions:**

```rust
pub async fn build_video(m: &M, config: BuildConfig) -> Result<BuildResult, BuildError>;
```

Internal flow:
1. Report `SceneBuilt` with timeline stats
2. Create temp dir `{temp_dir}/moron-build-{pid}`
3. Construct `RenderConfig` from `BuildConfig`, call `render()`
4. Construct `EncodeConfig`, call `encode()` (video-only .mp4)
5. Assemble audio track from timeline, write WAV to temp dir
6. Call `mux_audio()` to merge video + audio into final output
7. Clean up temp dir (unless `keep_frames`)
8. Report `Complete`, return `BuildResult`

### moron-core/src/demo.rs

Public module containing the built-in demo scene.

```rust
pub struct DemoScene;

impl Scene for DemoScene {
    fn build(m: &mut M) { ... }
}
```

The scene uses only existing facade methods and techniques:
`m.title()`, `m.narrate()`, `m.play(FadeIn)`, `m.beat()`, `m.section()`,
`m.play(FadeUp)`, `m.breath()`, `m.show()`.

Produces a timeline of ~4-6 seconds -- enough to validate the pipeline.

---

## Modified Files

### moron-core/src/lib.rs

- Add `pub mod build;` and `pub mod demo;` module declarations
- Re-export: `build::{build_video, BuildConfig, BuildError, BuildProgress, BuildResult}`
- Re-export: `demo::DemoScene`
- Add to prelude: same re-exports

### moron-cli/src/main.rs

Major changes:

1. Switch `fn main()` to `#[tokio::main] async fn main()`
2. Extend `Commands::Build` with additional CLI arguments:
   - `--output` / `-o`: output path (default "output.mp4")
   - `--html-path`: path to React app index.html
   - `--width`: viewport width (default 1920)
   - `--height`: viewport height (default 1080)
   - `--keep-frames`: preserve intermediate PNGs
3. Implement Build handler:
   - Construct `M::new()`, call `DemoScene::build(&mut m)`
   - Resolve html_path (CLI flag or `{path}/packages/ui/dist/index.html`)
   - Construct `BuildConfig` with progress callback
   - Call `build_video(&m, config).await`
   - Handle errors with user-friendly messages
4. Progress callback uses `eprintln!` for phased reporting

### moron-cli/Cargo.toml

No changes needed. `moron-core`, `clap`, `tokio`, and `anyhow` are already
dependencies. `thiserror` is not needed in CLI (errors come from moron-core,
CLI uses anyhow for reporting).

---

## Module Boundaries

```
moron-cli/src/main.rs
    |
    | uses: moron_core::{M, DemoScene, Scene, build_video, BuildConfig, BuildProgress, BuildError}
    |
moron-core/src/build.rs
    |
    | uses: crate::renderer::{render, RenderConfig, RenderError, RenderProgress}
    | uses: crate::chromium::BridgeConfig
    | uses: crate::ffmpeg::{encode, mux_audio, assemble_audio_track, EncodeConfig, FfmpegError}
    | uses: crate::facade::M
    | uses: moron_voice::AudioClip (for to_wav_bytes)
    |
moron-core/src/demo.rs
    |
    | uses: crate::facade::{M, Scene}
    | uses: moron_techniques::{FadeIn, FadeUp}
```

## File Sizes (estimated)

| File | Lines | New/Modified |
|------|-------|--------------|
| moron-core/src/build.rs | ~160 | New |
| moron-core/src/demo.rs | ~30 | New |
| moron-core/src/lib.rs | ~45 (from ~37) | Modified |
| moron-cli/src/main.rs | ~120 (from ~55) | Modified |

Total new/changed: ~355 lines across 4 files.

## Ordering Constraints

1. `demo.rs` and `build.rs` can be created independently (no dependency between them)
2. `lib.rs` must be updated after both new modules exist
3. `main.rs` must be updated last (depends on all moron-core changes)
4. `cargo check` validates the full chain
