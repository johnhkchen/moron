# T-005-04 Research: End-to-End Validation Test

## Pipeline Architecture

The moron rendering pipeline follows a linear flow:

```
Scene (DemoScene) -> M (facade) -> Timeline -> FrameState[] -> PNGs -> FFmpeg encode -> mux audio -> .mp4
```

### Entry Point: `build_video()` in `moron-core/src/build.rs`

`build_video(m: &M, config: BuildConfig) -> Result<BuildResult, BuildError>` is the
top-level orchestrator. It is async (Chromium bridge is async). Steps:

1. Reads timeline stats (duration, frame count, fps)
2. Creates temp directory for intermediate files
3. Renders frames via `renderer::render()` using `ChromiumBridge`
4. Encodes frames to video-only .mp4 via `ffmpeg::encode()`
5. Assembles audio track via `ffmpeg::assemble_audio_track()`
6. Muxes video + audio via `ffmpeg::mux_audio()`
7. Cleans up temp files

### Scene: `DemoScene` in `moron-core/src/demo.rs`

A minimal scene exercising title, narrate, play, beat, section, show, breath.
Produces ~5 seconds at 30 FPS. Already tested in unit tests.

### Chromium Bridge: `moron-core/src/chromium.rs`

`ChromiumBridge::launch()` requires a real Chrome/Chromium binary and an HTML file
exposing `window.__moron_setFrame`. This is the hardest dependency for CI.

### FFmpeg: `moron-core/src/ffmpeg.rs`

- `detect_ffmpeg()` checks PATH for ffmpeg binary
- `encode()` converts frame PNGs to H.264 video
- `mux_audio()` combines video + audio WAV to final .mp4
- `assemble_audio_track()` builds silence-only AudioClip from timeline

### Frame State: `moron-core/src/frame.rs`

`compute_frame_state(m, time)` produces a `FrameState` (serializable to JSON).
This is the Rust->React contract. No external dependencies.

### Existing Tests

- `moron-core/tests/integration.rs`: Tests facade->timeline->technique flow. No I/O.
- Unit tests in each module: Extensive coverage of types and logic, no system deps.

## System Dependencies

| Dependency  | Required for        | CI availability |
|-------------|---------------------|-----------------|
| Chrome      | Frame rendering     | Unlikely        |
| FFmpeg      | Video encoding/mux  | Sometimes       |
| React app   | HTML host page      | Not built in CI |

## Key Constraints

1. **The full `build_video()` path requires Chrome + FFmpeg + built React app** -- all
   three are unlikely in a standard CI environment.
2. **Individual pipeline stages can be tested in isolation** without some deps (e.g.,
   frame state computation needs nothing, FFmpeg encode needs only ffmpeg binary +
   frame PNGs on disk, audio assembly needs nothing).
3. **`#[ignore]` gating** is the ticket's specified approach for CI.
4. **tokio runtime** is required for async functions (`build_video`, `render`).

## Files Inventory

| File                               | Role                           |
|------------------------------------|--------------------------------|
| `moron-core/src/build.rs`         | `build_video`, `BuildConfig`   |
| `moron-core/src/demo.rs`          | `DemoScene`                    |
| `moron-core/src/renderer.rs`      | `render`, `RenderConfig`       |
| `moron-core/src/chromium.rs`      | `ChromiumBridge`, `BridgeConfig`|
| `moron-core/src/ffmpeg.rs`        | `encode`, `mux_audio`, etc.    |
| `moron-core/src/frame.rs`         | `compute_frame_state`          |
| `moron-core/src/facade.rs`        | `M`, `Scene`                   |
| `moron-core/src/timeline.rs`      | `Timeline`, `Segment`          |
| `moron-core/src/lib.rs`           | Re-exports, prelude            |
| `moron-core/tests/integration.rs` | Existing integration tests     |
| `moron-core/tests/e2e.rs`         | Target file (new)              |
