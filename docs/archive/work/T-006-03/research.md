# T-006-03 Research: Pipeline Audio Integration

## Current Build Pipeline (build.rs)

`build_video()` is the single entry point. It takes `&M` (built scene) and `BuildConfig`:

1. Extracts timeline stats (duration, frames, fps)
2. Creates temp dir for intermediates
3. Renders frames via Chromium bridge (`renderer::render`)
4. Encodes frames to video-only .mp4 (`ffmpeg::encode`)
5. Assembles audio track (`ffmpeg::assemble_audio_track`) -- **all silence**
6. Writes audio as WAV, muxes with video (`ffmpeg::mux_audio`)
7. Cleanup

**Key observation**: `build_video` takes `&M` (immutable). But `M::resolve_narration_durations` takes `&mut self`. The pipeline needs restructuring to call `resolve_narration_durations` before rendering, which means it needs `&mut M`.

**BuildConfig** currently has: output_path, html_path, width, height, keep_frames, progress. No TTS backend field.

## Audio Assembly (ffmpeg.rs)

`assemble_audio_track(timeline, sample_rate) -> AudioClip`:
- Iterates all segments, creates `AudioClip::silence(seg.duration(), sample_rate)` for every one
- Concatenates with `AudioClip::concat`
- No concept of real audio for narration segments

The function takes `&Timeline` and a sample_rate. It needs to optionally accept pre-synthesized audio clips.

## Facade (facade.rs)

`M` holds: next_element_id, current_theme, current_voice, timeline, elements.

Key methods for this ticket:
- `narrate(&mut self, text)` -- adds `Segment::Narration` with WPM-estimated duration
- `narration_count() -> usize` -- count of narration segments
- `resolve_narration_durations(&mut self, &[f64])` -- replaces WPM estimates with actual durations, recomputes element timestamps
- `timeline() -> &Timeline` -- immutable access

## Timeline (timeline.rs)

`Segment::Narration { text: String, duration: f64 }` -- text field available for TTS.
`narration_indices() -> Vec<usize>` -- returns indices of narration segments.
`segments() -> &[Segment]` -- ordered segment list.

## Voice Backend (moron-voice)

`VoiceBackend` trait: `synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error>`.
`KokoroBackend` implements it. Produces clips at 24kHz, mono.
`PiperBackend` is a stub (todo!()).

`AudioClip` has: data (Vec<f32>), duration, sample_rate, channels.
`AudioClip::silence(duration, sample_rate)` -- creates silence.
`AudioClip::concat(&[AudioClip], sample_rate, channels)` -- concatenates; panics on mismatch.

**Sample rate mismatch**: Kokoro outputs 24kHz. Build pipeline uses `DEFAULT_SAMPLE_RATE` (48kHz). Clips cannot be directly concatenated. Need resampling or use native sample rate.

## Key Constraints

1. **Sample rate**: Kokoro = 24kHz, pipeline = 48kHz. Either resample TTS output to 48kHz or use 24kHz throughout. Simplest: use the TTS backend's native sample rate for the whole audio track.
2. **Immutable M**: `build_video` currently takes `&M`. To call `resolve_narration_durations`, needs `&mut M`. Signature must change.
3. **Optional backend**: No TTS backend = all silence (current behavior). Backend availability is opt-in via BuildConfig.
4. **Ordering**: TTS synthesis must happen before rendering so durations are resolved first.
5. **AudioClip concat requires matching sample rates**: All clips in the final track must have the same sample rate.

## Dependencies

- moron-core depends on moron-voice (already in Cargo.toml)
- `VoiceBackend` trait is in moron-voice::backend
- `AudioClip` is in moron-voice::audio
- Both are re-exported from moron-voice crate root

## Existing Tests

- `build.rs`: Tests for BuildConfig defaults, BuildError variants, BuildResult fields, report callback
- `ffmpeg.rs`: Tests for assemble_audio_track with empty/single/mixed timelines, EncodeConfig, mux_audio validation
- `facade.rs`: Tests for narration_count, resolve_narration_durations, duration resolution
- No integration test that runs the full pipeline (requires Chromium + FFmpeg)

## Files To Modify

1. `moron-core/src/build.rs` -- BuildConfig (add optional backend), build_video signature and flow
2. `moron-core/src/ffmpeg.rs` -- assemble_audio_track signature (accept optional narration clips)
3. `moron-core/src/lib.rs` -- update re-exports if new types are added
