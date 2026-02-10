# T-008-01 Design: AudioClip Error Handling

## Decision

Create an `AudioError` enum in `audio.rs` with two variants, change `append` → `Result<(), AudioError>` and `concat` → `Result<AudioClip, AudioError>`. Update call sites to propagate errors.

## AudioError Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("sample rate mismatch: expected {expected}, got {got}")]
    SampleRateMismatch { expected: u32, got: u32 },

    #[error("channel count mismatch: expected {expected}, got {got}")]
    ChannelCountMismatch { expected: u16, got: u16 },
}
```

Follows the codebase pattern of `thiserror`-derived error types with descriptive `#[error]` messages.

## Call Site Changes

1. **`concat()` in `audio.rs`** — propagate with `?` from `append()`
2. **`assemble_audio_track()` in `ffmpeg.rs`** — return `Result<AudioClip, AudioError>` instead of bare `AudioClip`
3. **`build.rs`** — add `AudioError` variant to `BuildError`, propagate with `?`

## Alternatives Rejected

- **Keep panics, add `try_*` variants**: Doubles the API surface for no benefit. Callers should always use the fallible version.
- **Use `anyhow::Error` instead of a typed enum**: Inconsistent with the rest of the codebase which uses typed errors everywhere.
