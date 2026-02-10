# T-006-04 Structure: TTS Validation Tests

## Files Modified

### moron-core/tests/e2e.rs (extend)
- Add a `MockTtsBackend` at the helper section (similar to build.rs MockBackend but usable in integration tests).
- Add non-ignored test: `e2e_mock_tts_duration_resolution`.
- Add non-ignored test: `e2e_audio_assembly_with_tts_clips`.
- Add ignored test: `e2e_full_pipeline_with_tts` (requires Kokoro + optionally FFmpeg).
- Update module doc comment to describe TTS test running instructions.

### moron-voice/tests/tts.rs (new)
- Module doc comment with running instructions.
- Helper: `kokoro_config() -> Option<KokoroConfig>` reading env vars.
- Ignored test: `kokoro_synthesis_produces_valid_audio`.
- Ignored test: `kokoro_wav_encoding_roundtrip`.

### docs/active/tickets/T-006-04.md (update frontmatter)
- status: done
- phase: done

## Files Not Modified
- No changes to source code (src/) -- this ticket adds only tests.
- No changes to Cargo.toml files -- existing dependencies are sufficient.

## Module Boundaries
- moron-voice/tests/tts.rs tests the VoiceBackend trait and KokoroBackend through public API only.
- moron-core/tests/e2e.rs tests the pipeline integration through the prelude public API.
- MockTtsBackend in e2e.rs implements `moron_voice::VoiceBackend` directly (the trait is public).

## Public API Dependencies
- `moron_voice::{VoiceBackend, AudioClip, DEFAULT_SAMPLE_RATE}` -- used by both test files.
- `moron_voice::{KokoroBackend, KokoroConfig, KokoroVoice, KOKORO_SAMPLE_RATE}` -- used by tts.rs (feature-gated).
- `moron_core::prelude::*` -- used by e2e.rs (provides M, DemoScene, assemble_audio_track, etc.).
