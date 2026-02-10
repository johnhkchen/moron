# T-006-02 Research: Audio-Synced Timeline

## 1. Current Narration Duration Estimation

`M::narrate()` in `moron-core/src/facade.rs` (line 118-125) computes duration
from word count:

```rust
let words = text.split_whitespace().count().max(1) as f64;
let duration = words * 60.0 / DEFAULT_NARRATION_WPM;  // 150 WPM
```

This produces a `Segment::Narration { text, duration }` and pushes it onto the
timeline immediately. The duration is fixed at creation time -- there is no
mechanism to update it later.

## 2. Timeline Architecture

`Timeline` (`moron-core/src/timeline.rs`) is a `Vec<Segment>` with an FPS
setting. Key observations:

- **All segments carry fixed `f64` durations.** The `Segment` enum has four
  variants (Narration, Animation, Silence, Clip), each with a concrete `duration`
  field. There is no concept of "pending" or "deferred" duration.
- **`total_duration()`** sums all segment durations. Used by `total_frames()`,
  `frame_at()`, and `segments_in_range()`.
- **`segments` is a private `Vec`** accessed only through `segments()` (immutable
  slice) and `add_segment()` (append-only). There is no API to mutate an existing
  segment's duration or replace a segment at an index.
- **`TimelineBuilder`** is a fluent construction API. It also takes concrete
  durations at creation time.

The timeline has **no index-based mutation, no segment IDs, and no deferred
resolution** today.

## 3. Element `created_at` Timestamps

`M::mint_element_with_meta()` (facade.rs, line 226-246) records each element's
`created_at` as `self.timeline.total_duration()` at the moment of creation.
This timestamp drives visibility: `compute_frame_state` (frame.rs, line 132)
checks `rec.created_at <= clamped_time`.

**Critical coupling:** If narration durations change after scene build, any
element created *after* a narration segment will have a stale `created_at`.
Example:

```
m.narrate("long text");   // estimated 3.2s, actual TTS = 4.5s
m.title("Slide 2");       // created_at = 3.2s, should be 4.5s
```

The `elements` vec in `M` stores `created_at` as a plain `f64`. There is no
mechanism to recompute these timestamps after duration changes.

## 4. Build Pipeline Flow

`build_video()` in `moron-core/src/build.rs` (line 150-258):

1. Reads timeline stats (duration, frames, fps) from `M`
2. Renders frames via Chromium bridge (uses `compute_frame_state`)
3. Assembles audio track via `ffmpeg::assemble_audio_track()`
4. Encodes video, muxes audio, cleans up

**TTS synthesis does not exist in the pipeline yet.** The `assemble_audio_track()`
function (ffmpeg.rs, line 279-287) produces silence for every segment regardless
of type:

```rust
.map(|seg| AudioClip::silence(seg.duration(), sample_rate))
```

The pipeline is single-pass: scene build produces final M, then rendering and
audio assembly happen in parallel (conceptually). There is no "TTS pass" step.

## 5. VoiceBackend Trait and Kokoro Stub

`VoiceBackend` trait (moron-voice/src/backend.rs):
```rust
fn synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error>;
```

Returns an `AudioClip` with `data`, `duration`, `sample_rate`, `channels`.
The `duration` field of a synthesized clip is the ground truth -- this is what
should replace the WPM estimate.

`KokoroBackend` (moron-voice/src/kokoro.rs) is currently a `todo!()` stub.
T-006-01 will make it functional. The contract is: give it text, get back an
AudioClip whose duration reflects actual speech length.

The `Voice` struct has `speed` and `pitch` multipliers. These affect the TTS
output but do not change the interface -- `synthesize()` still returns an
AudioClip with the true duration.

## 6. AudioClip Capabilities

`AudioClip` (moron-voice/src/audio.rs) supports:
- `silence(duration, sample_rate)` -- generate silent clip
- `append(&other)` -- concatenate another clip (must match sample_rate/channels)
- `concat(clips, sample_rate, channels)` -- concatenate a slice
- `to_wav_bytes()` -- encode as WAV
- `duration()` -- read the duration field

There is no resample or speed-change utility. All clips must share the same
sample rate to be concatenated.

## 7. Where TTS Should Happen in the Pipeline

The current flow is: `Scene::build(m)` -> `build_video(m, config)`.

TTS must happen **after** scene build (to collect all narration texts) but
**before** frame rendering (to have correct durations for visual timing). This
means TTS synthesis is a new step inserted between scene build and rendering.

The `build_video()` function receives an `&M` (immutable reference). To update
durations, it would need either `&mut M` or a separate resolved timeline.

## 8. Scope Boundary with T-006-03

T-006-02 is about **duration resolution** -- making the timeline aware of real
TTS durations. T-006-03 is about **audio integration** -- wiring real AudioClip
data into the FFmpeg audio assembly.

T-006-02 must provide:
- A mechanism to resolve narration durations from TTS output
- Updated timeline and element timestamps reflecting real durations
- Continued fallback to WPM estimation when TTS is unavailable

T-006-03 will then:
- Call the TTS backend to get actual AudioClip data
- Plug those clips into `assemble_audio_track()`

## 9. Existing Test Surface

Tests that depend on narration timing:
- `facade::tests::narrate_records_narration` -- asserts 0.8s for "Hello world"
- `facade::tests::timeline_tracks_cumulative_duration` -- asserts cumulative sum
- `frame::tests::elements_not_visible_before_creation` -- asserts element
  visibility based on created_at relative to narration duration
- `frame::tests::active_narration_during_segment` -- checks narration text lookup

All of these use the WPM estimation. They must continue to pass after T-006-02
(fallback path). New tests must verify the TTS-resolved path.

## 10. Constraints and Assumptions

- T-006-01 (Kokoro backend) is not yet complete. T-006-02 must work with the
  `VoiceBackend` trait abstractly, not depend on Kokoro specifics.
- Scene authors must not need to change their code. `m.narrate("text")` should
  work identically from the author's perspective.
- The `M` struct is `pub` but its internals (timeline, elements) are `pub(crate)`.
  Duration resolution can use crate-internal access.
- TTS synthesis can be slow (even at 96x realtime, a 60-second narration takes
  0.6 seconds). The resolution step should be batch-friendly.
- `build_video` currently takes `&M`. Changing to `&mut M` is a public API
  change that affects T-006-03 and the CLI.
