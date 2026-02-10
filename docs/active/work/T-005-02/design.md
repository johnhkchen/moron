# T-005-02 Design: Audio Track Assembly

## Problem

Given a populated `Timeline` of segments, produce a single audio file (WAV)
and have FFmpeg mux it with the rendered video into a final .mp4 that contains
both video and audio streams. For now all segments produce silence; the
architecture must support real TTS audio in the future.

---

## Decision: Assembly Architecture

### Option A: Assembly function in moron-core

A function `assemble_audio(timeline: &Timeline) -> AudioClip` lives in
moron-core (e.g., in a new `audio_assembly.rs` or inside `ffmpeg.rs`).
It directly accesses `Timeline` and `Segment` types.

- Pro: Direct access to Timeline types, close to FFmpeg pipeline
- Con: Audio logic in the video/rendering crate, muddies crate boundaries

### Option B: Assembly function in moron-voice

A function `assemble_audio_track(segments: &[(SegmentKind, f64)]) -> AudioClip`
lives in moron-voice. It takes abstract segment descriptions (not the Timeline
type directly) to avoid a reverse dependency.

- Pro: Audio logic stays in the audio crate
- Con: Requires a bridging type or conversion at the call site; adds indirection

### Option C: Assembly function in moron-voice, accepting durations only

Since every segment currently produces silence, the assembler only needs a
total duration: `AudioClip::silence(total_duration, sample_rate)`. The
per-segment walk becomes relevant only when TTS produces real clips. We can
add `concat` to AudioClip and keep the segment-walk logic in moron-core
where it has access to Timeline.

- Pro: Minimal API surface now; concat utility is reusable
- Con: Splits the assembly across two crates

**Decision: Option C.** Add `concat` and `to_wav_bytes` to `AudioClip` in
moron-voice. The segment-walking logic lives in moron-core (which already
depends on moron-voice) as a function that builds an `AudioClip` per segment
and concatenates them. This keeps audio primitives in moron-voice and
pipeline orchestration in moron-core.

When real TTS arrives, the segment walker will call `VoiceBackend::synthesize`
for Narration segments instead of `AudioClip::silence`. The concat and WAV
encoding remain unchanged.

---

## Decision: WAV Encoding Approach

### Option A: Use the `hound` crate

A well-maintained Rust crate for reading/writing WAV files.

- Pro: Battle-tested, handles edge cases, supports f32/i16/i24
- Con: New dependency for a 44-byte header

### Option B: Hand-roll WAV encoding

WAV PCM is a trivial format: 44-byte RIFF header + raw PCM data.

- Pro: Zero dependencies, ~40 lines of code, easy to audit
- Con: Must handle endianness correctly; limited to our specific use case

### Option C: Write raw PCM, let FFmpeg handle format

Write raw f32 samples to a file; tell FFmpeg the format via command-line args
(`-f f32le -ar 48000 -ac 1`).

- Pro: No WAV encoding at all
- Con: Fragile; FFmpeg args must match exactly; less portable

**Decision: Option B (hand-rolled WAV).** The WAV header is 44 bytes of
well-documented structure. For our use case (16-bit signed PCM, mono or
stereo, known sample rate) it is trivial. This avoids adding a dependency
for something we can write in ~40 lines with full test coverage. If we
later need complex WAV features (24-bit, metadata chunks), we revisit.

The encoding will produce 16-bit signed integer PCM WAV:
- Format tag: 1 (PCM)
- Bits per sample: 16
- Sample rate: 48000 Hz (video-standard)
- Channels: 1 (mono)
- f32 samples are converted to i16 by clamping to [-1.0, 1.0] and scaling

---

## Decision: FFmpeg Audio Muxing Strategy

### Option A: Single-pass encoding (frames + audio -> .mp4)

T-005-01's encode function accepts an optional audio file path. FFmpeg
receives both the frame directory and the WAV in one invocation:

```
ffmpeg -framerate 30 -i frame_%06d.png -i audio.wav \
       -c:v libx264 -c:a aac -pix_fmt yuv420p output.mp4
```

- Pro: One FFmpeg invocation, no intermediate video-only file
- Con: Requires T-005-01 to expose an audio input parameter

### Option B: Two-pass muxing (video-only .mp4, then mux audio)

T-005-01 produces a video-only .mp4. A second FFmpeg call muxes audio:

```
ffmpeg -i video.mp4 -i audio.wav -c:v copy -c:a aac -shortest final.mp4
```

- Pro: Decoupled from T-005-01's API; video encoding is independent
- Con: Two FFmpeg invocations; intermediate file management; `-c:v copy`
  avoids re-encoding but still requires temp file cleanup

**Decision: Option A (single-pass), with Option B as fallback.**

The preferred approach is to extend T-005-01's API to accept an optional
audio file path. Since T-005-01 is being built by another agent, we design
for both:

1. If T-005-01 exposes an `audio_path: Option<PathBuf>` in its config,
   we pass the WAV path directly. Single FFmpeg invocation.
2. If T-005-01 does not support audio input, we add a separate
   `mux_audio(video_path, audio_path, output_path)` function that runs
   a second FFmpeg pass with `-c:v copy -c:a aac`.

The `mux_audio` function is useful regardless -- it is a clean, testable
unit that may be needed for other workflows (e.g., replacing audio in an
existing video).

