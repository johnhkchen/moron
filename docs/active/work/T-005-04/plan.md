# T-005-04 Plan: End-to-End Validation Test

## Step 1: Create `moron-core/tests/e2e.rs` with header and imports

- Add file header explaining how to run: `cargo test --test e2e -- --ignored`
- Import `moron_core::prelude::*` and necessary std types
- Verify: `cargo check --test e2e`

## Step 2: Implement helper functions

- `create_minimal_png()`: hardcoded valid PNG bytes (small solid image)
- `write_synthetic_frames()`: writes numbered PNGs to a directory
- `ffprobe_duration()`: optional ffprobe duration check
- `ffprobe_has_video_stream()`: optional ffprobe stream check
- `ffprobe_has_audio_stream()`: optional ffprobe stream check
- Verify: `cargo check --test e2e`

## Step 3: Implement non-ignored tests

- `e2e_demo_scene_frame_states_serialize`: Builds DemoScene, iterates all
  frames, computes FrameState, serializes to JSON, asserts valid.
- `e2e_empty_scene_produces_error`: Uses tokio runtime to call `build_video`
  with empty scene, asserts Config error.
- Verify: `cargo test --test e2e` (runs only non-ignored)

## Step 4: Implement ignored `e2e_full_pipeline` test

- Build DemoScene, verify timeline has frames
- Compute all FrameStates (exercise frame computation)
- Create temp dir, write synthetic frames
- Call `ffmpeg::encode()` to produce video-only mp4
- Call `ffmpeg::assemble_audio_track()` + write WAV
- Call `ffmpeg::mux_audio()` to produce final mp4
- Assert: output exists, file size > 0
- Optional: ffprobe duration and stream checks
- Cleanup temp files
- Verify: `cargo test --test e2e -- --ignored` (needs FFmpeg locally)

## Step 5: Implement ignored `e2e_encode_and_mux_roundtrip` test

- Simpler focused test: write frames, encode, mux, validate
- No scene building -- just validates the FFmpeg path works
- Verify: `cargo test --test e2e -- --ignored`

## Step 6: Update ticket frontmatter

- Set status: done, phase: done in `docs/active/tickets/T-005-04.md`

## Step 7: Final verification

- `cargo check` (full workspace)
- `cargo test` (all non-ignored tests pass)
- Review that no existing tests are broken

## Testing Strategy

| Test                                  | Ignored? | Needs       |
|---------------------------------------|----------|-------------|
| `e2e_demo_scene_frame_states_serialize` | No     | Nothing     |
| `e2e_empty_scene_produces_error`       | No      | Nothing     |
| `e2e_full_pipeline`                    | Yes     | FFmpeg      |
| `e2e_encode_and_mux_roundtrip`         | Yes     | FFmpeg      |
