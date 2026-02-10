# T-008-01 Structure: AudioClip Error Handling

## Files Modified

### `moron-voice/src/audio.rs`
- Add `AudioError` enum (before `AudioClip` impl block)
- Change `append()` signature: `-> Result<(), AudioError>`
- Change `concat()` signature: `-> Result<AudioClip, AudioError>`
- Replace `assert_eq!` with `if` checks + `return Err(...)`
- Update doc comments: remove `# Panics`, add `# Errors`
- Convert 2 `#[should_panic]` tests → assert `Err` variant tests
- Add `?` to `concat`'s internal `append()` call
- Update happy-path concat tests to unwrap Results

### `moron-voice/src/lib.rs`
- Add `AudioError` to the `pub use audio::` re-export line

### `moron-core/src/ffmpeg.rs`
- Change `assemble_audio_track()` return type: `-> Result<AudioClip, AudioError>`
- Add `?` to the `AudioClip::concat()` call
- Update 6 tests to unwrap the Result

### `moron-core/src/build.rs`
- Add `Audio(AudioError)` variant to `BuildError`
- Propagate error from `assemble_audio_track()` with `?` or `.map_err()`
- Add display arm for `BuildError::Audio`

No files created or deleted.
