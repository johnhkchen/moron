# T-006-03 Design: Pipeline Audio Integration

## Problem

The build pipeline produces all-silence audio. With TTS backends now available, narration segments should use synthesized speech. The pipeline ordering must change: TTS synthesis happens before frame rendering so resolved durations drive frame timing.

## Approach: Optional VoiceBackend in BuildConfig

### BuildConfig Change

Add an optional `voice_backend` field to BuildConfig:

```rust
pub voice_backend: Option<Arc<dyn VoiceBackend + Send + Sync>>,
```

Using `Arc<dyn VoiceBackend + Send + Sync>` because:
- `Arc` allows shared ownership (BuildConfig may be cloned/shared)
- `dyn` trait object allows any backend (Kokoro, Piper, future backends)
- `Send + Sync` required since build_video is async

### build_video Signature Change

Change from `&M` to `&mut M` to allow calling `resolve_narration_durations`:

```rust
pub async fn build_video(m: &mut M, config: BuildConfig) -> Result<BuildResult, BuildError>
```

### New Pipeline Flow

1. Build scene (unchanged -- happens before build_video is called)
2. **NEW**: If voice_backend is Some, synthesize TTS for all narration segments
3. **NEW**: Resolve narration durations with actual TTS durations
4. Create temp dir
5. Render frames (timing now matches TTS durations)
6. Encode video
7. **MODIFIED**: Assemble audio track using real TTS clips for narrations, silence for gaps
8. Mux and cleanup

### assemble_audio_track Change

Change signature to accept optional narration clips:

```rust
pub fn assemble_audio_track(
    timeline: &Timeline,
    sample_rate: u32,
    narration_clips: Option<&[AudioClip]>,
) -> AudioClip
```

When `narration_clips` is Some, use the actual audio clip for narration segments. When None, fall back to silence (current behavior).

### Sample Rate Strategy

**Decision**: Use the TTS backend's native sample rate for the entire audio track when TTS is active. When no backend, use DEFAULT_SAMPLE_RATE (48kHz).

Rationale:
- Avoids resampling complexity (no resampling library needed)
- Kokoro produces 24kHz which is adequate for speech
- FFmpeg handles the sample rate during mux (AAC encoding normalizes it)
- Silence clips are trivially generated at any sample rate

The sample rate for silence segments will match the TTS output sample rate, determined from the first synthesized clip (or DEFAULT_SAMPLE_RATE if no narrations).

### Error Handling

Add a new variant to BuildError:

```rust
#[error("TTS synthesis failed for segment {segment}: {source}")]
Tts { segment: usize, source: anyhow::Error }
```

TTS failure should be a hard error (not a graceful fallback to silence) when a backend is configured. The "graceful fallback" is when no backend is configured at all -- then everything is silence.

### Rejected Alternatives

**Option A: TTS as a separate pre-processing step outside build_video.**
Rejected because it splits pipeline logic across callers, making the API harder to use correctly. The ordering constraint (TTS before render) should be enforced by the pipeline itself.

**Option B: Resample TTS output to 48kHz.**
Rejected because it adds complexity (need a resampling function), increases binary size, and FFmpeg already handles format conversion during mux. No quality benefit for speech audio.

**Option C: Make build_video take &M and have a separate step for duration resolution.**
Rejected because it exposes pipeline ordering to callers. The pipeline should encapsulate this.

### Progress Reporting

Add a new BuildProgress variant:

```rust
BuildProgress::SynthesizingTts { current: usize, total: usize }
```

This reports TTS synthesis progress before rendering begins.

## Summary of Changes

| File | Change |
|------|--------|
| build.rs | BuildConfig: add voice_backend; build_video: &M -> &mut M; add TTS step; add BuildError::Tts; add BuildProgress::SynthesizingTts |
| ffmpeg.rs | assemble_audio_track: add narration_clips parameter |
| lib.rs | No changes needed (existing re-exports cover new types) |
