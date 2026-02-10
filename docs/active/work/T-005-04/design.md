# T-005-04 Design: End-to-End Validation Test

## Goal

Write an end-to-end test that exercises: scene -> timeline -> frames -> video.
Gated with `#[ignore]` for CI environments lacking Chrome + FFmpeg.

## Options Considered

### Option A: Call `build_video()` directly

Use `build_video()` as-is. This requires Chrome, a built React app (html_path),
and FFmpeg. It is the truest end-to-end test but needs all three system dependencies
simultaneously. The React app HTML does not exist in the repository by default.

### Option B: Manual pipeline steps with synthetic frames

Skip Chrome rendering entirely. Instead:
1. Build DemoScene to get a valid M with timeline
2. Compute FrameState for each frame and verify serialization
3. Write synthetic PNGs (e.g., solid-color 1x1 or small images) to a temp dir
4. Encode via `ffmpeg::encode()`
5. Assemble audio and mux via `ffmpeg::mux_audio()`
6. Validate the output .mp4

This tests the full pipeline minus Chrome, but Chrome is the one piece we
already know works (it's a thin wrapper around chromiumoxide). The interesting
integration points are: scene builds correctly, frame states serialize, FFmpeg
accepts the frames, muxing produces valid output.

### Option C: Hybrid -- manual steps + optional full `build_video()`

Provide two test functions:
- `e2e_pipeline_manual`: Uses synthetic frames (needs only FFmpeg)
- `e2e_build_video_full`: Calls `build_video()` (needs Chrome + FFmpeg + HTML)

## Decision: Option B (manual pipeline with synthetic frames)

**Rationale:**
- Chrome is the most restrictive dependency and least likely to be available
- The Chromium bridge is well-tested in isolation and is a thin CDP wrapper
- The real integration risk is in the frame->encode->mux->mp4 path
- Writing synthetic PNG frames is trivial and tests the same FFmpeg path
- We can still validate FrameState computation for every frame
- A single `#[ignore]` test that needs only FFmpeg is more likely to be run

We will also include a helper function `create_minimal_png()` that produces
valid PNG bytes for a small solid-color image. FFmpeg requires valid PNGs
(not just any bytes), so we use the minimal valid PNG format.

## Validation Strategy

The test will verify:
1. DemoScene builds and produces a valid timeline (frames > 0, duration > 0)
2. FrameState can be computed and serialized for every frame
3. Synthetic PNGs written to temp dir are accepted by FFmpeg
4. `ffmpeg::encode()` produces a video-only .mp4
5. `ffmpeg::mux_audio()` produces a final .mp4 with video + audio
6. Output file exists and is non-empty
7. Optional: ffprobe validation of duration and streams (if ffprobe available)

## Error Handling Tests

Also include a non-ignored test that validates error paths:
- `build_video()` with zero-frame scene returns appropriate error
- `BuildConfig` with missing html_path is caught

These tests need no system dependencies and can run in CI.

## Test File Location

`moron-core/tests/e2e.rs` -- as specified in the ticket.
