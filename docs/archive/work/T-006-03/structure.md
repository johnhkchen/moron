# T-006-03 Structure: Pipeline Audio Integration

## Files Modified

### moron-core/src/build.rs

**BuildError** -- add variant:
```rust
#[error("TTS synthesis failed for segment {segment}: {source}")]
Tts { segment: usize, source: anyhow::Error },
```

**BuildProgress** -- add variant:
```rust
SynthesizingTts { current: usize, total: usize },
```

**BuildConfig** -- add field:
```rust
pub voice_backend: Option<Arc<dyn moron_voice::VoiceBackend + Send + Sync>>,
```

Initialize to `None` in `BuildConfig::new()`.

**build_video** -- signature change:
```rust
pub async fn build_video(m: &mut M, config: BuildConfig) -> Result<BuildResult, BuildError>
```

New internal function:
```rust
fn synthesize_narrations(
    m: &mut M,
    backend: &dyn VoiceBackend,
    progress: &Option<Arc<dyn Fn(BuildProgress) + Send + Sync>>,
) -> Result<Vec<AudioClip>, BuildError>
```

This function:
1. Gets narration indices and texts from timeline
2. For each narration segment, calls backend.synthesize(text)
3. Reports SynthesizingTts progress
4. Collects audio clips and their durations
5. Calls m.resolve_narration_durations(&durations)
6. Returns the clips

Pipeline flow in build_video:
```
// Step 0: Synthesize TTS (if backend available)
let narration_clips = match &config.voice_backend {
    Some(backend) => Some(synthesize_narrations(m, backend.as_ref(), &config.progress)?),
    None => None,
};

// Recompute timeline stats after duration resolution
let total_duration = m.timeline().total_duration();
let total_frames = m.timeline().total_frames();

// ... existing steps (render, encode, assemble, mux) ...

// Step 3: Assemble audio with real clips
let sample_rate = narration_clips.as_ref()
    .and_then(|clips| clips.first())
    .map(|c| c.sample_rate)
    .unwrap_or(moron_voice::DEFAULT_SAMPLE_RATE);
let audio_clip = ffmpeg::assemble_audio_track(m.timeline(), sample_rate, narration_clips.as_deref());
```

### moron-core/src/ffmpeg.rs

**assemble_audio_track** -- signature change:
```rust
pub fn assemble_audio_track(
    timeline: &Timeline,
    sample_rate: u32,
    narration_clips: Option<&[AudioClip]>,
) -> AudioClip
```

Logic:
- Maintain a narration clip index counter (starts at 0)
- For each segment:
  - If Narration AND narration_clips is Some: use clips[narration_idx], increment index
  - Otherwise: AudioClip::silence(seg.duration(), sample_rate)
- Concatenate all clips

### moron-core/src/lib.rs

No changes needed. BuildError, BuildProgress, BuildConfig are already re-exported.

## Module Boundaries

- build.rs owns the pipeline orchestration and TTS synthesis step
- ffmpeg.rs owns audio assembly (receives pre-synthesized clips, doesn't know about VoiceBackend)
- facade.rs is unchanged (resolve_narration_durations already exists)
- moron-voice is unchanged (VoiceBackend trait and AudioClip already exist)

## Public API Changes

1. `BuildConfig::voice_backend` -- new optional field
2. `build_video` -- `&M` becomes `&mut M`
3. `BuildError::Tts` -- new error variant
4. `BuildProgress::SynthesizingTts` -- new progress variant
5. `assemble_audio_track` -- new third parameter

## Test Strategy

- Unit test `synthesize_narrations` with a mock backend
- Unit test `assemble_audio_track` with provided narration clips
- Unit test `assemble_audio_track` with None (backward compat)
- Unit test BuildConfig defaults (voice_backend is None)
- Existing tests continue to pass (assemble_audio_track call sites updated)
