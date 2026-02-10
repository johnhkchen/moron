# T-006-04 Research: TTS Validation Tests

## Existing Test Infrastructure

### moron-core/tests/e2e.rs
- 624 lines of end-to-end tests.
- Non-ignored tests: `e2e_demo_scene_frame_states_serialize`, `e2e_demo_scene_timeline_properties`, `e2e_empty_scene_produces_build_error`, `e2e_audio_assembly_from_demo_scene`.
- Ignored tests (require FFmpeg): `e2e_full_pipeline`, `e2e_encode_and_mux_roundtrip`, `e2e_ffmpeg_rejects_empty_frames_dir`.
- Helper functions: `minimal_png_bytes()`, `write_synthetic_frames()`, `ffprobe_duration()`, `ffprobe_has_video_stream()`, `ffprobe_has_audio_stream()`, `test_temp_dir()`.
- Uses `moron_core::prelude::*` for imports.

### moron-core/src/build.rs (inline tests)
- Already contains a `MockBackend` struct implementing `VoiceBackend` (sample_rate, seconds_per_word).
- Tests: `synthesize_narrations_resolves_durations`, `synthesize_narrations_no_narrations`, `synthesize_narrations_reports_progress`, `synthesize_narrations_propagates_error`.
- Also has a `FailingBackend` mock.
- These are unit tests in `mod tests`, not integration tests.

### moron-voice/tests/ -- empty
- No existing test files under `moron-voice/tests/`.
- All moron-voice tests are inline in `src/kokoro.rs`, `src/audio.rs`, `src/backend.rs`.

## TTS Pipeline Architecture

### VoiceBackend trait (moron-voice/src/backend.rs)
- `fn synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error>`
- `fn name(&self) -> &str`

### AudioClip (moron-voice/src/audio.rs)
- Fields: `data: Vec<f32>`, `duration: f64`, `sample_rate: u32`, `channels: u16`.
- Methods: `silence()`, `duration()`, `append()`, `concat()`, `to_wav_bytes()`.
- `DEFAULT_SAMPLE_RATE = 48000`.

### KokoroBackend (moron-voice/src/kokoro.rs)
- Feature-gated behind `kokoro` (default feature).
- `KokoroConfig::new(model_path, voices_path)` with `.with_voice()`, `.with_speed()`.
- `KokoroBackend::new(config)` -- lazy model loading.
- Produces `KOKORO_SAMPLE_RATE = 24000` Hz audio.
- Already has ignored integration tests using `KOKORO_MODEL_PATH` / `KOKORO_VOICES_PATH` env vars.

### Build pipeline (moron-core/src/build.rs)
- `build_video(m, config)` -- full pipeline entry point.
- `synthesize_narrations(m, backend, progress)` -- extracts narration texts, calls `backend.synthesize()`, collects clips, calls `m.resolve_narration_durations()`.
- `BuildConfig` has `voice_backend: Option<Arc<dyn VoiceBackend + Send + Sync>>`.

### Audio assembly (moron-core/src/ffmpeg.rs)
- `assemble_audio_track(timeline, sample_rate, narration_clips)` -- walks segments, splices TTS clips into narration positions, silence elsewhere.

### Facade (moron-core/src/facade.rs)
- `M::narrate(text)` -- adds `Segment::Narration` with WPM-estimated duration.
- `M::resolve_narration_durations(durations)` -- replaces WPM estimates with actual TTS durations.

## What Needs Testing

1. **TTS synthesis properties** (moron-voice): Kokoro produces valid AudioClip with correct sample_rate, non-empty data, positive duration.
2. **Duration resolution** (moron-core): Mock backend -> synthesize_narrations -> durations resolve correctly in timeline.
3. **Audio assembly with TTS clips** (moron-core): assemble_audio_track with real narration_clips splices audio correctly.
4. **Full pipeline with TTS** (moron-core): scene -> TTS synthesis -> audio assembly -> mux -> .mp4 with audio stream.

## Gating Strategy

- Kokoro model tests: `#[ignore]` with env vars `KOKORO_MODEL_PATH` and `KOKORO_VOICES_PATH`.
- FFmpeg tests: `#[ignore]` (already established pattern).
- Mock backend tests: non-ignored (no external dependencies).

## Existing Kokoro Tests (already in kokoro.rs)
- `synthesize_produces_audio` (ignored)
- `synthesize_duration_scales_with_text` (ignored)
- `synthesize_different_voices` (ignored)

These already cover basic Kokoro synthesis properties. New tests should focus on pipeline integration.
