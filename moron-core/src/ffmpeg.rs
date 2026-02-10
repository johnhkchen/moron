//! FFmpeg pipeline: frame encoding, muxing, and output format handling.
//!
//! This module wraps the FFmpeg CLI to encode a directory of numbered PNG frames
//! into an H.264/MP4 video. It uses [`std::process::Command`] to spawn FFmpeg as
//! a subprocess -- no Rust FFmpeg bindings are needed.
//!
//! # Usage
//!
//! ```ignore
//! use moron_core::ffmpeg::{detect_ffmpeg, encode, EncodeConfig};
//!
//! // Check FFmpeg is available (optional -- encode() does this automatically).
//! detect_ffmpeg()?;
//!
//! let config = EncodeConfig::new("frames/", "output.mp4");
//! encode(&config)?;
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// FfmpegError
// ---------------------------------------------------------------------------

/// Errors produced by the FFmpeg encoding pipeline.
#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    /// FFmpeg binary was not found on the system PATH.
    #[error(
        "FFmpeg not found. Install FFmpeg and ensure it is on your PATH. \
         (https://ffmpeg.org/download.html)"
    )]
    NotFound,

    /// The encoding configuration is invalid.
    #[error("invalid input: {reason}")]
    InvalidInput {
        /// Description of what is wrong.
        reason: String,
    },

    /// FFmpeg exited with a non-zero status.
    #[error("FFmpeg encoding failed: {message}")]
    EncodeFailed {
        /// Summary of the failure.
        message: String,
        /// Captured stderr output from FFmpeg.
        stderr: String,
    },
}

// ---------------------------------------------------------------------------
// EncodeConfig
// ---------------------------------------------------------------------------

/// Default CRF (Constant Rate Factor) for H.264 encoding.
///
/// CRF 23 is FFmpeg's default -- visually good quality at reasonable file size.
/// Range: 0 (lossless) to 51 (worst quality). Lower values = better quality.
pub const DEFAULT_CRF: u8 = 23;

/// Default output width in pixels.
pub const DEFAULT_WIDTH: u32 = 1920;

/// Default output height in pixels.
pub const DEFAULT_HEIGHT: u32 = 1080;

/// Default frames per second.
pub const DEFAULT_FPS: u32 = 30;

/// Configuration for the FFmpeg encoding pipeline.
pub struct EncodeConfig {
    /// Directory containing numbered PNG frames (`frame_000000.png`, etc.).
    pub input_dir: PathBuf,
    /// Path to the output `.mp4` file.
    pub output_path: PathBuf,
    /// Frames per second for the output video.
    pub fps: u32,
    /// Output video width in pixels.
    pub width: u32,
    /// Output video height in pixels.
    pub height: u32,
    /// H.264 Constant Rate Factor (0-51). Lower = better quality.
    pub crf: u8,
}

impl EncodeConfig {
    /// Create a new encoding configuration with sensible defaults.
    ///
    /// - FPS: 30
    /// - Resolution: 1920x1080
    /// - CRF: 23
    pub fn new(input_dir: impl Into<PathBuf>, output_path: impl Into<PathBuf>) -> Self {
        Self {
            input_dir: input_dir.into(),
            output_path: output_path.into(),
            fps: DEFAULT_FPS,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            crf: DEFAULT_CRF,
        }
    }

    /// Set the frames per second.
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Set the output resolution.
    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the CRF quality value (0-51).
    pub fn crf(mut self, crf: u8) -> Self {
        self.crf = crf;
        self
    }
}

// ---------------------------------------------------------------------------
// detect_ffmpeg
// ---------------------------------------------------------------------------

/// Check that FFmpeg is available on the system PATH.
///
/// Runs `ffmpeg -version` and checks for a successful exit. Returns
/// `Ok(())` if FFmpeg is found and runs, or `Err(FfmpegError::NotFound)`
/// if the binary is missing or fails to execute.
///
/// This is called automatically by [`encode`], but can be used for
/// early detection (e.g., at application startup).
pub fn detect_ffmpeg() -> Result<(), FfmpegError> {
    let result = Command::new("ffmpeg").arg("-version").output();

    match result {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(FfmpegError::NotFound),
    }
}

