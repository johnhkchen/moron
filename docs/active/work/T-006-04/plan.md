# T-006-04 Plan: TTS Validation Tests

## Step 1: Create moron-voice/tests/tts.rs

Create the new test file with:
- Module doc comment explaining env var setup and how to run.
- Helper function `kokoro_config()` to read env vars.
- Two ignored tests:
  1. `kokoro_synthesis_produces_valid_audio` -- synthesize short text, assert AudioClip properties.
  2. `kokoro_wav_encoding_roundtrip` -- synthesize, encode to WAV, verify header and non-silence.

Verify: `cargo test -p moron-voice --test tts` (should skip ignored tests).

## Step 2: Add MockTtsBackend to moron-core/tests/e2e.rs

Add a MockTtsBackend struct in the helper section that:
- Implements `moron_voice::VoiceBackend`.
- Produces deterministic audio (0.3s per word, non-zero sample value 0.42).
- Has configurable sample_rate.

## Step 3: Add non-ignored e2e tests

Add to moron-core/tests/e2e.rs:
1. `e2e_mock_tts_duration_resolution` -- Build DemoScene, call synthesize_narrations equivalent logic (use facade methods directly since synthesize_narrations is private), verify duration changes.
2. `e2e_audio_assembly_with_tts_clips` -- Build scene with narrations, create mock AudioClips, pass to assemble_audio_track, verify non-zero samples at narration positions.

Note: `synthesize_narrations` is private to build.rs, so we use the public API: create MockTtsBackend, call backend.synthesize() for each narration text, collect durations, call m.resolve_narration_durations(), then verify.

Verify: `cargo test --test e2e` (non-ignored tests pass).

## Step 4: Add ignored e2e test with real Kokoro

Add `e2e_full_pipeline_with_tts`:
- Ignored (requires KOKORO_MODEL_PATH and KOKORO_VOICES_PATH).
- Create KokoroBackend, build DemoScene, synthesize all narrations.
- Verify timeline durations changed from WPM estimates.
- Assemble audio track with real TTS clips.
- Verify WAV output has non-zero samples.
- Optionally encode + mux if FFmpeg is available.

Verify: `cargo test --test e2e -- --ignored` (requires model + FFmpeg).

## Step 5: Update e2e.rs module doc comment

Extend the header to document TTS test running with env vars.

## Step 6: Update ticket frontmatter

Set status: done, phase: done in T-006-04.md.

## Step 7: Verify

- `cargo check` passes.
- `cargo test` passes (all non-ignored tests).
- `cargo clippy` passes.

## Testing Strategy

- Non-ignored tests use MockTtsBackend (no external deps).
- Ignored tests require Kokoro model files (env vars) and optionally FFmpeg.
- All tests independent and can run in any order.
