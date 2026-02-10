# T-005-03 Plan: moron-build-cli

## Implementation Steps

### Step 1: Create moron-core/src/demo.rs

Create the DemoScene struct implementing Scene trait. Uses only existing
facade methods and techniques (FadeIn, FadeUp).

**Verification:** Module compiles in isolation (checked in step 3).

### Step 2: Create moron-core/src/build.rs

Create the build pipeline module with:
- `BuildConfig` struct
- `BuildProgress` enum
- `BuildResult` struct
- `BuildError` enum (wraps RenderError, FfmpegError, io::Error)
- `build_video()` async function implementing the full pipeline

Pipeline flow inside `build_video`:
1. Read timeline stats, report `SceneBuilt`
2. Create temp dir `{temp_dir}/moron-build-{pid}`
3. Build `RenderConfig` from `BuildConfig`, call `render().await`
4. Build `EncodeConfig`, call `encode()` for video-only intermediate
5. Call `assemble_audio_track()`, write WAV via `to_wav_bytes()`
6. Call `mux_audio()` to combine video + audio into final output
7. Clean up temp dir and intermediate files (unless `keep_frames`)
8. Report `Complete`, return `BuildResult`

**Verification:** Module compiles in isolation (checked in step 3).

### Step 3: Update moron-core/src/lib.rs

Add module declarations and re-exports:
- `pub mod build;`
- `pub mod demo;`
- Re-export key types at crate root and in prelude

**Verification:** `cargo check -p moron-core` passes.

### Step 4: Update moron-cli/src/main.rs

1. Change `fn main()` to `#[tokio::main] async fn main()`
2. Extend Build command with CLI args (output, html_path, width, height, keep_frames)
3. Implement Build handler:
   - Construct M, build DemoScene
   - Resolve html_path
   - Create BuildConfig with eprintln progress callback
   - Call `build_video().await`
   - Map errors to user-friendly messages

**Verification:** `cargo check -p moron-cli` passes.

### Step 5: Run full verification

- `cargo check` (workspace-wide)
- `cargo test` (all tests pass)
- `cargo clippy` (no warnings)

---

## Testing Strategy

### Unit Tests in build.rs

- `build_config_defaults`: Verify BuildConfig::new() defaults
- `build_error_display`: Verify error message formatting
- `build_progress_variants`: Verify all progress enum variants exist
- `build_result_fields`: Verify BuildResult field access

### Unit Tests in demo.rs

- `demo_scene_builds`: Verify DemoScene::build produces a non-empty timeline
- `demo_scene_has_frames`: Verify timeline produces > 0 frames
- `demo_scene_uses_available_techniques`: Verify scene builds without panic

### Existing Tests

All existing tests in moron-core (renderer, ffmpeg, facade, timeline, frame,
chromium) must continue to pass. No existing code is modified in ways that
could break them.

### Integration Testing

Full pipeline integration testing (actually running Chrome + FFmpeg) is
deferred to T-005-04 (end-to-end validation). This ticket focuses on
wiring and compilation correctness.

---

## Risk Mitigation

- **FFmpeg `encode()` is synchronous**: The design called for async encode,
  but the actual implementation in ffmpeg.rs uses `std::process::Command`
  synchronously. The `build_video` function will call it directly (no `.await`).
  This is fine -- FFmpeg runs as a subprocess, and the blocking call is
  acceptable for a CLI tool.

- **Audio muxing creates intermediate files**: The pipeline needs a video-only
  .mp4, then muxes with audio. This means two FFmpeg passes and a temporary
  video file. The temp dir handles cleanup.

- **html_path might not exist**: The CLI checks for html_path existence before
  calling build_video. A clear error message guides the user.

---

## Step Dependencies

```
Step 1 (demo.rs) ──┐
                    ├── Step 3 (lib.rs) ── Step 4 (main.rs) ── Step 5 (verify)
Step 2 (build.rs) ─┘
```

Steps 1 and 2 are independent. Step 3 depends on both. Step 4 depends on 3.
Step 5 depends on 4.