// ---------------------------------------------------------------------------
// encode
// ---------------------------------------------------------------------------

/// Encode a directory of numbered PNG frames into an H.264/MP4 video.
///
/// This function:
/// 1. Validates the input configuration (directory exists, contains frames)
/// 2. Detects FFmpeg on the system PATH
/// 3. Spawns FFmpeg as a subprocess with the appropriate arguments
/// 4. Waits for encoding to complete
///
/// The input directory must contain files matching the pattern
/// `frame_000000.png`, `frame_000001.png`, etc. (6-digit zero-padded).
///
/// # Errors
///
/// Returns [`FfmpegError`] if:
/// - The input directory does not exist or contains no frame PNGs
/// - FFmpeg is not installed or not on PATH
/// - FFmpeg exits with a non-zero status
pub fn encode(config: &EncodeConfig) -> Result<(), FfmpegError> {
    validate_input(config)?;
    detect_ffmpeg()?;

    let args = build_ffmpeg_args(config);

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| FfmpegError::EncodeFailed {
            message: format!("failed to spawn FFmpeg process: {e}"),
            stderr: String::new(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output
            .status
            .code()
            .map(|c| format!("exit code {c}"))
            .unwrap_or_else(|| "killed by signal".to_string());

        return Err(FfmpegError::EncodeFailed {
            message: format!("FFmpeg exited with {code}"),
            stderr,
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Validate the encoding configuration before running FFmpeg.
fn validate_input(config: &EncodeConfig) -> Result<(), FfmpegError> {
    // Check that the input directory exists.
    if !config.input_dir.exists() {
        return Err(FfmpegError::InvalidInput {
            reason: format!(
                "input directory does not exist: {}",
                config.input_dir.display()
            ),
        });
    }

    if !config.input_dir.is_dir() {
        return Err(FfmpegError::InvalidInput {
            reason: format!(
                "input path is not a directory: {}",
                config.input_dir.display()
            ),
        });
    }

    // Check that the directory contains at least one frame PNG.
    let has_frames = has_frame_files(&config.input_dir);
    if !has_frames {
        return Err(FfmpegError::InvalidInput {
            reason: format!(
                "no frame_*.png files found in {}",
                config.input_dir.display()
            ),
        });
    }

    // Validate CRF range.
    if config.crf > 51 {
        return Err(FfmpegError::InvalidInput {
            reason: format!("CRF must be 0-51, got {}", config.crf),
        });
    }

    // Validate FPS.
    if config.fps == 0 {
        return Err(FfmpegError::InvalidInput {
            reason: "FPS must be greater than 0".to_string(),
        });
    }

    // Validate resolution.
    if config.width == 0 || config.height == 0 {
        return Err(FfmpegError::InvalidInput {
            reason: format!(
                "resolution must be non-zero, got {}x{}",
                config.width, config.height
            ),
        });
    }

    Ok(())
}

/// Check if a directory contains any files matching the `frame_*.png` pattern.
fn has_frame_files(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };

    entries
        .filter_map(|e| e.ok())
        .any(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.starts_with("frame_") && name.ends_with(".png")
        })
}

/// Build the FFmpeg command-line arguments for encoding.
///
/// Produces arguments equivalent to:
/// ```text
/// ffmpeg -y -framerate {fps} -i {input_dir}/frame_%06d.png \
///   -c:v libx264 -pix_fmt yuv420p -crf {crf} \
///   -vf scale={width}:{height} {output_path}
/// ```
fn build_ffmpeg_args(config: &EncodeConfig) -> Vec<String> {
    let input_pattern = config
        .input_dir
        .join("frame_%06d.png")
        .to_string_lossy()
        .to_string();

    let output = config.output_path.to_string_lossy().to_string();

    vec![
        // Overwrite output file without asking.
        "-y".to_string(),
        // Input framerate.
        "-framerate".to_string(),
        config.fps.to_string(),
        // Input file pattern.
        "-i".to_string(),
        input_pattern,
        // H.264 codec.
        "-c:v".to_string(),
        "libx264".to_string(),
        // Pixel format for maximum compatibility (required for some players).
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        // Quality setting.
        "-crf".to_string(),
        config.crf.to_string(),
        // Output resolution.
        "-vf".to_string(),
        format!("scale={}:{}", config.width, config.height),
        // Output file path.
        output,
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // -- EncodeConfig tests ------------------------------------------------

    #[test]
    fn encode_config_defaults() {
        let config = EncodeConfig::new("/tmp/frames", "/tmp/output.mp4");
        assert_eq!(config.input_dir, PathBuf::from("/tmp/frames"));
        assert_eq!(config.output_path, PathBuf::from("/tmp/output.mp4"));
        assert_eq!(config.fps, 30);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.crf, 23);
    }

    #[test]
    fn encode_config_builder_fps() {
        let config = EncodeConfig::new("/tmp/frames", "/tmp/out.mp4").fps(60);
        assert_eq!(config.fps, 60);
    }

    #[test]
    fn encode_config_builder_resolution() {
        let config = EncodeConfig::new("/tmp/frames", "/tmp/out.mp4").resolution(3840, 2160);
        assert_eq!(config.width, 3840);
        assert_eq!(config.height, 2160);
    }

    #[test]
    fn encode_config_builder_crf() {
        let config = EncodeConfig::new("/tmp/frames", "/tmp/out.mp4").crf(18);
        assert_eq!(config.crf, 18);
    }

    #[test]
    fn encode_config_builder_chaining() {
        let config = EncodeConfig::new("/tmp/frames", "/tmp/out.mp4")
            .fps(60)
            .resolution(1280, 720)
            .crf(28);
        assert_eq!(config.fps, 60);
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.crf, 28);
    }

    // -- FfmpegError display tests -----------------------------------------

    #[test]
    fn error_display_not_found() {
        let err = FfmpegError::NotFound;
        let msg = format!("{err}");
        assert!(msg.contains("FFmpeg not found"));
        assert!(msg.contains("PATH"));
    }

    #[test]
    fn error_display_invalid_input() {
        let err = FfmpegError::InvalidInput {
            reason: "directory does not exist".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("invalid input"));
        assert!(msg.contains("directory does not exist"));
    }

    #[test]
    fn error_display_encode_failed() {
        let err = FfmpegError::EncodeFailed {
            message: "exit code 1".to_string(),
            stderr: "No such file".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("FFmpeg encoding failed"));
        assert!(msg.contains("exit code 1"));
    }

    // -- build_ffmpeg_args tests -------------------------------------------

    #[test]
    fn build_args_default_config() {
        let config = EncodeConfig::new("/tmp/frames", "/tmp/output.mp4");
        let args = build_ffmpeg_args(&config);

        assert!(args.contains(&"-y".to_string()));
        assert!(args.contains(&"-framerate".to_string()));
        assert!(args.contains(&"30".to_string()));
        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert!(args.contains(&"-pix_fmt".to_string()));
        assert!(args.contains(&"yuv420p".to_string()));
        assert!(args.contains(&"-crf".to_string()));
        assert!(args.contains(&"23".to_string()));
        assert!(args.contains(&"-vf".to_string()));
        assert!(args.contains(&"scale=1920:1080".to_string()));

        // Input pattern should end with frame_%06d.png
        let input_idx = args.iter().position(|a| a == "-i").unwrap();
        let input_pattern = &args[input_idx + 1];
        assert!(input_pattern.ends_with("frame_%06d.png"));
        assert!(input_pattern.starts_with("/tmp/frames"));

        // Output should be the last argument
        assert_eq!(args.last().unwrap(), "/tmp/output.mp4");
    }

    #[test]
    fn build_args_custom_config() {
        let config = EncodeConfig::new("/home/user/render", "/home/user/video.mp4")
            .fps(60)
            .resolution(1280, 720)
            .crf(18);
        let args = build_ffmpeg_args(&config);

        assert!(args.contains(&"60".to_string()));
        assert!(args.contains(&"18".to_string()));
        assert!(args.contains(&"scale=1280:720".to_string()));
        assert_eq!(args.last().unwrap(), "/home/user/video.mp4");
    }

    // -- validate_input tests ----------------------------------------------

    #[test]
    fn validate_input_missing_dir() {
        let config = EncodeConfig::new("/nonexistent/path/xyz", "/tmp/out.mp4");
        let result = validate_input(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FfmpegError::InvalidInput { .. }));
        let msg = format!("{err}");
        assert!(msg.contains("does not exist"));
    }

    #[test]
    fn validate_input_not_a_directory() {
        // Create a temporary file (not a directory)
        let tmp = std::env::temp_dir().join("moron_test_not_a_dir.txt");
        fs::write(&tmp, "not a dir").unwrap();

        let config = EncodeConfig::new(&tmp, "/tmp/out.mp4");
        let result = validate_input(&config);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("not a directory"));

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn validate_input_empty_dir() {
        let tmp = std::env::temp_dir().join("moron_test_empty_frames");
        fs::create_dir_all(&tmp).unwrap();

        // Remove any leftover frame files
        if let Ok(entries) = fs::read_dir(&tmp) {
            for entry in entries.flatten() {
                fs::remove_file(entry.path()).ok();
            }
        }

        let config = EncodeConfig::new(&tmp, "/tmp/out.mp4");
        let result = validate_input(&config);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("no frame_*.png"));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn validate_input_valid_dir() {
        let tmp = std::env::temp_dir().join("moron_test_valid_frames");
        fs::create_dir_all(&tmp).unwrap();

        // Create a fake frame file
        fs::write(tmp.join("frame_000000.png"), &[0u8; 4]).unwrap();

        let config = EncodeConfig::new(&tmp, "/tmp/out.mp4");
        let result = validate_input(&config);
        assert!(result.is_ok());

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn validate_input_zero_fps() {
        let tmp = std::env::temp_dir().join("moron_test_zero_fps");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("frame_000000.png"), &[0u8; 4]).unwrap();

        let mut config = EncodeConfig::new(&tmp, "/tmp/out.mp4");
        config.fps = 0;
        let result = validate_input(&config);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("FPS"));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn validate_input_zero_resolution() {
        let tmp = std::env::temp_dir().join("moron_test_zero_res");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("frame_000000.png"), &[0u8; 4]).unwrap();

        let mut config = EncodeConfig::new(&tmp, "/tmp/out.mp4");
        config.width = 0;
        let result = validate_input(&config);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("non-zero"));

        fs::remove_dir_all(&tmp).ok();
    }

    // -- has_frame_files tests ---------------------------------------------

    #[test]
    fn has_frame_files_with_frames() {
        let tmp = std::env::temp_dir().join("moron_test_has_frames");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("frame_000000.png"), &[0u8; 4]).unwrap();
        fs::write(tmp.join("frame_000001.png"), &[0u8; 4]).unwrap();

        assert!(has_frame_files(&tmp));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn has_frame_files_without_frames() {
        let tmp = std::env::temp_dir().join("moron_test_no_frames");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("other_file.txt"), "hello").unwrap();

        assert!(!has_frame_files(&tmp));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn has_frame_files_nonexistent_dir() {
        assert!(!has_frame_files(Path::new("/nonexistent/dir")));
    }

    // -- detect_ffmpeg tests -----------------------------------------------

    #[test]
    fn detect_ffmpeg_runs() {
        // This test verifies behavior regardless of whether FFmpeg is installed.
        // If FFmpeg is installed, it should succeed; if not, it should return NotFound.
        let result = detect_ffmpeg();
        match result {
            Ok(()) => {
                // FFmpeg is available -- great
            }
            Err(FfmpegError::NotFound) => {
                // FFmpeg is not installed -- that's fine for CI
            }
            Err(e) => panic!("unexpected error variant: {e}"),
        }
    }

    // -- Constants tests ---------------------------------------------------

    #[test]
    fn default_constants() {
        assert_eq!(DEFAULT_CRF, 23);
        assert_eq!(DEFAULT_WIDTH, 1920);
        assert_eq!(DEFAULT_HEIGHT, 1080);
        assert_eq!(DEFAULT_FPS, 30);
    }
}