---

## Decision: Sample Rate

**48000 Hz.** This is the standard for video production (broadcast, film,
web video). 44100 Hz is CD audio and would also work, but 48000 aligns
with the video domain. Define as a constant `DEFAULT_AUDIO_SAMPLE_RATE`.

---

## Proposed API Surface

### moron-voice/src/audio.rs additions

```rust
// Constants
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;

impl AudioClip {
    /// Concatenate another clip onto this one.
    /// Panics if sample_rate or channels differ.
    pub fn append(&mut self, other: &AudioClip) { ... }

    /// Concatenate a sequence of clips into one.
    /// Returns an empty clip if the input is empty.
    pub fn concat(clips: &[AudioClip], sample_rate: u32) -> AudioClip { ... }

    /// Encode this clip as WAV bytes (16-bit signed PCM).
    pub fn to_wav_bytes(&self) -> Vec<u8> { ... }

    /// Write this clip to a WAV file.
    pub fn write_wav(&self, path: &Path) -> Result<(), std::io::Error> { ... }
}
```

### moron-core audio assembly (new module or in ffmpeg.rs)

```rust
/// Walk timeline segments and produce a single AudioClip.
/// Narration and Silence segments produce silence of the specified duration.
/// Animation and Clip segments also produce silence (placeholder).
pub fn assemble_audio_track(timeline: &Timeline) -> AudioClip { ... }
```

### moron-core FFmpeg extension

```rust
/// Mux a video file with an audio file into a final .mp4.
/// Video stream is copied (no re-encoding). Audio is encoded to AAC.
pub fn mux_audio(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
) -> Result<(), FfmpegError> { ... }
```

---

## Pipeline Flow

```
Scene::build(&mut m)
        |
        v
    Timeline (segments with durations)
        |
        +---> render() -> PNG frames directory
        |
        +---> assemble_audio_track(&timeline) -> AudioClip
                    |
                    v
              clip.write_wav(temp_audio.wav)
                    |
                    v
    FFmpeg: frames + audio.wav -> output.mp4
                    |
                    v
           (or two-pass: encode frames -> video.mp4, then mux_audio)
```

---

## WAV Header Layout (for reference)

```
Offset  Size  Field           Value
0       4     ChunkID         "RIFF"
4       4     ChunkSize       36 + data_size
8       4     Format          "WAVE"
12      4     Subchunk1ID     "fmt "
16      4     Subchunk1Size   16
20      2     AudioFormat     1 (PCM)
22      2     NumChannels     1 or 2
24      4     SampleRate      48000
28      4     ByteRate        SampleRate * NumChannels * BitsPerSample/8
32      2     BlockAlign      NumChannels * BitsPerSample/8
34      2     BitsPerSample   16
36      4     Subchunk2ID     "data"
40      4     Subchunk2Size   NumSamples * NumChannels * BitsPerSample/8
44      ...   Data            PCM samples (little-endian i16)
```

Total header: 44 bytes. All multi-byte fields are little-endian.

---

## Testing Strategy

### Unit tests in moron-voice/src/audio.rs

1. `concat_empty` -- concatenating zero clips returns empty clip
2. `concat_single` -- concatenating one clip returns equivalent clip
3. `concat_multiple` -- durations and sample counts add up correctly
4. `append_extends_data` -- append grows data vector correctly
5. `to_wav_bytes_header` -- first 44 bytes match expected WAV header
6. `to_wav_bytes_silence` -- silence clip produces all-zero PCM data
7. `to_wav_bytes_round_trip` -- write then parse header, verify fields
8. `f32_to_i16_conversion` -- edge cases: 0.0, 1.0, -1.0, clipping

### Unit tests in moron-core (assembly)

1. `assemble_empty_timeline` -- empty timeline produces zero-duration clip
2. `assemble_silence_only` -- silence segments produce correct duration
3. `assemble_narration_placeholder` -- narration produces silence of duration
4. `assemble_mixed_segments` -- all segment types, total duration matches
5. `assembled_track_duration_matches_timeline` -- property: assembled clip
   duration equals `timeline.total_duration()`

### Integration tests (with FFmpeg, gated)

1. `mux_audio_produces_valid_mp4` -- write test WAV + video, run mux,
   verify output has both streams (via `ffprobe`)

---

## Rejected Alternatives

### Opus/OGG instead of WAV
More compact but adds encoding complexity and dependencies. WAV is lossless,
trivial to produce, and FFmpeg re-encodes to AAC anyway. No benefit for an
intermediate format.

### Writing audio directly to FFmpeg stdin
Possible via `-f s16le -i pipe:0` but complicates the pipeline. WAV file is
simpler, debuggable (can be played independently), and avoids pipe-management
complexity.

### Stereo output
Mono is sufficient for narration/explainer videos. Stereo doubles the file
size with no benefit for speech. Future work can add stereo support if needed
for music or sound effects.

---

## Open Questions for Implementation

1. **T-005-01 API shape**: Will the FFmpeg encode function accept an audio
   path? If not, we need the two-pass mux approach. Design accommodates both.

2. **Temp file management**: Where does the intermediate WAV file live?
   Likely in the same temp directory as the frames, cleaned up after muxing.

3. **Error handling**: WAV write failures and FFmpeg mux failures need
   clear error types. Extend the existing error enums or create new ones.
