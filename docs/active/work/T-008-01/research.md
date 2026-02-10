# T-008-01 Research: AudioClip Error Handling

## Current Behavior

`moron-voice/src/audio.rs` defines `AudioClip` with two methods that panic on invalid input:

### `append(&mut self, other: &AudioClip)` (line 49)
- `assert_eq!` on `sample_rate` mismatch → panic with "cannot append clips with different sample rates"
- `assert_eq!` on `channels` mismatch → panic with "cannot append clips with different channel counts"

### `concat(clips: &[AudioClip], sample_rate: u32, channels: u16) -> AudioClip` (line 74)
- Delegates to `append()` in a loop, so inherits the same panics
- Empty input is fine (returns zero-duration clip)

Both document panics via `# Panics` doc comments.

## Call Sites

### Production
1. **`AudioClip::concat()` calls `self.append()`** — internal, same file (line 82)
2. **`moron-core/src/ffmpeg.rs:313`** — `assemble_audio_track()` calls `AudioClip::concat(&clips, sample_rate, 1)`
   - Return type: `AudioClip` (no error handling)
   - Called from `build.rs:283` which stores result directly
3. **`moron-core/src/build.rs:283`** — `let audio_clip = ffmpeg::assemble_audio_track(...)`
   - No `?` or error propagation currently

### Tests
- `audio.rs`: 2 `#[should_panic]` tests (lines 181-197), plus 3 happy-path tests for concat
- `ffmpeg.rs`: 6 tests call `assemble_audio_track()`, all use matching sample rates (no mismatch)

## Exports

`moron-voice/src/lib.rs` exports: `AudioClip`, `DEFAULT_SAMPLE_RATE`. No `AudioError` exists yet.

## Error Pattern in Codebase

- `moron-core` uses `BuildError`, `RenderError`, `FfmpegError` — all `thiserror`-derived enums
- `moron-voice` uses `KokoroError` — also `thiserror`-derived
- Convention: error enums with `#[derive(Debug, thiserror::Error)]` and `#[error("...")]` display messages

## Dependencies

`moron-voice/Cargo.toml` already depends on `thiserror` (used by `KokoroError`).
