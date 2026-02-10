# T-006-01 Structure: Kokoro TTS Backend

## Files Modified

### moron-voice/Cargo.toml
Add dependencies:
- `kokoro-tts = "0.3"` — Kokoro TTS engine
- Feature-gate behind `kokoro` feature (default enabled) so the heavy ONNX dep is optional

```toml
[features]
default = ["kokoro"]
kokoro = ["dep:kokoro-tts"]

[dependencies]
kokoro-tts = { version = "0.3", optional = true }
```

### moron-voice/src/kokoro.rs
**Complete rewrite.** Replace stub with full implementation.

Public API:
```
KokoroConfig         — configuration struct
KokoroVoice          — voice selection enum
KokoroBackend        — VoiceBackend implementor
KokoroError          — error types (thiserror)
```

Module-internal:
- `ensure_loaded()` — lazy model loading
- Voice mapping from KokoroVoice -> kokoro_tts::Voice

Structure:
```rust
// --- Errors ---
#[derive(Debug, thiserror::Error)]
pub enum KokoroError { ... }

// --- Config ---
pub struct KokoroConfig { ... }
impl KokoroConfig {
    pub fn new(model_path, voices_path) -> Self;
    pub fn with_voice(self, voice) -> Self;
    pub fn with_speed(self, speed) -> Self;
    fn validate(&self) -> Result<(), KokoroError>;
}

// --- Voice ---
pub enum KokoroVoice { ... }
impl KokoroVoice {
    fn to_kokoro_tts_voice(&self) -> kokoro_tts::Voice;
}

// --- Backend ---
pub struct KokoroBackend {
    config: KokoroConfig,
    engine: std::sync::OnceLock<kokoro_tts::KokoroTts>,
    runtime: tokio::runtime::Runtime,
}
impl KokoroBackend {
    pub fn new(config: KokoroConfig) -> Result<Self, KokoroError>;
    fn ensure_loaded(&self) -> Result<&kokoro_tts::KokoroTts, KokoroError>;
}
impl VoiceBackend for KokoroBackend { ... }
```

### moron-voice/src/lib.rs
Update re-exports to include new public types:
```rust
pub use kokoro::{KokoroBackend, KokoroConfig, KokoroVoice, KokoroError};
```
Gate behind `#[cfg(feature = "kokoro")]`.

### moron-voice/src/backend.rs
No changes needed. VoiceBackend trait is stable.

### moron-voice/src/audio.rs
No changes needed. AudioClip is stable.

## Files NOT Modified

- `moron-voice/src/piper.rs` — separate ticket
- `moron-voice/src/alignment.rs` — separate ticket
- Workspace Cargo.toml — no workspace-level dep needed (kokoro-tts is local to moron-voice)
- Any other crate — no cross-crate changes

## Module Boundaries

```
moron-voice/src/
├── lib.rs              ← re-exports (gated)
├── audio.rs            ← AudioClip (unchanged)
├── backend.rs          ← VoiceBackend trait (unchanged)
├── kokoro.rs           ← KokoroBackend (REWRITTEN)
├── piper.rs            ← PiperBackend stub (unchanged)
└── alignment.rs        ← empty (unchanged)
```

## Public Interface

After this change, consumers can:
```rust
use moron_voice::{KokoroBackend, KokoroConfig, KokoroVoice, VoiceBackend};

let config = KokoroConfig::new("models/kokoro.onnx", "models/voices.bin")
    .with_voice(KokoroVoice::AfHeart)
    .with_speed(1.0);
let backend = KokoroBackend::new(config)?;
let clip = backend.synthesize("Hello, world!")?;
// clip.sample_rate == 24000, clip.channels == 1
```

## Conditional Compilation

When the `kokoro` feature is disabled:
- `kokoro-tts` dep is not compiled
- `KokoroBackend` and related types are not available
- `lib.rs` does not re-export kokoro types
- The `kokoro` module is still compiled but contains a minimal stub

This allows the workspace to build without ONNX runtime overhead when only working on other crates.

## Size Estimate

- kokoro.rs: ~200-250 lines (config, error, voice enum, backend impl, tests)
- Cargo.toml changes: ~5 lines
- lib.rs changes: ~5 lines
- Total new/modified code: ~210-260 lines
