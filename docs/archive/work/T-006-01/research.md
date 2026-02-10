# T-006-01 Research: Kokoro TTS Backend

## Current Codebase State

### moron-voice module structure
- `lib.rs` — re-exports `KokoroBackend`, `PiperBackend`, `AudioClip`, `Voice`, `VoiceBackend`, `VoiceBackendType`
- `backend.rs` — `VoiceBackend` trait (`synthesize(&self, text) -> Result<AudioClip>`, `name() -> &str`), `VoiceBackendType` enum (Kokoro, Piper, ApiProvider, PreRecorded), `Voice` config struct
- `audio.rs` — `AudioClip` struct (data: `Vec<f32>`, duration: f64, sample_rate: u32, channels: u16), `DEFAULT_SAMPLE_RATE` = 48000, plus `silence()`, `append()`, `concat()`, `to_wav_bytes()` helpers
- `kokoro.rs` — stub `KokoroBackend` unit struct, `VoiceBackend` impl with `todo!()` in synthesize
- `piper.rs` — stub `PiperBackend`, same pattern
- `alignment.rs` — empty module

### VoiceBackend trait
```rust
pub trait VoiceBackend {
    fn synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error>;
    fn name(&self) -> &str;
}
```
Synchronous interface. Returns `anyhow::Error`. No async.

### AudioClip
```rust
pub struct AudioClip {
    pub data: Vec<f32>,       // f32 PCM [-1.0, 1.0]
    pub duration: f64,        // seconds
    pub sample_rate: u32,     // Hz
    pub channels: u16,        // 1=mono, 2=stereo
}
```
Kokoro models output at 24000 Hz mono. `DEFAULT_SAMPLE_RATE` is 48000 (broadcast). Sample rate conversion may be needed downstream but is NOT this ticket's concern — we store the native rate.

### Current dependencies (moron-voice/Cargo.toml)
serde, tokio, anyhow, thiserror. No ONNX or TTS deps yet.

### Workspace Cargo.toml
edition = "2024", resolver = "3". Workspace deps include tokio with "full" features.

## Available Kokoro TTS Rust Crates

### kokoro-tts (crates.io, v0.3.2)
- **Source**: Kokoros project (lucasjinreal/Kokoros)
- **API**: Async — `KokoroTts::new(model_path, voices_path).await`, `tts.synth(text, voice).await -> Result<(Vec<f32>, Duration), KokoroError>`
- **Output**: `Vec<f32>` samples + `Duration`, sample rate 24000 Hz
- **Voice enum**: 157 variants (AfHeart, AmAdam, etc.)
- **Dependencies**: ort 2.0.0-rc.x, ndarray, tokio, bincode
- **Phonemizer**: Built-in G2P (no espeak-ng required)
- **License**: unclear (Kokoros repo doesn't state clearly)
- **Concern**: Async API while VoiceBackend is sync. Uses `ort` v2 RC.
- **Model files**: ONNX model (~87MB) + voices.bin from HuggingFace

### kokoroxide (crates.io, v0.1.5)
- **Source**: dhruv304c2/kokoroxide
- **API**: Sync — `KokoroTTS::with_config(config)`, `tts.generate_speech(text, &voice, speed) -> Result<GeneratedAudio>`
- **Output**: `GeneratedAudio` with samples, duration, sample_rate
- **Dependencies**: ort 1.16, ndarray 0.15, hound, espeak-ng (system dep!)
- **Phonemizer**: Requires espeak-ng system library
- **License**: MIT/Apache-2.0
- **Concern**: espeak-ng system dependency complicates builds and CI.
- **Model files**: ONNX model + tokenizer.json + voice .bin files

### kokorox (WismutHansen/kokorox)
- **Not a library crate** — CLI application only
- GPL 3.0 due to espeak-ng static linking
- Not usable as a dependency

## Model Files

All Kokoro implementations need:
1. ONNX model file (~87-100MB) — from `huggingface.co/hexgrad/Kokoro-82M` or `huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX`
2. Voice style data — voices.bin or individual .bin files
3. Tokenizer config — tokenizer.json (for some implementations)

Files must be downloaded once, cached locally. NOT checked into repo.

## Key Constraints

1. **Sync trait vs async crates**: VoiceBackend::synthesize is sync. kokoro-tts is async. We'd need `tokio::Runtime::block_on()` to bridge, or use kokoroxide which is sync.
2. **espeak-ng dependency**: kokoroxide requires espeak-ng system library. kokoro-tts has built-in G2P.
3. **ort version**: kokoroxide uses ort 1.16 (stable). kokoro-tts uses ort 2.0 RC.
4. **Air-gapped**: After model download, everything must work offline. Both crates satisfy this.
5. **CI testing**: Neither model files nor espeak-ng can be assumed in CI. Tests needing the model must be #[ignore].
6. **edition 2024**: Workspace uses edition 2024. Dependency crates must be compatible.
7. **Solo maintainable**: Minimize complexity. Fewer system deps = better.

## Sample Rate Consideration

Kokoro models natively produce 24000 Hz audio. Our AudioClip stores the actual sample rate. Downstream consumers can resample to 48000 Hz if needed. This ticket stores native 24000 Hz.

## Error Scenarios to Handle

- Model file not found at path
- Voice file not found at path
- Invalid/corrupt model file
- Empty text input
- Very long text input (model may have token limits)
- ONNX runtime initialization failure
