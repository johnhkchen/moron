# T-005-02 Progress: Audio Track Assembly

## Status: COMPLETE

All implementation steps have been executed and verified.

---

## Steps Completed

### Step 1: f32_to_i16 helper and DEFAULT_SAMPLE_RATE
- Added `pub const DEFAULT_SAMPLE_RATE: u32 = 48000` to `moron-voice/src/audio.rs`
- Added private `fn f32_to_i16(sample: f32) -> i16` with clamping
- Added `test_f32_to_i16_conversion` covering 0.0, 1.0, -1.0, >1.0, <-1.0, 0.5
- Verified: `cargo test -p moron-voice` -- all pass

### Step 2: AudioClip::append
- Added `pub fn append(&mut self, other: &AudioClip)` with sample_rate/channels assertions
- Recomputes duration from data length after appending
- Added `test_append_same_rate` and two `#[should_panic]` tests for mismatch cases
- Verified: `cargo test -p moron-voice` -- all pass

### Step 3: AudioClip::concat
- Added `pub fn concat(clips: &[AudioClip], sample_rate: u32, channels: u16) -> AudioClip`
- Delegates to append in a loop; empty input returns zero-duration clip
- Added `test_concat_empty`, `test_concat_single`, `test_concat_multiple`
- Verified: `cargo test -p moron-voice` -- all pass

### Step 4: AudioClip::to_wav_bytes
- Added `pub fn to_wav_bytes(&self) -> Vec<u8>` producing a complete 44-byte RIFF header + i16 PCM
- Uses f32_to_i16 for sample conversion; all header fields little-endian
- Added `test_to_wav_bytes_header` (byte-by-byte verification of all 44 header bytes)
- Added `test_to_wav_bytes_silence_is_zeros` and `test_to_wav_bytes_with_signal`
- Verified: `cargo test -p moron-voice` -- all pass

### Step 5: Re-export DEFAULT_SAMPLE_RATE
- Updated `moron-voice/src/lib.rs`: `pub use audio::{AudioClip, DEFAULT_SAMPLE_RATE};`
- Verified: `cargo check -p moron-voice`

### Step 6: assemble_audio_track
- Added to `moron-core/src/ffmpeg.rs`
- Walks `timeline.segments()`, creates silence per segment, concatenates via `AudioClip::concat`
- Added 4 tests: empty timeline, single segment, mixed segments, sample count verification
- Verified: `cargo test -p moron-core` -- all pass

### Step 7: mux_audio and build_mux_args
- Added `pub fn mux_audio(video_path, audio_path, output_path) -> Result<(), FfmpegError>`
- Validates both input files exist, detects FFmpeg, spawns process with muxing args
- Added `fn build_mux_args` producing: `ffmpeg -y -i video -i audio -c:v copy -c:a aac -shortest output`
- Added 3 tests: build_mux_args verification, missing video validation, missing audio validation
- Verified: `cargo test -p moron-core` -- all pass

### Step 8: Re-exports in moron-core/src/lib.rs
- Added `assemble_audio_track` and `mux_audio` to both the crate root re-exports and the prelude
- Verified: `cargo check`

### Step 9: Full verification
- `cargo check` -- clean (0 warnings)
- `cargo test` -- 119 tests pass (0 failures)
- `cargo clippy` -- clean (0 warnings)

---

## Deviations from Plan

None. All steps executed as planned.

---

## Test Summary

### New tests in moron-voice/src/audio.rs (8 tests)
- `test_f32_to_i16_conversion`
- `test_append_same_rate`
- `test_append_panics_on_sample_rate_mismatch`
- `test_append_panics_on_channel_mismatch`
- `test_concat_empty`
- `test_concat_single`
- `test_concat_multiple`
- `test_to_wav_bytes_header`
- `test_to_wav_bytes_silence_is_zeros`
- `test_to_wav_bytes_with_signal`
- `test_default_sample_rate`

### New tests in moron-core/src/ffmpeg.rs (7 tests)
- `test_assemble_empty_timeline`
- `test_assemble_single_segment`
- `test_assemble_mixed_segments`
- `test_assemble_sample_count`
- `test_build_mux_args`
- `test_mux_audio_missing_video`
- `test_mux_audio_missing_audio`

---

## Files Modified

| File | Change |
|------|--------|
| `moron-voice/src/audio.rs` | Added DEFAULT_SAMPLE_RATE, f32_to_i16, append, concat, to_wav_bytes + 11 tests |
| `moron-voice/src/lib.rs` | Added DEFAULT_SAMPLE_RATE to re-exports |
| `moron-core/src/ffmpeg.rs` | Added assemble_audio_track, mux_audio, build_mux_args + 7 tests |
| `moron-core/src/lib.rs` | Added assemble_audio_track, mux_audio to re-exports and prelude |
