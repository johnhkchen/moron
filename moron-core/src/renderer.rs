//! Frame rendering loop: iterate the timeline and capture each frame as a PNG.
//!
//! The renderer is the orchestrator of the image-sequence pipeline. Given a built
//! scene ([`M`] with a recorded timeline), it:
//!
//! 1. Computes total frame count from the timeline's duration and FPS
//! 2. Launches a headless Chromium bridge to the React rendering layer
//! 3. For each frame: computes the timestamp, builds a [`FrameState`], serializes
//!    it to JSON, sends it through the bridge, and saves the resulting PNG
//! 4. Outputs numbered files: `frame_000000.png`, `frame_000001.png`, etc.
//! 5. Reports progress (frame N of M)
//!
//! The output directory of numbered PNGs is consumed by the FFmpeg encoding
//! pipeline (S-005) to produce the final video.

use std::path::{Path, PathBuf};

use crate::chromium::{BridgeConfig, BridgeError, ChromiumBridge};
use crate::facade::M;
use crate::frame::compute_frame_state;

// ---------------------------------------------------------------------------
// RenderError
// ---------------------------------------------------------------------------

/// Errors produced by the frame rendering loop.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The Chromium bridge failed (launch, capture, or close).
    #[error("bridge error: {0}")]
    Bridge(#[from] BridgeError),

    /// The output directory could not be created.
    #[error("failed to create output directory {}: {source}", path.display())]
    OutputDir {
        /// The directory path that failed.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// FrameState serialization to JSON failed.
    #[error("failed to serialize frame {frame}: {source}")]
    Serialize {
        /// The frame number that failed.
        frame: u32,
        /// The underlying serialization error.
        source: serde_json::Error,
    },

    /// Writing a PNG file to disk failed.
    #[error("failed to write frame {frame} to {}: {source}", path.display())]
    WriteFrame {
        /// The frame number that failed.
        frame: u32,
        /// The file path that failed.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

// ---------------------------------------------------------------------------
// RenderProgress
// ---------------------------------------------------------------------------

/// Progress report emitted once per rendered frame.
pub struct RenderProgress {
    /// The frame that was just rendered (0-indexed).
    pub current_frame: u32,
    /// Total number of frames to render.
    pub total_frames: u32,
}

// ---------------------------------------------------------------------------
// RenderConfig
// ---------------------------------------------------------------------------

/// Configuration for the frame rendering loop.
pub struct RenderConfig {
    /// Directory where numbered PNG files will be written.
    pub output_dir: PathBuf,
    /// Configuration for launching the Chromium bridge.
    pub bridge_config: BridgeConfig,
    /// Optional progress callback, called after each frame is rendered.
    /// If `None`, progress is printed to stderr.
    pub progress: Option<Box<dyn Fn(RenderProgress)>>,
}

impl RenderConfig {
    /// Create a new render configuration.
    ///
    /// `output_dir` is the directory where `frame_NNNNNN.png` files will be written.
    /// `bridge_config` configures the headless Chromium bridge (viewport size, HTML path, etc.).
    /// Progress defaults to printing to stderr; use the `progress` field to override.
    pub fn new(output_dir: impl Into<PathBuf>, bridge_config: BridgeConfig) -> Self {
        Self {
            output_dir: output_dir.into(),
            bridge_config,
            progress: None,
        }
    }
}

// ---------------------------------------------------------------------------
// RenderResult
// ---------------------------------------------------------------------------

/// Summary returned after a successful render.
pub struct RenderResult {
    /// Number of frames rendered.
    pub total_frames: u32,
    /// Directory containing the rendered PNG files.
    pub output_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// render — the main entry point
// ---------------------------------------------------------------------------

/// Render a scene's timeline to a sequence of numbered PNG files.
///
/// Given a built scene (`M` with a recorded timeline) and a [`RenderConfig`],
/// this function iterates through the timeline at the target FPS, captures each
/// frame via the Chromium bridge, and saves the resulting PNGs to disk.
///
/// # Empty timelines
///
/// If the timeline has zero duration (no segments), the function returns
/// immediately with `total_frames: 0`. No output directory is created and
/// no Chrome process is launched.
///
/// # Errors
///
/// Returns [`RenderError`] if:
/// - The output directory cannot be created
/// - The Chromium bridge fails to launch
/// - Any frame fails to serialize, capture, or write to disk
///
/// On error, any frames already written to disk are preserved (not cleaned up).
pub async fn render(m: &M, config: RenderConfig) -> Result<RenderResult, RenderError> {
    let total_frames = m.timeline().total_frames();
    let fps = m.timeline().fps();
    let output_dir = config.output_dir;

    // Empty timeline: no frames to render.
    if total_frames == 0 {
        return Ok(RenderResult {
            total_frames: 0,
            output_dir,
        });
    }

    // Create the output directory if it doesn't exist.
    std::fs::create_dir_all(&output_dir).map_err(|e| RenderError::OutputDir {
        path: output_dir.clone(),
        source: e,
    })?;

    // Launch the Chromium bridge.
    let bridge = ChromiumBridge::launch(config.bridge_config).await?;

    // Render each frame.
    let result = render_frames(m, &bridge, &output_dir, total_frames, fps, &config.progress).await;

    // Always attempt to close the bridge, even if rendering failed.
    // We prioritize the render error over a close error.
    let close_result = bridge.close().await;

    match result {
        Ok(()) => {
            // If rendering succeeded but close failed, report the close error.
            close_result?;
            Ok(RenderResult {
                total_frames,
                output_dir,
            })
        }
        Err(e) => {
            // Rendering failed. Ignore close errors; the render error is more important.
            let _ = close_result;
            Err(e)
        }
    }
}

/// Inner loop: render all frames. Separated from `render` so the bridge
/// cleanup logic in `render` stays clean.
async fn render_frames(
    m: &M,
    bridge: &ChromiumBridge,
    output_dir: &Path,
    total_frames: u32,
    fps: u32,
    progress: &Option<Box<dyn Fn(RenderProgress)>>,
) -> Result<(), RenderError> {
    for frame_num in 0..total_frames {
        let time = frame_num as f64 / fps as f64;

        // Compute the visual state at this timestamp.
        let state = compute_frame_state(m, time);

        // Serialize to JSON for the React bridge.
        let json = serde_json::to_string(&state).map_err(|e| RenderError::Serialize {
            frame: frame_num,
            source: e,
        })?;

        // Capture the rendered frame as PNG bytes.
        let png_bytes = bridge.capture_frame(&json).await?;

        // Write the PNG to disk.
        let path = frame_path(output_dir, frame_num);
        std::fs::write(&path, &png_bytes).map_err(|e| RenderError::WriteFrame {
            frame: frame_num,
            path: path.clone(),
            source: e,
        })?;

        // Report progress.
        report_progress(progress, frame_num, total_frames);
    }

    Ok(())
}

/// Report progress for a completed frame.
fn report_progress(
    callback: &Option<Box<dyn Fn(RenderProgress)>>,
    current_frame: u32,
    total_frames: u32,
) {
    let progress = RenderProgress {
        current_frame,
        total_frames,
    };

    if let Some(cb) = callback {
        cb(progress);
    } else {
        eprintln!(
            "Rendering frame {} of {} ({:.0}%)",
            current_frame + 1,
            total_frames,
            (current_frame + 1) as f64 / total_frames as f64 * 100.0,
        );
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Format the output path for a given frame number.
///
/// Produces: `{output_dir}/frame_000000.png`, `{output_dir}/frame_000042.png`, etc.
fn frame_path(output_dir: &Path, frame_num: u32) -> PathBuf {
    output_dir.join(format!("frame_{:06}.png", frame_num))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromium::BridgeConfig;
    use crate::facade::M;

    #[test]
    fn frame_path_formatting() {
        let dir = Path::new("/tmp/out");
        assert_eq!(
            frame_path(dir, 0),
            PathBuf::from("/tmp/out/frame_000000.png")
        );
        assert_eq!(
            frame_path(dir, 42),
            PathBuf::from("/tmp/out/frame_000042.png")
        );
        assert_eq!(
            frame_path(dir, 999999),
            PathBuf::from("/tmp/out/frame_999999.png")
        );
    }

    #[test]
    fn frame_path_with_nested_dir() {
        let dir = Path::new("/home/user/project/output/frames");
        assert_eq!(
            frame_path(dir, 1),
            PathBuf::from("/home/user/project/output/frames/frame_000001.png")
        );
    }

    #[test]
    fn render_config_new() {
        let config = RenderConfig::new("/tmp/out", BridgeConfig::new("/tmp/index.html"));
        assert_eq!(config.output_dir, PathBuf::from("/tmp/out"));
        assert!(config.progress.is_none());
    }

    #[test]
    fn render_config_with_progress_callback() {
        let mut called = false;
        let config = RenderConfig {
            output_dir: PathBuf::from("/tmp/out"),
            bridge_config: BridgeConfig::new("/tmp/index.html"),
            progress: Some(Box::new(|_p| {
                // callback exists
            })),
        };
        assert!(config.progress.is_some());

        // Invoke the callback to verify it works.
        if let Some(cb) = &config.progress {
            cb(RenderProgress {
                current_frame: 0,
                total_frames: 10,
            });
            called = true;
        }
        assert!(called);
    }

    #[test]
    fn render_progress_fields() {
        let p = RenderProgress {
            current_frame: 5,
            total_frames: 100,
        };
        assert_eq!(p.current_frame, 5);
        assert_eq!(p.total_frames, 100);
    }

    #[test]
    fn render_result_fields() {
        let r = RenderResult {
            total_frames: 30,
            output_dir: PathBuf::from("/tmp/out"),
        };
        assert_eq!(r.total_frames, 30);
        assert_eq!(r.output_dir, PathBuf::from("/tmp/out"));
    }

    #[test]
    fn render_error_display_messages() {
        let err = RenderError::OutputDir {
            path: PathBuf::from("/bad/path"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let msg = format!("{err}");
        assert!(msg.contains("/bad/path"));
        assert!(msg.contains("denied"));

        let err = RenderError::WriteFrame {
            frame: 42,
            path: PathBuf::from("/tmp/frame_000042.png"),
            source: std::io::Error::new(std::io::ErrorKind::Other, "disk full"),
        };
        let msg = format!("{err}");
        assert!(msg.contains("42"));
        assert!(msg.contains("disk full"));
    }

    #[test]
    fn empty_timeline_has_zero_frames() {
        let m = M::new();
        assert_eq!(m.timeline().total_frames(), 0);
    }

    #[test]
    fn non_empty_timeline_frame_count() {
        let mut m = M::new();
        m.wait(1.0); // 1 second at 30 fps = 30 frames
        assert_eq!(m.timeline().total_frames(), 30);
        assert_eq!(m.timeline().fps(), 30);
    }

    #[test]
    fn time_computation_matches_frame_at() {
        let mut m = M::new();
        m.wait(2.0); // 2 seconds at 30 fps = 60 frames

        let fps = m.timeline().fps();
        // Frame 0 -> time 0.0
        assert_eq!(0.0_f64 / fps as f64, 0.0);
        // Frame 30 -> time 1.0
        assert!((30.0_f64 / fps as f64 - 1.0).abs() < f64::EPSILON);
        // Frame 59 -> time ~1.967
        let time_59 = 59.0_f64 / fps as f64;
        assert_eq!(m.timeline().frame_at(time_59), 59);
    }

    #[test]
    fn report_progress_with_callback() {
        let cb: Box<dyn Fn(RenderProgress)> = Box::new(|p| {
            // Cannot capture &mut in Fn, but we can verify the callback is invoked
            // by checking that it doesn't panic.
            assert!(p.current_frame < p.total_frames);
        });
        let callback = Some(cb);

        // Should not panic.
        report_progress(&callback, 0, 10);
        report_progress(&callback, 9, 10);
    }

    #[test]
    fn report_progress_without_callback() {
        // With no callback, progress is printed to stderr.
        // This test just verifies it doesn't panic.
        let callback: Option<Box<dyn Fn(RenderProgress)>> = None;
        report_progress(&callback, 0, 10);
        report_progress(&callback, 9, 10);
    }
}
