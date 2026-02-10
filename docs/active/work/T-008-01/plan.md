# T-008-01 Plan: AudioClip Error Handling

## Step 1: Add AudioError and update append/concat in audio.rs

- Define `AudioError` enum with `SampleRateMismatch` and `ChannelCountMismatch`
- Change `append()` to return `Result<(), AudioError>`
- Change `concat()` to return `Result<AudioClip, AudioError>`
- Replace `assert_eq!` with `if` + `return Err(...)`
- Update doc comments
- Verify: `cargo check -p moron-voice` (tests won't pass yet)

## Step 2: Update audio.rs tests

- Convert `test_append_panics_on_sample_rate_mismatch` → error assertion
- Convert `test_append_panics_on_channel_mismatch` → error assertion
- Update happy-path tests (`test_append_same_rate`, `test_concat_*`) to unwrap Results
- Verify: `cargo test -p moron-voice`

## Step 3: Export AudioError from moron-voice

- Add `AudioError` to `pub use audio::` in `lib.rs`

## Step 4: Update ffmpeg.rs (assemble_audio_track)

- Import `AudioError`
- Change return type to `Result<AudioClip, AudioError>`
- Add `?` to `AudioClip::concat()` call
- Update 6 tests to unwrap Results
- Verify: `cargo check -p moron-core`

## Step 5: Update build.rs (BuildError propagation)

- Add `Audio(AudioError)` variant to `BuildError` (or `#[from]`)
- Propagate error from `assemble_audio_track()` call
- Add display format and CLI error message for the new variant
- Verify: `cargo test && cargo clippy`
