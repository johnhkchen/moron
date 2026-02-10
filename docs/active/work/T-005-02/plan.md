# T-005-02 Plan: Audio Track Assembly

## Implementation Order

Changes are ordered so that each step compiles and tests independently.
moron-voice changes come first (no dependency on moron-core). moron-core
changes come second (depend on moron-voice).

---

## Step 1: Add f32_to_i16 helper and DEFAULT_SAMPLE_RATE to audio.rs

**File:** `moron-voice/src/audio.rs`

- Add `pub const DEFAULT_SAMPLE_RATE: u32 = 48000;` at module top
- Add private `fn f32_to_i16(sample: f32) -> i16` that clamps to [-1.0, 1.0]
  and scales by 32767.0
- Add test `test_f32_to_i16_conversion` covering 0.0, 1.0, -1.0, >1.0, <-1.0

**Verify:** `cargo test -p moron-voice`

---

## Step 2: Add AudioClip::append method

**File:** `moron-voice/src/audio.rs`

- Add `pub fn append(&mut self, other: &AudioClip)` to `impl AudioClip`
- Panics if sample_rate or channels differ
- Extends `self.data` with `other.data`
- Recomputes `self.duration` as `self.data.len() as f64 / (self.sample_rate as f64 * self.channels as f64)`
- Add test `test_append_same_rate` -- two 1-second clips produce 2-second clip
- Add test `test_append_panics_on_mismatch` -- #[should_panic]

**Verify:** `cargo test -p moron-voice`

---

## Step 3: Add AudioClip::concat associated function

**File:** `moron-voice/src/audio.rs`

- Add `pub fn concat(clips: &[AudioClip], sample_rate: u32, channels: u16) -> AudioClip`
- Creates empty clip, iterates and appends each
- Add test `test_concat_empty` -- returns zero-duration clip
- Add test `test_concat_single` -- matches input clip
- Add test `test_concat_multiple` -- three clips, verify total duration and sample count

**Verify:** `cargo test -p moron-voice`

---

## Step 4: Add AudioClip::to_wav_bytes method

**File:** `moron-voice/src/audio.rs`

- Add `pub fn to_wav_bytes(&self) -> Vec<u8>` to `impl AudioClip`
- Writes 44-byte RIFF header + i16 PCM data
- Uses `f32_to_i16` for sample conversion
- Add test `test_to_wav_bytes_header` -- verify RIFF, WAVE, fmt, data markers,
  sample rate, channels, bits per sample, data size fields
- Add test `test_to_wav_bytes_silence_is_zeros` -- all PCM bytes are zero for silence

**Verify:** `cargo test -p moron-voice`

---

## Step 5: Re-export DEFAULT_SAMPLE_RATE from moron-voice lib.rs

**File:** `moron-voice/src/lib.rs`

- Add `pub use audio::DEFAULT_SAMPLE_RATE;`

**Verify:** `cargo check -p moron-voice`

---

## Step 6: Add assemble_audio_track to ffmpeg.rs

**File:** `moron-core/src/ffmpeg.rs`

- Add `use crate::timeline::Timeline;` and `use moron_voice::{AudioClip, DEFAULT_SAMPLE_RATE};`
- Add `pub fn assemble_audio_track(timeline: &Timeline, sample_rate: u32) -> AudioClip`
- Walks segments, creates silence for each, concatenates
- Add test `test_assemble_empty_timeline`
- Add test `test_assemble_single_segment`
- Add test `test_assemble_mixed_segments`
- Add test `test_assemble_sample_count`

**Verify:** `cargo test -p moron-core`

---

## Step 7: Add mux_audio and build_mux_args to ffmpeg.rs

**File:** `moron-core/src/ffmpeg.rs`

- Add `pub fn mux_audio(video_path: &Path, audio_path: &Path, output_path: &Path) -> Result<(), FfmpegError>`
- Add `fn build_mux_args(video_path: &Path, audio_path: &Path, output_path: &Path) -> Vec<String>`
- Validates inputs exist, detects ffmpeg, spawns process
- Add test `test_build_mux_args` -- verify correct args
- Add test `test_mux_audio_missing_video` -- returns InvalidInput
- Add test `test_mux_audio_missing_audio` -- returns InvalidInput

**Verify:** `cargo test -p moron-core`

---

## Step 8: Add re-exports to moron-core/src/lib.rs

**File:** `moron-core/src/lib.rs`

- Add `mux_audio` and `assemble_audio_track` to the ffmpeg re-export line
- Add same to the prelude module

**Verify:** `cargo check`

---

## Step 9: Full verification

- `cargo check` -- entire workspace compiles
- `cargo test` -- all tests pass (existing + new)
- `cargo clippy` -- no warnings

---

## Step 10: Write progress.md and update ticket frontmatter

- Write `docs/active/work/T-005-02/progress.md`
- Update `docs/active/tickets/T-005-02.md` frontmatter:
  `status: done`, `phase: done`

---

## Testing Strategy

**Unit tests (moron-voice, ~8 tests):**
All audio primitive logic is testable in isolation. Tests verify:
- Sample conversion accuracy (f32 -> i16)
- Append semantics (data growth, duration recomputation)
- Concat behavior (empty, single, multiple)
- WAV encoding correctness (header bytes, PCM payload)

**Unit tests (moron-core, ~7 tests):**
Assembly and mux logic. Tests verify:
- Timeline-to-audio mapping (duration matching)
- FFmpeg argument construction
- Input validation (missing files)

**No integration tests requiring FFmpeg:**
The mux_audio function's FFmpeg invocation is not tested end-to-end
because CI may not have FFmpeg installed. The argument builder and
input validation are tested thoroughly. The detect_ffmpeg test already
handles both installed and not-installed cases gracefully.

---

## Risk Mitigation

1. **WAV header correctness**: Verified byte-by-byte in test against
   the documented 44-byte layout. Each field offset and value is checked.

2. **f32-to-i16 precision**: Edge cases (0.0, +/-1.0, clipping) are
   explicitly tested. The conversion is the only lossy operation.

3. **Backward compatibility**: No existing public APIs are changed.
   All additions are new methods/functions. Existing tests must continue
   to pass unchanged.
