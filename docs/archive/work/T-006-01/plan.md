# T-006-01 Plan: Kokoro TTS Backend

## Step 1: Add kokoro-tts dependency to Cargo.toml

**Changes**: moron-voice/Cargo.toml
- Add `kokoro-tts` as optional dependency
- Add `kokoro` feature (default enabled)

**Verify**: `cargo check -p moron-voice` compiles (may take time for ort download)

## Step 2: Implement KokoroError

**Changes**: moron-voice/src/kokoro.rs
- Define `KokoroError` enum with thiserror derives
- Variants: ModelNotFound, VoicesNotFound, ModelLoadFailed, SynthesisFailed, EmptyText

**Verify**: `cargo check -p moron-voice`

## Step 3: Implement KokoroConfig

**Changes**: moron-voice/src/kokoro.rs
- Define `KokoroConfig` struct with model_path, voices_path, voice, speed
- Implement `new()`, `with_voice()`, `with_speed()`, `validate()`

**Verify**: `cargo check -p moron-voice`

## Step 4: Implement KokoroVoice enum

**Changes**: moron-voice/src/kokoro.rs
- Define curated `KokoroVoice` enum (AfHeart, AfSky, AmAdam, AmPuck, etc.)
- Implement mapping to `kokoro_tts::Voice`
- Default voice: AfHeart

**Verify**: `cargo check -p moron-voice`

## Step 5: Implement KokoroBackend

**Changes**: moron-voice/src/kokoro.rs
- Define `KokoroBackend` struct with config, OnceLock<KokoroTts>, Runtime
- Implement `new()` — validates config, creates tokio runtime
- Implement `ensure_loaded()` — lazy model loading via OnceLock
- Implement `VoiceBackend` trait — synthesize() bridges async to sync, converts output to AudioClip

**Verify**: `cargo check -p moron-voice`

## Step 6: Update lib.rs re-exports

**Changes**: moron-voice/src/lib.rs
- Add cfg-gated re-exports for KokoroBackend, KokoroConfig, KokoroVoice, KokoroError

**Verify**: `cargo check -p moron-voice`

## Step 7: Write unit tests (non-ignored)

**Changes**: moron-voice/src/kokoro.rs (test module)
- Test: config construction with valid paths
- Test: config validate catches missing model path
- Test: config validate catches missing voices path
- Test: voice enum has expected variants and Default
- Test: backend name() returns "kokoro"
- Test: synthesize returns error for missing model
- Test: synthesize returns error for empty text

**Verify**: `cargo test -p moron-voice`

## Step 8: Write integration tests (ignored)

**Changes**: moron-voice/src/kokoro.rs (test module, #[ignore])
- Test: synthesize produces non-empty AudioClip
- Test: output sample rate is 24000
- Test: output is mono (channels == 1)
- Test: duration is proportional to text length

**Verify**: `cargo test -p moron-voice` (ignored tests skipped)

## Step 9: Final verification

- `cargo check` (full workspace)
- `cargo test` (full workspace, non-ignored)
- `cargo clippy -p moron-voice` (lint)

## Step 10: Update ticket frontmatter

**Changes**: docs/active/tickets/T-006-01.md
- Set status: done, phase: done
