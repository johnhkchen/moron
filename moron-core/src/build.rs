//! Build pipeline: orchestrate the full scene-to-video workflow.
//!
//! The [`build_video`] function is the single entry point for producing an `.mp4`
//! from a built scene. It renders frames via the Chromium bridge, encodes video
//! via FFmpeg, assembles and muxes the audio track, and cleans up intermediate
//! files.
//!
//! The CLI (`moron build`) is a thin wrapper around this function.

use std::path::PathBuf;
use std::sync::Arc;

use crate::chromium::BridgeConfig;
use crate::facade::M;
use crate::ffmpeg::{self, EncodeConfig, FfmpegError};
use crate::renderer::{self, RenderConfig, RenderError, RenderProgress};

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
}

// ---------------------------------------------------------------------------
// BuildProgress
// ---------------------------------------------------------------------------

/// Progress updates emitted during the build pipeline.
pub enum BuildProgress {
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
}

impl BuildConfig {
    /// Create a new build config with sensible defaults.
    ///
    /// - Resolution: 1920x1080
    /// - keep_frames: false
    /// - No progress callback
    pub fn new(output_path: impl Into<PathBuf>, html_path: impl Into<PathBuf>) -> Self {
        Self {
            output_path: output_path.into(),
            html_path: html_path.into(),
            width: 1920,
            height: 1080,
            keep_frames: false,
            progress: None,
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
/// 1. Reports scene statistics
/// 2. Creates a temporary directory for intermediate files
/// 3. Renders frames via the Chromium bridge
/// 4. Encodes frames into a video-only `.mp4` via FFmpeg
/// 5. Assembles an audio track from the timeline and writes it as WAV
/// 6. Muxes video + audio into the final `.mp4`
/// 7. Cleans up intermediate files (unless `keep_frames` is set)
///
/// # Errors
///
/// Returns [`BuildError`] if any pipeline stage fails.
pub async fn build_video(m: &M, config: BuildConfig) -> Result<BuildResult, BuildError> {
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

    let audio_clip = ffmpeg::assemble_audio_track(
        m.timeline(),
        moron_voice::DEFAULT_SAMPLE_RATE,
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
}
