# T-005-02 Research: Audio Track Assembly

## Objective

Map the codebase components relevant to assembling an audio track from timeline
segments and muxing it with rendered video via FFmpeg.

---

## 1. Timeline Segments (moron-core/src/timeline.rs)

The `Segment` enum defines four variant types:

```rust
pub enum Segment {
    Narration { text: String, duration: f64 },
    Animation { name: String, duration: f64 },
    Silence  { duration: f64 },
    Clip     { path: PathBuf, duration: f64 },
}
```

Every variant carries a `duration: f64` (seconds). The `Segment::duration()`
method extracts this uniformly across all variants.

`Timeline` stores an ordered `Vec<Segment>` plus an FPS value (default 30).
Key accessors:
- `segments() -> &[Segment]` -- iterate all segments in order
- `total_duration() -> f64` -- sum of all segment durations
- `fps() -> u32` -- the configured frame rate

For audio assembly, we need to walk `segments()` and produce audio data for
each segment based on its type and duration. The timeline is the single source
of truth for ordering and timing.

### Audio-relevant segment types

| Segment     | Audio behavior for T-005-02                       |
|-------------|----------------------------------------------------|
| Narration   | Placeholder silence of `duration` seconds          |
| Silence     | Silence of `duration` seconds                      |
| Animation   | Silence of `duration` seconds (no audio content)   |
| Clip        | Silence of `duration` seconds (real clip loading is future work) |

For this ticket, every segment becomes silence. The distinction matters for
future work (TTS produces real audio for Narration; Clip loads a file), but
right now the assembler generates a silent track whose duration exactly matches
the video.

---

## 2. AudioClip (moron-voice/src/audio.rs)

```rust
pub struct AudioClip {
    pub data: Vec<f32>,       // interleaved PCM, f32, [-1.0, 1.0]
    pub duration: f64,        // seconds
    pub sample_rate: u32,     // Hz (22050, 44100, etc.)
    pub channels: u16,        // 1=mono, 2=stereo
}
```

`AudioClip::silence(duration, sample_rate)` creates a mono clip filled with
zeros. The sample count is `(duration * sample_rate) as usize`.

Current limitations:
- No `concat` or `append` method exists.
- No WAV serialization exists.
- No method to convert f32 PCM to i16 PCM (needed for standard WAV).
- The struct is `Clone` and `Debug`.

### What needs to be added

1. **Concatenation** -- join multiple AudioClips into one contiguous clip.
   All clips must share the same sample_rate and channels, or conversion
   is needed. For T-005-02 all clips are mono silence at one sample rate,
   so this is straightforward.

2. **WAV encoding** -- serialize to WAV bytes (or write to a file path).
   WAV is the simplest uncompressed format FFmpeg can ingest. The WAV
   header is 44 bytes; the payload is PCM samples. We need to decide
   between f32 WAV and i16 WAV. i16 is more universally supported by
   FFmpeg and smaller. For silence it does not matter, but the code path
   should be correct for future real audio.

---

## 3. moron-voice Module Structure (moron-voice/src/lib.rs)

```
moron-voice/
  src/
    lib.rs          -- re-exports AudioClip, Voice, VoiceBackend, backends
    audio.rs        -- AudioClip struct and silence() constructor
    backend.rs      -- VoiceBackend trait, Voice config, VoiceBackendType enum
    alignment.rs    -- stub (word-level timestamps, future)
    kokoro.rs       -- KokoroBackend stub (todo!())
    piper.rs        -- PiperBackend stub (todo!())
```

`audio.rs` is the natural home for audio assembly utilities (concat, WAV
encoding). The module already owns the `AudioClip` type and is re-exported
at the crate root.

Dependencies: `moron-voice` depends on `serde`, `tokio`, `anyhow`, `thiserror`.
No audio-specific crates. WAV encoding will need to be either hand-rolled
(44-byte header + PCM data -- trivial for our case) or use a crate like `hound`.

---

## 4. FFmpeg Module (moron-core/src/ffmpeg.rs)

Currently a one-line stub:
```rust
//! FFmpeg pipeline: frame encoding, muxing, and output format handling.
```

T-005-01 (being implemented by another agent) will replace this with:
- FFmpeg detection on PATH
- `std::process::Command`-based spawning
- PNG directory -> H.264 .mp4 encoding
- Configurable FPS, resolution, output path

T-005-02 must extend whatever T-005-01 produces to add audio muxing. The
expected FFmpeg command for muxing audio with video is:

