# T-005-02 Structure: Audio Track Assembly

## Overview

Three files are modified. No new files are created. The changes add audio
concatenation and WAV encoding to `AudioClip` in moron-voice, add an audio
assembly function and FFmpeg mux function in moron-core, and wire up
re-exports.

---

## File Changes

### 1. moron-voice/src/audio.rs (MODIFY)

This file currently defines `AudioClip` with `silence()` and `duration()`.
We add four items:

**Constant:**
```
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;
```
Canonical sample rate for video production. Used by the assembly function
in moron-core and available to any caller.

**Method: `AudioClip::append(&mut self, other: &AudioClip)`**

Appends another clip's samples to this clip. Panics if `sample_rate` or
`channels` differ (mismatched clips are a programming error, not a runtime
condition). Updates `self.data` and recomputes `self.duration` from the
new sample count.

Location: inside `impl AudioClip`, after `duration()`.

**Associated function: `AudioClip::concat(clips: &[AudioClip], sample_rate: u32, channels: u16) -> AudioClip`**

Creates a new empty clip and appends each clip from the slice. Returns the
concatenated result. If `clips` is empty, returns a zero-duration clip with
the given sample_rate and channels. This is a convenience wrapper over
`append`.

Location: inside `impl AudioClip`, after `append`.

**Method: `AudioClip::to_wav_bytes(&self) -> Vec<u8>`**

Encodes the clip as a complete WAV file in memory:
- 44-byte RIFF/WAVE header (little-endian)
- PCM format tag 1 (integer)
- 16 bits per sample
- f32 samples are converted to i16 by: clamp to [-1.0, 1.0], multiply
  by 32767.0, round, cast to i16
- All header fields written with `to_le_bytes()`

Returns `Vec<u8>` containing the full WAV. The caller can write this to
a file or pass it to FFmpeg.

Location: inside `impl AudioClip`, after `concat`.

**Helper function: `f32_to_i16(sample: f32) -> i16`**

Private function. Clamps input to [-1.0, 1.0], scales to i16 range.
Used by `to_wav_bytes`.

Location: module-level, private, before `impl AudioClip` or after it.

**Tests added to existing `mod tests`:**

1. `test_append_same_rate` -- append two clips, verify combined length
2. `test_append_panics_on_mismatch` -- #[should_panic] on different sample_rate
3. `test_concat_empty` -- concat of empty slice returns zero-duration clip
4. `test_concat_single` -- concat of one clip equals that clip
5. `test_concat_multiple` -- concat three clips, verify total samples and duration
6. `test_to_wav_bytes_header` -- verify first 44 bytes: RIFF, WAVE, fmt, data markers,
   sample rate, bits per sample, channels, data size
7. `test_to_wav_bytes_silence_is_zeros` -- silence clip produces all-zero PCM payload
8. `test_f32_to_i16_conversion` -- 0.0 -> 0, 1.0 -> 32767, -1.0 -> -32767,
   values beyond range are clamped

---

### 2. moron-core/src/ffmpeg.rs (MODIFY)

Two additions to the existing FFmpeg module:

**Function: `mux_audio(video_path: &Path, audio_path: &Path, output_path: &Path) -> Result<(), FfmpegError>`**

Muxes a video file with an audio file into a final .mp4:
- Calls `detect_ffmpeg()` first
- Validates that video_path and audio_path exist
- Builds FFmpeg args: `ffmpeg -y -i {video} -i {audio} -c:v copy -c:a aac -shortest {output}`
- Spawns FFmpeg, captures stderr
- Returns `Ok(())` or `FfmpegError::EncodeFailed`

Location: after the `encode()` function, before the internal helpers section.

**Function: `assemble_audio_track(timeline: &Timeline, sample_rate: u32) -> AudioClip`**

Walks `timeline.segments()`, creates an `AudioClip::silence(seg.duration(), sample_rate)`
for each segment, collects into a Vec, and calls `AudioClip::concat()`.

This is the bridge between the timeline (moron-core type) and audio
primitives (moron-voice type). It lives in ffmpeg.rs because it is part
of the FFmpeg/muxing pipeline, not general-purpose audio logic.

Location: after `mux_audio`, before the internal helpers section.

**Internal helper: `build_mux_args(video_path, audio_path, output_path) -> Vec<String>`**

Builds the FFmpeg argument list for muxing. Follows the pattern of
the existing `build_ffmpeg_args`.

**Tests added to existing `mod tests`:**

1. `test_assemble_empty_timeline` -- empty timeline produces zero-duration clip
2. `test_assemble_single_segment` -- one narration segment, correct duration
3. `test_assemble_mixed_segments` -- all four segment types, total duration matches
4. `test_assemble_sample_count` -- verify exact sample count matches expected
5. `test_build_mux_args` -- verify correct FFmpeg args for muxing
6. `test_mux_audio_missing_video` -- returns InvalidInput when video doesn't exist
7. `test_mux_audio_missing_audio` -- returns InvalidInput when audio doesn't exist

---

### 3. moron-core/src/lib.rs (MODIFY)

Add re-exports for the new public items:

```rust
pub use ffmpeg::{mux_audio, assemble_audio_track};
```

Add to prelude as well:

```rust
pub use crate::ffmpeg::{mux_audio, assemble_audio_track};
```

---

## Module Boundaries

```
moron-voice/src/audio.rs
  - Owns: AudioClip struct, concat, append, to_wav_bytes, f32_to_i16
  - Exports: AudioClip, DEFAULT_SAMPLE_RATE (via moron-voice crate root)

moron-core/src/ffmpeg.rs
  - Owns: FFmpeg interaction, EncodeConfig, encode, mux_audio, assemble_audio_track
  - Uses: Timeline from timeline.rs, AudioClip from moron-voice
  - Exports: all public functions via moron-core crate root

moron-core/src/timeline.rs
  - Read-only. No changes.
```

---

## Public API Surface (new items)

| Crate | Item | Signature |
|-------|------|-----------|
| moron-voice | `DEFAULT_SAMPLE_RATE` | `pub const u32 = 48000` |
| moron-voice | `AudioClip::append` | `(&mut self, other: &AudioClip)` |
| moron-voice | `AudioClip::concat` | `(clips: &[AudioClip], sample_rate: u32, channels: u16) -> AudioClip` |
| moron-voice | `AudioClip::to_wav_bytes` | `(&self) -> Vec<u8>` |
| moron-core | `assemble_audio_track` | `(timeline: &Timeline, sample_rate: u32) -> AudioClip` |
| moron-core | `mux_audio` | `(video: &Path, audio: &Path, output: &Path) -> Result<(), FfmpegError>` |

---

## Dependency Flow

```
moron-core/src/ffmpeg.rs
    |
    +--uses--> moron_voice::AudioClip         (already a dependency)
    +--uses--> moron_voice::DEFAULT_SAMPLE_RATE
    +--uses--> crate::timeline::{Timeline, Segment}  (same crate)
```

No new crate dependencies. No Cargo.toml changes.

---

## Files NOT Changed

- moron-voice/src/lib.rs -- already re-exports `AudioClip`; `DEFAULT_SAMPLE_RATE`
  will be re-exported here
- moron-voice/Cargo.toml -- no new dependencies
- moron-core/Cargo.toml -- no new dependencies
- moron-core/src/timeline.rs -- read-only
- moron-core/src/renderer.rs -- not touched in this ticket
- moron-core/src/facade.rs -- not touched in this ticket
