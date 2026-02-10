# T-006-01 Design: Kokoro TTS Backend

## Decision: Use kokoro-tts crate

### Options Evaluated

#### Option A: kokoro-tts crate (v0.3.2)
- Pros: No espeak-ng system dep, built-in G2P, actively maintained, rich Voice enum
- Cons: Async API (needs bridge), uses ort v2 RC, pulls in tokio (already in workspace)
- Risk: ort v2 RC may have breaking changes before stable

#### Option B: kokoroxide crate (v0.1.5)
- Pros: Sync API matches VoiceBackend trait, MIT/Apache-2.0 license
- Cons: Requires espeak-ng system library (complicates builds, CI, cross-compilation), uses older ort 1.16
- Risk: espeak-ng is GPL — license contamination risk. System dep breaks air-gapped first-run.

#### Option C: Direct ort usage (roll our own)
- Pros: Full control, no external crate deps beyond ort
- Cons: Must implement G2P/phonemizer (~1000+ lines), tokenization, voice loading. Violates <15K LOC goal.
- Risk: Massive scope creep for a single ticket.

### Decision: Option A — kokoro-tts

**Rationale**: kokoro-tts eliminates the espeak-ng system dependency (the biggest operational burden), has a comprehensive Voice enum, and produces exactly what we need (Vec<f32> + Duration). The async-to-sync bridge is trivial since tokio is already in the workspace. The ort v2 RC risk is acceptable — it's used in production by the crate author and we pin versions.

### Architecture

```
KokoroBackend
  ├── config: KokoroConfig          (model_path, voices_path, voice, speed)
  ├── engine: Option<KokoroTts>     (lazy-loaded on first synthesize call)
  └── runtime: tokio::Runtime       (for async bridge)
```

### KokoroBackend Design

**Constructor**: `KokoroBackend::new(config: KokoroConfig) -> Self`
- Takes config with paths, does NOT load model yet (fast construction)
- Model loaded lazily on first `synthesize()` call, or eagerly via `load()`

**Config struct**:
```rust
pub struct KokoroConfig {
    pub model_path: PathBuf,     // path to .onnx model
    pub voices_path: PathBuf,    // path to voices.bin
    pub voice: KokoroVoice,      // which voice to use
    pub speed: f32,              // speech speed (1.0 = normal)
}
```

**Voice mapping**: We define our own `KokoroVoice` enum that maps to kokoro-tts Voice variants. Start with a small curated set (e.g., AfHeart, AmAdam, AfSky) rather than exposing all 157.

**Synthesize flow**:
1. Ensure engine is loaded (lazy init)
2. Call `engine.synth(text, voice).await` via tokio Runtime
3. Convert `(Vec<f32>, Duration)` to `AudioClip { data, duration, sample_rate: 24000, channels: 1 }`
4. Return Result

### Error Strategy

Define `KokoroError` (thiserror) for:
- `ModelNotFound` — model_path doesn't exist
- `VoicesNotFound` — voices_path doesn't exist
- `ModelLoadFailed` — ONNX runtime error during load
- `SynthesisFailed` — runtime error during synthesis
- `EmptyText` — empty input text

All converted to `anyhow::Error` at the VoiceBackend boundary.

### Lazy Loading

The ONNX model is ~87MB and takes time to load. We use interior mutability (`OnceCell` or manual Option + init flag) to load on first use. This keeps `KokoroBackend::new()` instant and allows checking config validity before the heavy load.

### Testing Strategy

**Non-ignored tests** (run in CI without model):
- Config construction and validation
- Error when model path doesn't exist
- Error on empty text
- KokoroVoice enum mapping
- Backend name() returns "kokoro"

**Ignored tests** (require model files):
- Actual synthesis produces non-empty AudioClip
- Output sample rate is 24000
- Duration is reasonable for input length
- Multiple sequential syntheses work

### What Was Rejected

- **kokoroxide**: espeak-ng system dependency is a showstopper for air-gapped and CI environments.
- **Direct ort**: Too much scope. G2P alone would exceed the ticket's budget.
- **Changing VoiceBackend to async**: Would cascade changes across the codebase. Not warranted for one backend.
- **Embedding model in binary**: Model is ~87MB. Not viable.
- **Sample rate conversion**: Downstream concern. Store native 24000 Hz.
