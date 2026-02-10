# T-006-03 Progress: Pipeline Audio Integration

## Status: Complete

## Completed Steps

### Step 1: Update assemble_audio_track signature (ffmpeg.rs)
- Added `narration_clips: Option<&[AudioClip]>` parameter to `assemble_audio_track`
- Implemented logic: use real clip for Narration segments when clips provided, silence otherwise
- Updated call site in build.rs to pass `None`
- Updated 2 call sites in e2e.rs to pass `None`
- Updated all 4 existing assemble tests to pass `None`
- Added 2 new tests: `test_assemble_with_narration_clips`, `test_assemble_narration_clips_none_fallback`
- Fixed clippy warning: collapsed nested if into `if let ... &&` form
- Verified: `cargo check && cargo test` pass

### Step 2: Add TTS-related types to build.rs
- Added `BuildError::Tts { segment, source }` variant
- Added `BuildProgress::SynthesizingTts { current, total }` variant
- Added `voice_backend: Option<Arc<dyn VoiceBackend + Send + Sync>>` to BuildConfig
- Initialized to `None` in `BuildConfig::new()`
- Updated `build_config_defaults` test to assert `voice_backend.is_none()`
- Added `build_error_display_tts` test
- Verified: `cargo check && cargo test` pass

### Step 3: Implement synthesize_narrations helper (build.rs)
- Added `synthesize_narrations(m, backend, progress)` function
- Extracts narration texts from timeline via `narration_indices()`
- Calls `backend.synthesize(text)` for each, collects clips and durations
- Reports `SynthesizingTts` progress per segment
- Calls `m.resolve_narration_durations(&durations)` to update timeline
- Returns `Vec<AudioClip>` on success, `BuildError::Tts` on failure
- Added MockBackend and FailingBackend test doubles
- Added 4 tests: resolves_durations, no_narrations, reports_progress, propagates_error
- Verified: `cargo check && cargo test` pass

### Step 4: Wire TTS into build_video pipeline (build.rs)
- Changed `build_video` signature from `&M` to `&mut M`
- Added TTS synthesis step before timeline stats computation
- Moved timeline stats after TTS synthesis (durations may change)
- Determined sample rate from TTS clips or DEFAULT_SAMPLE_RATE
- Passed narration_clips to assemble_audio_track
- Updated doc comments to reflect new pipeline flow
- Verified: `cargo check && cargo test` pass

### Step 5: Update downstream callers and re-exports
- Updated `moron-core/tests/e2e.rs`: `&m` -> `&mut m` for build_video call
- Updated `moron-cli/src/main.rs`:
  - Added `voice_backend: None` to BuildConfig construction
  - Changed `&m` -> `&mut m` for build_video call
  - Added `SynthesizingTts` match arm to progress callback
  - Added `BuildError::Tts` match arm to `format_build_error`
- Verified: `cargo check && cargo test && cargo clippy` all pass clean

### Step 6: Update ticket frontmatter
- Set status: done, phase: done in T-006-03.md

## Test Results
- 105 unit tests passing (up from 100)
- 4 e2e tests passing (3 ignored, require Chrome/FFmpeg)
- 5 integration tests passing
- 0 clippy warnings

## New Tests Added (5 total)
1. `build::tests::build_error_display_tts` -- TTS error formatting
2. `build::tests::synthesize_narrations_resolves_durations` -- mock TTS resolves durations
3. `build::tests::synthesize_narrations_no_narrations` -- no-op when no narrations
4. `build::tests::synthesize_narrations_reports_progress` -- progress callback invoked
5. `build::tests::synthesize_narrations_propagates_error` -- TTS failure becomes BuildError
6. `ffmpeg::tests::test_assemble_with_narration_clips` -- real clips wired into audio track
7. `ffmpeg::tests::test_assemble_narration_clips_none_fallback` -- None = silence

## Deviations from Plan
None. All steps executed as planned.
