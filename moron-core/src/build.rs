//! Build pipeline: orchestrate the full scene-to-video workflow.
//!
//! The [`build_video`] function is the single entry point for producing an `.mp4`
//! from a built scene. It renders frames via the Chromium bridge, encodes video
//! via FFmpeg, assembles and muxes the audio track, and cleans up intermediate
//! files.
//!
//! When a [`VoiceBackend`](moron_voice::VoiceBackend) is configured in the
//! [`BuildConfig`], narration segments are synthesized via TTS before frame
//! rendering begins. The resulting audio durations replace the initial WPM
//! estimates so that frame timing matches the actual speech. Without a backend,
//! all narration segments are rendered as silence (backward-compatible).
//!
//! The CLI (`moron build`) is a thin wrapper around this function.

use std::path::PathBuf;
use std::sync::Arc;

use moron_voice::{AudioClip, VoiceBackend};

use crate::chromium::BridgeConfig;
use crate::facade::M;
use crate::ffmpeg::{self, EncodeConfig, FfmpegError};
use crate::renderer::{self, RenderConfig, RenderError, RenderProgress};
use crate::timeline::Segment;

// ---------------------------------------------------------------------------
// BuildError
// ---------------------------------------------------------------------------

/// Errors produced by the build pipeline.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// Frame rendering failed.
    #[error("render failed: {0}")]
    Render(#[from] RenderError),

    /// FFmpeg encoding or muxing failed.
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(#[from] FfmpegError),

    /// File system I/O failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid configuration.
    #[error("{0}")]
    Config(String),

    /// TTS synthesis failed for a narration segment.
    #[error("TTS synthesis failed for segment {segment}: {source}")]
    Tts {
        /// Zero-based index of the narration segment that failed.
        segment: usize,
        /// The underlying synthesis error.
        source: anyhow::Error,
    },
}

// ---------------------------------------------------------------------------
// BuildProgress
// ---------------------------------------------------------------------------

/// Progress updates emitted during the build pipeline.
pub enum BuildProgress {
    /// TTS synthesis is in progress.
    SynthesizingTts {
        /// The narration segment being synthesized (0-indexed).
        current: usize,
        /// Total number of narration segments.
        total: usize,
    },
    /// The scene has been analyzed; rendering is about to begin.
    SceneBuilt {
        /// Total duration of the timeline in seconds.
        total_duration: f64,
        /// Total number of frames to render.
        total_frames: u32,
    },
    /// A frame has been rendered.
    RenderingFrame {
        /// The frame that was just rendered (0-indexed).
        current: u32,
        /// Total number of frames.
        total: u32,
    },
    /// FFmpeg video encoding has started.
    Encoding,
    /// FFmpeg audio muxing has started.
    MuxingAudio,
    /// The pipeline has completed successfully.
    Complete {
        /// Path to the final .mp4 file.
        output_path: PathBuf,
        /// Total number of frames in the video.
        total_frames: u32,
        /// Duration of the video in seconds.
        duration: f64,
    },
}

// ---------------------------------------------------------------------------
// BuildConfig
// ---------------------------------------------------------------------------

/// Configuration for the full build pipeline.
pub struct BuildConfig {
    /// Path to the final output `.mp4` file.
    pub output_path: PathBuf,
    /// Path to the built React app's `index.html`.
    pub html_path: PathBuf,
    /// Viewport and output width in pixels.
    pub width: u32,
    /// Viewport and output height in pixels.
    pub height: u32,
    /// If true, preserve the temporary frame directory after build.
    pub keep_frames: bool,
    /// Optional progress callback, wrapped in `Arc` so it can be shared
    /// with the render progress forwarder.
    pub progress: Option<Arc<dyn Fn(BuildProgress) + Send + Sync>>,
    /// Optional TTS backend for synthesizing narration audio.
    ///
    /// When `Some`, narration segments are synthesized before frame rendering
    /// and the resulting audio is wired into the final .mp4. When `None`,
    /// all narration segments produce silence (backward-compatible).
    pub voice_backend: Option<Arc<dyn VoiceBackend + Send + Sync>>,
}

impl BuildConfig {
    /// Create a new build config with sensible defaults.
    ///
    /// - Resolution: 1920x1080
    /// - keep_frames: false
    /// - No progress callback
    /// - No TTS backend (all narration is silence)
    pub fn new(output_path: impl Into<PathBuf>, html_path: impl Into<PathBuf>) -> Self {
        Self {
            output_path: output_path.into(),
            html_path: html_path.into(),
            width: 1920,
            height: 1080,
            keep_frames: false,
            progress: None,
            voice_backend: None,
        }
    }
}

// ---------------------------------------------------------------------------
// BuildResult
// ---------------------------------------------------------------------------

/// Summary returned after a successful build.
pub struct BuildResult {
    /// Path to the output `.mp4` file.
    pub output_path: PathBuf,
    /// Total number of frames rendered.
    pub total_frames: u32,
    /// Duration of the video in seconds.
    pub duration: f64,
}

// ---------------------------------------------------------------------------
// build_video
// ---------------------------------------------------------------------------

/// Run the full rendering pipeline: render frames, encode video, mux audio.
///
/// Given a built scene (`M` with a recorded timeline) and a [`BuildConfig`],
/// this function:
///
/// 1. Synthesizes TTS for narration segments (if `voice_backend` is configured)
/// 2. Resolves narration durations from actual TTS output
/// 3. Reports scene statistics
/// 4. Creates a temporary directory for intermediate files
/// 5. Renders frames via the Chromium bridge (timing matches TTS durations)
/// 6. Encodes frames into a video-only `.mp4` via FFmpeg
/// 7. Assembles an audio track (real TTS audio for narrations, silence for gaps)
/// 8. Muxes video + audio into the final `.mp4`
/// 9. Cleans up intermediate files (unless `keep_frames` is set)
///
/// # Errors
///
/// Returns [`BuildError`] if any pipeline stage fails, including TTS synthesis.
pub async fn build_video(m: &mut M, config: BuildConfig) -> Result<BuildResult, BuildError> {
    // -----------------------------------------------------------------------
    // Step 0: Synthesize TTS (if backend available)
    // -----------------------------------------------------------------------

    let narration_clips = match &config.voice_backend {
        Some(backend) => {
            Some(synthesize_narrations(m, backend.as_ref(), &config.progress)?)
        }
        None => None,
    };

    // Recompute timeline stats after potential duration resolution.
    let total_duration = m.timeline().total_duration();
    let total_frames = m.timeline().total_frames();
    let fps = m.timeline().fps();

    // Report scene stats.
    report(&config.progress, BuildProgress::SceneBuilt {
        total_duration,
        total_frames,
    });

    if total_frames == 0 {
        return Err(BuildError::Config(
            "scene has no timeline segments (0 frames to render)".to_string(),
        ));
    }

    // Create temp directory for intermediate files.
    let temp_dir = std::env::temp_dir().join(format!("moron-build-{}", std::process::id()));
    let frames_dir = temp_dir.join("frames");
    std::fs::create_dir_all(&frames_dir)?;

    // Intermediate file paths.
    let video_only_path = temp_dir.join("video_only.mp4");
    let audio_path = temp_dir.join("audio.wav");

    // -----------------------------------------------------------------------
    // Step 1: Render frames
    // -----------------------------------------------------------------------

    let bridge_config = BridgeConfig {
        width: config.width,
        height: config.height,
        html_path: config.html_path.clone(),
        chrome_executable: None,
        headless: true,
        launch_timeout: std::time::Duration::from_secs(20),
    };

    let render_progress: Option<Box<dyn Fn(RenderProgress)>> = match &config.progress {
        Some(cb) => {
            let cb = Arc::clone(cb);
            Some(Box::new(move |p: RenderProgress| {
                cb(BuildProgress::RenderingFrame {
                    current: p.current_frame,
                    total: p.total_frames,
                });
            }))
        }
        None => None,
    };

    let render_config = RenderConfig {
        output_dir: frames_dir.clone(),
        bridge_config,
        progress: render_progress,
    };

    renderer::render(m, render_config).await?;

    // -----------------------------------------------------------------------
    // Step 2: Encode video (frames -> video-only .mp4)
    // -----------------------------------------------------------------------

    report(&config.progress, BuildProgress::Encoding);

    let encode_config = EncodeConfig::new(&frames_dir, &video_only_path)
        .fps(fps)
        .resolution(config.width, config.height);

    ffmpeg::encode(&encode_config)?;

    // -----------------------------------------------------------------------
    // Step 3: Assemble audio track and mux with video
    // -----------------------------------------------------------------------

    report(&config.progress, BuildProgress::MuxingAudio);

    // Determine sample rate: use the TTS output sample rate when available,
    // fall back to the default broadcast sample rate otherwise.
    let sample_rate = narration_clips
        .as_ref()
        .and_then(|clips| clips.first())
        .map(|c| c.sample_rate)
        .unwrap_or(moron_voice::DEFAULT_SAMPLE_RATE);

    let audio_clip = ffmpeg::assemble_audio_track(
        m.timeline(),
        sample_rate,
        narration_clips.as_deref(),
    );
    let wav_bytes = audio_clip.to_wav_bytes();
    std::fs::write(&audio_path, &wav_bytes)?;

    ffmpeg::mux_audio(&video_only_path, &audio_path, &config.output_path)?;

    // -----------------------------------------------------------------------
    // Step 4: Clean up
    // -----------------------------------------------------------------------

    if !config.keep_frames {
        // Best-effort cleanup. Failure to clean up is not a build error.
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    // Report completion.
    report(&config.progress, BuildProgress::Complete {
        output_path: config.output_path.clone(),
        total_frames,
        duration: total_duration,
    });

    Ok(BuildResult {
        output_path: config.output_path,
        total_frames,
        duration: total_duration,
    })
}

// ---------------------------------------------------------------------------
// synthesize_narrations
// ---------------------------------------------------------------------------

/// Synthesize TTS audio for all narration segments in the timeline.
///
/// For each narration segment, this function:
/// 1. Extracts the text from the segment
/// 2. Calls `backend.synthesize(text)` to produce an [`AudioClip`]
/// 3. Reports `SynthesizingTts` progress
/// 4. Collects the resulting durations
/// 5. Calls `m.resolve_narration_durations()` to update the timeline
///
/// Returns the synthesized clips in timeline order (one per narration segment).
fn synthesize_narrations(
    m: &mut M,
    backend: &dyn VoiceBackend,
    progress: &Option<Arc<dyn Fn(BuildProgress) + Send + Sync>>,
) -> Result<Vec<AudioClip>, BuildError> {
    let narration_indices = m.timeline().narration_indices();
    let total = narration_indices.len();

    if total == 0 {
        return Ok(Vec::new());
    }

    // Collect narration texts.
    let texts: Vec<String> = narration_indices
        .iter()
        .filter_map(|&idx| {
            match &m.timeline().segments()[idx] {
                Segment::Narration { text, .. } => Some(text.clone()),
                _ => None,
            }
        })
        .collect();

    // Synthesize each narration.
    let mut clips = Vec::with_capacity(total);
    let mut durations = Vec::with_capacity(total);

    for (i, text) in texts.iter().enumerate() {
        report(progress, BuildProgress::SynthesizingTts {
            current: i,
            total,
        });

        let clip = backend.synthesize(text).map_err(|e| BuildError::Tts {
            segment: i,
            source: e,
        })?;

        durations.push(clip.duration());
        clips.push(clip);
    }

    // Resolve WPM-estimated durations with actual TTS durations.
    m.resolve_narration_durations(&durations).map_err(|e| {
        BuildError::Config(format!("failed to resolve narration durations: {e}"))
    })?;

    Ok(clips)
}

/// Helper to invoke the progress callback if present.
fn report(progress: &Option<Arc<dyn Fn(BuildProgress) + Send + Sync>>, event: BuildProgress) {
    if let Some(cb) = progress {
        cb(event);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_config_defaults() {
        let config = BuildConfig::new("/tmp/output.mp4", "/tmp/index.html");
        assert_eq!(config.output_path, PathBuf::from("/tmp/output.mp4"));
        assert_eq!(config.html_path, PathBuf::from("/tmp/index.html"));
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(!config.keep_frames);
        assert!(config.progress.is_none());
        assert!(config.voice_backend.is_none());
    }

    #[test]
    fn build_error_display() {
        let err = BuildError::Config("bad config".to_string());
        assert_eq!(format!("{err}"), "bad config");

        let io_err = BuildError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let msg = format!("{io_err}");
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn build_error_display_tts() {
        let err = BuildError::Tts {
            segment: 2,
            source: anyhow::anyhow!("model not loaded"),
        };
        let msg = format!("{err}");
        assert!(msg.contains("TTS synthesis failed"));
        assert!(msg.contains("segment 2"));
        assert!(msg.contains("model not loaded"));
    }

    #[test]
    fn build_error_from_render_error() {
        let render_err = RenderError::OutputDir {
            path: PathBuf::from("/bad"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let build_err: BuildError = render_err.into();
        let msg = format!("{build_err}");
        assert!(msg.contains("render failed"));
    }

    #[test]
    fn build_error_from_ffmpeg_error() {
        let ffmpeg_err = FfmpegError::NotFound;
        let build_err: BuildError = ffmpeg_err.into();
        let msg = format!("{build_err}");
        assert!(msg.contains("ffmpeg failed"));
    }

    #[test]
    fn build_result_fields() {
        let result = BuildResult {
            output_path: PathBuf::from("out.mp4"),
            total_frames: 90,
            duration: 3.0,
        };
        assert_eq!(result.output_path, PathBuf::from("out.mp4"));
        assert_eq!(result.total_frames, 90);
        assert!((result.duration - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn report_with_callback() {
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_clone = flag.clone();
        let cb: Arc<dyn Fn(BuildProgress) + Send + Sync> = Arc::new(move |_| {
            flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        let progress = Some(cb);
        report(&progress, BuildProgress::Encoding);
        assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn report_without_callback() {
        let progress: Option<Arc<dyn Fn(BuildProgress) + Send + Sync>> = None;
        // Should not panic.
        report(&progress, BuildProgress::Encoding);
    }

    // -- synthesize_narrations tests -----------------------------------------

    /// A simple mock VoiceBackend that produces deterministic audio.
    struct MockBackend {
        /// Sample rate of produced clips.
        sample_rate: u32,
        /// Duration in seconds for each synthesized word.
        seconds_per_word: f64,
    }

    impl VoiceBackend for MockBackend {
        fn synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error> {
            let words = text.split_whitespace().count().max(1) as f64;
            let duration = words * self.seconds_per_word;
            let num_samples = (duration * self.sample_rate as f64) as usize;
            Ok(AudioClip {
                data: vec![0.42; num_samples],
                duration,
                sample_rate: self.sample_rate,
                channels: 1,
            })
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn synthesize_narrations_resolves_durations() {
        let mut m = M::new();
        m.narrate("hello world");          // WPM estimate: 2 words * 60/150 = 0.8s
        m.wait(0.5);
        m.narrate("goodbye");              // WPM estimate: 1 word * 60/150 = 0.4s

        // Before synthesis: total = 0.8 + 0.5 + 0.4 = 1.7s
        assert!((m.timeline().total_duration() - 1.7).abs() < 1e-10);

        let backend = MockBackend {
            sample_rate: 48000,
            seconds_per_word: 0.5,
        };

        let clips = synthesize_narrations(&mut m, &backend, &None).unwrap();

        // "hello world" = 2 words * 0.5 = 1.0s
        // "goodbye" = 1 word * 0.5 = 0.5s
        assert_eq!(clips.len(), 2);
        assert!((clips[0].duration() - 1.0).abs() < 1e-10);
        assert!((clips[1].duration() - 0.5).abs() < 1e-10);

        // After resolution: total = 1.0 + 0.5 + 0.5 = 2.0s
        assert!((m.timeline().total_duration() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn synthesize_narrations_no_narrations() {
        let mut m = M::new();
        m.wait(1.0);

        let backend = MockBackend {
            sample_rate: 48000,
            seconds_per_word: 0.5,
        };

        let clips = synthesize_narrations(&mut m, &backend, &None).unwrap();
        assert!(clips.is_empty());

        // Duration unchanged
        assert!((m.timeline().total_duration() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn synthesize_narrations_reports_progress() {
        let mut m = M::new();
        m.narrate("one");
        m.narrate("two");

        let backend = MockBackend {
            sample_rate: 48000,
            seconds_per_word: 0.5,
        };

        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count_clone = count.clone();
        let cb: Arc<dyn Fn(BuildProgress) + Send + Sync> = Arc::new(move |event| {
            if matches!(event, BuildProgress::SynthesizingTts { .. }) {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        });

        let clips = synthesize_narrations(&mut m, &backend, &Some(cb)).unwrap();
        assert_eq!(clips.len(), 2);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    /// A mock backend that always fails synthesis.
    struct FailingBackend;

    impl VoiceBackend for FailingBackend {
        fn synthesize(&self, _text: &str) -> Result<AudioClip, anyhow::Error> {
            anyhow::bail!("synthesis engine crashed")
        }

        fn name(&self) -> &str {
            "failing-mock"
        }
    }

    #[test]
    fn synthesize_narrations_propagates_error() {
        let mut m = M::new();
        m.narrate("hello");

        let backend = FailingBackend;
        let result = synthesize_narrations(&mut m, &backend, &None);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BuildError::Tts { segment: 0, .. }));
        let msg = format!("{err}");
        assert!(msg.contains("synthesis engine crashed"));
    }
}