```
ffmpeg -i video.mp4 -i audio.wav -c:v copy -c:a aac -shortest output.mp4
```

Or, if T-005-01 produces video-only .mp4, we can mux in a second pass.
Alternatively, audio can be provided alongside frames in a single FFmpeg
invocation:

```
ffmpeg -framerate 30 -i frame_%06d.png -i audio.wav \
       -c:v libx264 -c:a aac -pix_fmt yuv420p output.mp4
```

This single-pass approach is preferable: fewer temp files, one FFmpeg
invocation. It depends on how T-005-01 structures its API.

---

## 5. Render Pipeline (moron-core/src/renderer.rs)

The current `render()` function:
1. Computes total frames from timeline
2. Launches ChromiumBridge
3. For each frame: compute_frame_state -> JSON -> bridge.capture_frame -> PNG
4. Closes bridge
5. Returns `RenderResult { total_frames, output_dir }`

The render result gives us the output directory of PNGs. The FFmpeg step
(T-005-01) takes this directory and encodes to .mp4. Audio assembly
(T-005-02) happens between rendering and encoding, or is fed directly
to FFmpeg alongside the frames.

---

## 6. M Facade (moron-core/src/facade.rs)

The `M` struct holds the `Timeline` and exposes `timeline() -> &Timeline`.
Scene authors call `m.narrate()`, `m.beat()`, `m.wait()`, `m.play()` which
push segments onto the timeline. After `Scene::build(m)` completes, the
timeline is fully populated.

The facade does not currently expose any audio assembly API. Audio assembly
is an internal pipeline step, not a scene-authoring concern. The assembler
will read `m.timeline()` to produce the audio track.

---

## 7. Dependency Graph

```
moron-core depends on moron-voice (Cargo.toml)
```

This means moron-core can use `AudioClip` and any audio utilities we add to
moron-voice. The audio assembly function could live in either crate:
- **moron-voice**: natural home for audio types and utilities
- **moron-core**: closer to the FFmpeg pipeline that consumes the audio

The assembly logic needs both `Timeline` (moron-core) and `AudioClip`
(moron-voice). Since moron-core already depends on moron-voice, the
assembly function that takes a `&Timeline` and returns an `AudioClip`
could live in moron-core's ffmpeg module or a new audio_assembly module.
Alternatively, a standalone function in moron-voice that takes segment
durations (not the Timeline type) keeps the dependency direction clean.

---

## 8. WAV Format Requirements

FFmpeg accepts WAV with these characteristics:
- RIFF/WAVE container
- PCM encoding (format tag 1 for integer PCM, tag 3 for float PCM)
- 16-bit signed integer is the most portable choice
- Sample rates: 44100 Hz or 48000 Hz are standard for video
- Mono is fine (stereo is optional)

The WAV header is exactly 44 bytes for standard PCM. Writing it is
straightforward: 12 bytes RIFF header, 24 bytes fmt chunk, 8 bytes
data chunk header, then raw PCM bytes.

For video work, 48000 Hz is conventional (matches broadcast/film standards).
44100 Hz is CD audio. Either works; 48000 is slightly more appropriate for
video output.

---

## 9. Constraints and Assumptions

1. T-005-01 API is unknown -- we must design for a likely interface
   (accepts a directory of PNGs, FPS, output path, and optionally an
   audio file path).
2. All segments produce silence for now -- the assembly logic is simple
   but must be structured to support real AudioClips from TTS in the future.
3. The output .mp4 must have both video and audio streams per the
   acceptance criteria.
4. `cargo check` must pass -- no breaking changes to existing types.
5. moron-voice has no audio crate dependencies -- WAV encoding must be
   either hand-rolled or a new dependency must be added.

---

## 10. Key Files Summary

| File | Role | Status |
|------|------|--------|
| `moron-voice/src/audio.rs` | AudioClip type, needs concat + WAV | Extend |
| `moron-core/src/timeline.rs` | Segment enum, Timeline | Read-only |
| `moron-core/src/ffmpeg.rs` | Stub, T-005-01 replaces | Extend (after T-005-01) |
| `moron-core/src/facade.rs` | M facade, exposes timeline | Read-only |
| `moron-core/src/renderer.rs` | Frame render loop | Read-only |
| `moron-voice/src/lib.rs` | Module exports | May need new re-exports |
| `moron-voice/Cargo.toml` | Dependencies | May add hound crate |
| `moron-core/Cargo.toml` | Dependencies | Unchanged |
