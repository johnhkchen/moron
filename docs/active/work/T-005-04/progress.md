# T-005-04 Progress: End-to-End Validation Test

## Status: Complete

## Completed Steps

### Step 1: Created `moron-core/tests/e2e.rs` with header and imports
- Added file header with run instructions
- Imported `moron_core::prelude::*` and std types
- Verified: `cargo check` passes

### Step 2: Implemented helper functions
- `minimal_png_bytes()`: Constructs a valid 8x8 solid-blue PNG from scratch
  using raw PNG construction (signature, IHDR, IDAT with zlib stored blocks, IEND).
  No external image crate dependency.
- `write_synthetic_frames()`: Writes numbered frame PNGs to a directory
- `ffprobe_duration()`: Optional ffprobe duration extraction
- `ffprobe_has_video_stream()`: Optional ffprobe video stream check
- `ffprobe_has_audio_stream()`: Optional ffprobe audio stream check
- `test_temp_dir()`: Creates unique temp directories per test
- Supporting helpers: `write_png_chunk`, `crc32`, `zlib_compress_stored`, `adler32`

### Step 3: Implemented non-ignored tests (4 tests)
- `e2e_demo_scene_frame_states_serialize`: Builds DemoScene, computes and
  serializes FrameState for every frame, validates JSON structure.
- `e2e_demo_scene_timeline_properties`: Validates segment count, duration
  range, FPS, and frame count consistency.
- `e2e_empty_scene_produces_build_error`: Verifies build_video rejects
  zero-frame scenes with clear error message.
- `e2e_audio_assembly_from_demo_scene`: Validates audio track assembly
  produces correct duration and valid WAV bytes.
- All 4 pass: `cargo test --test e2e`

### Step 4: Implemented ignored tests (3 tests)
- `e2e_full_pipeline`: Complete pipeline: DemoScene -> timeline -> FrameState
  computation -> synthetic PNGs -> FFmpeg encode -> audio assembly -> mux ->
  validate output .mp4 (exists, non-empty, ffprobe checks).
- `e2e_encode_and_mux_roundtrip`: Focused FFmpeg test: write 30 frames,
  encode, mux with silence audio, validate.
- `e2e_ffmpeg_rejects_empty_frames_dir`: Error path: encoding with no frames.
- All gated with `#[ignore]` and require FFmpeg on PATH.
- Tests produce clear error messages when FFmpeg is unavailable.

### Step 5: Updated ticket frontmatter
- Set status: done, phase: done

### Step 6: Final verification
- `cargo check` passes (full workspace)
- `cargo test` passes: 130 tests pass, 6 ignored (3 e2e + 3 doctests)
- All existing tests unaffected

## Deviations from Plan

- Skipped `e2e_encode_and_mux_roundtrip` as "Step 5" since it was natural
  to implement alongside the other ignored tests in Step 4.
- Added `e2e_audio_assembly_from_demo_scene` (not in original plan) as a
  non-ignored test validating audio assembly and WAV encoding.
- Added `e2e_ffmpeg_rejects_empty_frames_dir` as an additional error-path test.
- PNG construction implemented from scratch (CRC32, zlib stored blocks, adler32)
  rather than using a hardcoded byte array, for clarity and correctness.

## Test Summary

| Test                                    | Ignored | Dependencies | Result  |
|-----------------------------------------|---------|--------------|---------|
| `e2e_demo_scene_frame_states_serialize` | No      | None         | Pass    |
| `e2e_demo_scene_timeline_properties`    | No      | None         | Pass    |
| `e2e_empty_scene_produces_build_error`  | No      | None         | Pass    |
| `e2e_audio_assembly_from_demo_scene`    | No      | None         | Pass    |
| `e2e_full_pipeline`                     | Yes     | FFmpeg       | Gated   |
| `e2e_encode_and_mux_roundtrip`          | Yes     | FFmpeg       | Gated   |
| `e2e_ffmpeg_rejects_empty_frames_dir`   | Yes     | FFmpeg       | Gated   |
