# T-006-04 Progress: TTS Validation Tests

## Completed

### Step 1: Created moron-voice/tests/tts.rs
- New test file with 2 ignored tests requiring Kokoro model files.
- `kokoro_synthesis_produces_valid_audio`: synthesizes text, validates AudioClip properties (sample_rate, channels, duration, data range, non-silence).
- `kokoro_wav_encoding_roundtrip`: synthesizes text, encodes to WAV, validates RIFF/WAVE header, PCM format, non-silence data.
- Module doc comment with setup instructions for downloading model files.

### Step 2: Added MockTtsBackend to moron-core/tests/e2e.rs
- `MockTtsBackend` struct implementing `VoiceBackend` with deterministic audio output.
- Produces 0.3s per word at a constant signal value (0.42) for easy verification.
- Uses `DEFAULT_SAMPLE_RATE` (48000 Hz).

### Step 3: Added non-ignored e2e tests
- `e2e_mock_tts_duration_resolution`: builds DemoScene, synthesizes with mock backend, resolves durations, verifies timeline updates from WPM estimates (0.4s/word) to mock TTS durations (0.3s/word).
- `e2e_audio_assembly_with_tts_clips`: builds scene with narrations + silence, synthesizes mock clips, assembles audio track, verifies sample-level accuracy (non-zero at narration positions, zero at silence positions), validates WAV output.

### Step 4: Added ignored e2e test with real Kokoro
- `e2e_full_pipeline_with_tts`: full integration test requiring Kokoro model files. Synthesizes DemoScene narrations, validates clip properties, resolves durations, assembles audio, encodes WAV. Optionally muxes with FFmpeg if available, producing a final .mp4 with both video and audio streams.

### Step 5: Updated e2e.rs module doc comment
- Added TTS test running instructions with env var setup.

### Step 6: Updated ticket frontmatter
- Set status: done, phase: done in T-006-04.md.

### Step 7: Verification
- `cargo check` passes.
- `cargo test` passes: 6 e2e tests pass, 4 ignored (3 FFmpeg, 1 Kokoro). All 105 unit tests pass. 2 tts.rs tests correctly ignored.
- `cargo clippy` clean for new code (existing warnings are pre-existing).

## Test Summary

| Test | File | Ignored | Requires |
|------|------|---------|----------|
| `e2e_mock_tts_duration_resolution` | e2e.rs | No | Nothing |
| `e2e_audio_assembly_with_tts_clips` | e2e.rs | No | Nothing |
| `e2e_full_pipeline_with_tts` | e2e.rs | Yes | Kokoro model + optionally FFmpeg |
| `kokoro_synthesis_produces_valid_audio` | tts.rs | Yes | Kokoro model |
| `kokoro_wav_encoding_roundtrip` | tts.rs | Yes | Kokoro model |

## Deviations from Plan
- None. Implementation followed the plan exactly.
