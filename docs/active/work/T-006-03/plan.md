# T-006-03 Plan: Pipeline Audio Integration

## Step 1: Update assemble_audio_track signature (ffmpeg.rs)

- Add `narration_clips: Option<&[AudioClip]>` parameter
- Implement logic: use real clip for Narration segments when available, silence otherwise
- Update existing call site in build.rs to pass `None` (preserves current behavior)
- Update all existing tests to pass `None` as third argument
- Add new tests: assemble with real narration clips, mixed segments with clips

**Verify**: `cargo check && cargo test`

## Step 2: Add TTS-related types to build.rs

- Add `BuildError::Tts { segment: usize, source: anyhow::Error }`
- Add `BuildProgress::SynthesizingTts { current: usize, total: usize }`
- Add `voice_backend: Option<Arc<dyn moron_voice::VoiceBackend + Send + Sync>>` to BuildConfig
- Initialize voice_backend to None in BuildConfig::new()
- Update build_config_defaults test

**Verify**: `cargo check && cargo test`

## Step 3: Implement synthesize_narrations helper (build.rs)

- Add internal function `synthesize_narrations`
- Extracts narration texts from timeline segments using narration_indices
- Calls backend.synthesize(text) for each narration
- Reports SynthesizingTts progress
- Collects durations and calls m.resolve_narration_durations
- Returns Vec<AudioClip>
- Add unit tests with a simple mock VoiceBackend

**Verify**: `cargo check && cargo test`

## Step 4: Wire TTS into build_video pipeline (build.rs)

- Change build_video signature: `&M` -> `&mut M`
- Add TTS synthesis step before the "report scene stats" step
- Move timeline stats extraction after TTS synthesis (durations may change)
- Determine sample rate from narration clips or use DEFAULT_SAMPLE_RATE
- Pass narration_clips to assemble_audio_track
- Update the build_video doc comment

**Verify**: `cargo check && cargo test`

## Step 5: Update downstream callers and re-exports

- Check if any code calls build_video with &M and update to &mut M
- Check lib.rs re-exports still work
- Run full workspace check and test

**Verify**: `cargo check && cargo test` (workspace-wide)

## Step 6: Update ticket frontmatter

- Set status: done, phase: done in T-006-03.md

## Verification Criteria

- `cargo check` passes (no compilation errors)
- `cargo test` passes (all existing + new tests)
- `cargo clippy` passes (no warnings)
- When voice_backend is None, behavior is identical to before
- When voice_backend is Some, narration clips are synthesized and wired into audio track
- Audio track duration matches timeline duration
