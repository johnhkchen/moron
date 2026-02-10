# T-006-04 Design: TTS Validation Tests

## Test Categories

### Category 1: Non-ignored (mock backend, no external deps)

**moron-core/tests/e2e.rs additions:**

1. `e2e_mock_tts_duration_resolution` -- Build DemoScene, synthesize with mock backend, verify timeline durations updated from WPM estimates to mock TTS durations. This tests the integration between facade, synthesize_narrations, and resolve_narration_durations.

2. `e2e_audio_assembly_with_tts_clips` -- Build a scene with narrations, create synthetic AudioClips (non-silence), pass them to assemble_audio_track, verify the assembled clip contains non-zero samples at narration positions and zero samples at silence positions.

### Category 2: Ignored (need Kokoro model files)

**moron-voice/tests/tts.rs (new file):**

1. `kokoro_synthesis_produces_valid_audio` -- Basic synthesis, verify AudioClip properties (sample_rate, channels, non-empty data, positive duration, data in [-1.0, 1.0]).

2. `kokoro_wav_encoding_roundtrip` -- Synthesize text, encode to WAV, verify WAV header correctness and non-silence.

**moron-core/tests/e2e.rs additions:**

3. `e2e_full_pipeline_with_tts` -- Build DemoScene with real KokoroBackend, run through synthesize_narrations, verify timeline durations differ from WPM estimates, assemble audio with real clips, optionally mux with FFmpeg if available, verify output.

## Design Decisions

### Placing moron-voice tests in tests/tts.rs
The ticket explicitly requests this. It exercises KokoroBackend through the public API (VoiceBackend trait), separate from the inline unit tests in kokoro.rs.

### Reusing existing mock pattern
The MockBackend in build.rs tests is a private test struct. Rather than making it public, we define a similar mock in e2e.rs. The pattern is simple: produce deterministic audio based on word count. This avoids coupling test infrastructure across module boundaries.

### env var gating pattern
Already established in kokoro.rs: `KOKORO_MODEL_PATH` and `KOKORO_VOICES_PATH`. We reuse this exact pattern. Tests that need both Kokoro and FFmpeg are double-gated (ignored + check both).

### Not duplicating kokoro.rs integration tests
The existing `synthesize_produces_audio`, `synthesize_duration_scales_with_text`, and `synthesize_different_voices` tests in kokoro.rs already cover basic Kokoro properties. The new tests in tts.rs focus on WAV output validation and properties not covered there.

### Rejected: Testing build_video with real TTS
Too heavy for a test -- requires Chrome + FFmpeg + Kokoro. The pipeline is already tested in stages. Testing individual stages with real TTS is sufficient.

## Documentation

Add a comment block at the top of tts.rs and extend the e2e.rs header comment explaining how to run TTS tests with model files.
