# T-006-01 Progress: Kokoro TTS Backend

## Status: COMPLETE

## Completed Steps

### Step 1: Add kokoro-tts dependency to Cargo.toml
- Added `kokoro-tts = { version = "0.3", optional = true }` to moron-voice/Cargo.toml
- Added `kokoro` feature (default enabled) with `dep:kokoro-tts`
- `cargo check -p moron-voice` passes

### Step 2: Implement KokoroError
- Defined `KokoroError` enum with thiserror: ModelNotFound, VoicesNotFound, ModelLoadFailed, SynthesisFailed, EmptyText, RuntimeCreationFailed
- Compiles clean

### Step 3: Implement KokoroConfig
- Defined `KokoroConfig` struct with model_path, voices_path, voice, speed
- Builder pattern: `new()`, `with_voice()`, `with_speed()`
- `validate()` checks paths exist on disk
- Serializable via serde

### Step 4: Implement KokoroVoice enum
- 12 curated voices: AfHeart (default), AfSky, AfBella, AfNova, AfSarah, AmAdam, AmPuck, AmEric, AmMichael, BfEmma, BmGeorge, BmLewis
- Derives: Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize
- Feature-gated mapping to `kokoro_tts::Voice` variants

### Step 5: Implement KokoroBackend
- Struct with config, `Mutex<Option<KokoroTts>>` for lazy loading, tokio Runtime for async bridge
- `new()` creates runtime, does not load model
- `ensure_loaded()` validates paths then loads model via `block_on`
- `synthesize()` checks empty text, ensures loaded, calls `synth()`, returns AudioClip
- Feature-gated: full impl with `kokoro` feature, stub without

### Step 6: Update lib.rs re-exports
- Added cfg-gated re-exports for KokoroConfig, KokoroError, KokoroVoice, KOKORO_SAMPLE_RATE
- KokoroBackend always re-exported (stub when feature disabled)

### Step 7: Write unit tests (non-ignored)
- config_new_sets_defaults, config_builder_methods
- config_validate_missing_model, config_validate_missing_voices
- voice_default_is_af_heart, voice_enum_has_expected_variants, voice_clone_and_copy
- backend_name, synthesize_missing_model_returns_error
- synthesize_empty_text_returns_error, synthesize_whitespace_only_returns_error
- All 27 non-ignored tests pass

### Step 8: Write integration tests (ignored)
- synthesize_produces_audio, synthesize_duration_scales_with_text, synthesize_different_voices
- All 3 gated behind `#[ignore]` with env var instructions

### Step 9: Final verification
- `cargo check` — full workspace passes
- `cargo test` — all 145 tests pass (6 ignored across workspace)
- `cargo clippy -p moron-voice` — zero warnings (fixed derivable_impls lint)

### Step 10: Update ticket frontmatter
- Set status: done, phase: done

## Deviations from Plan

1. **OnceLock -> Mutex<Option<>>**: `OnceLock::get_or_try_init` is unstable (feature `once_cell_try`). Switched to `Mutex<Option<KokoroTts>>` for lazy initialization. Functionally identical but uses stable APIs.

2. **kokoro-tts chosen over kokoroxide**: Research confirmed kokoro-tts (v0.3.2) has a built-in G2P phonemizer (no espeak-ng system dependency), while kokoroxide requires espeak-ng. This aligns better with the air-gapped design principle.

## Files Changed

- `moron-voice/Cargo.toml` — added kokoro-tts dependency and kokoro feature
- `moron-voice/src/kokoro.rs` — complete rewrite (~290 lines)
- `moron-voice/src/lib.rs` — added re-exports
- `docs/active/tickets/T-006-01.md` — status: done, phase: done
