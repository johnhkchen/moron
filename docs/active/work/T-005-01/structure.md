# T-005-01 Structure: FFmpeg Encoding

## Files Modified

### moron-core/src/ffmpeg.rs (Replace Stub)

Complete rewrite. New contents:

```
Module doc comment (existing, keep)
Imports: std::path, std::process::Command, thiserror

--- FfmpegError enum ---
  NotFound           -- ffmpeg not on PATH
  InvalidInput       -- bad config (missing dir, no frames, etc.)
  EncodeFailed       -- ffmpeg exited with error

--- EncodeConfig struct ---
  input_dir: PathBuf
  output_path: PathBuf
  fps: u32
  width: u32
  height: u32
  crf: u8
  + new() constructor with defaults
  + builder-style setters (fps, resolution, crf)

--- Public functions ---
  detect_ffmpeg() -> Result<(), FfmpegError>
  encode(config: &EncodeConfig) -> Result<(), FfmpegError>

--- Private helpers ---
  build_ffmpeg_args(config: &EncodeConfig) -> Vec<String>
  validate_input(config: &EncodeConfig) -> Result<(), FfmpegError>

--- Tests module ---
  test_encode_config_defaults
  test_encode_config_builder_methods
  test_encode_config_new
  test_ffmpeg_error_display
  test_build_ffmpeg_args
  test_build_ffmpeg_args_custom
  test_validate_input_missing_dir
  test_validate_input_empty_dir
  test_validate_input_valid_dir (with tempdir + fake PNGs)
  test_detect_ffmpeg (conditional on ffmpeg availability)
```

### moron-core/src/lib.rs (Modify)

Add re-exports:
```rust
pub use ffmpeg::{detect_ffmpeg, encode as encode_video, EncodeConfig, FfmpegError};
```

Add to prelude:
```rust
pub use crate::ffmpeg::{detect_ffmpeg, encode as encode_video, EncodeConfig, FfmpegError};
```

Note: `encode` is re-exported as `encode_video` to avoid ambiguity with other potential `encode` functions at crate root.

## Module Boundaries

- `ffmpeg.rs` is self-contained: no dependencies on other moron-core modules
- It consumes the output of `renderer.rs` (the frame directory) but has no code dependency on it
- The coupling is through the file convention: `frame_%06d.png` naming pattern
- `EncodeConfig` is the sole interface between the caller and the FFmpeg pipeline

## Public Interface

```rust
// Detection
pub fn detect_ffmpeg() -> Result<(), FfmpegError>;

// Encoding
pub fn encode(config: &EncodeConfig) -> Result<(), FfmpegError>;

// Config
pub struct EncodeConfig {
    pub input_dir: PathBuf,
    pub output_path: PathBuf,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub crf: u8,
}

// Errors
pub enum FfmpegError {
    NotFound,
    InvalidInput { reason: String },
    EncodeFailed { message: String, stderr: String },
}
```

## No New Dependencies

All implementation uses:
- `std::process::Command` (stdlib)
- `std::path::{Path, PathBuf}` (stdlib)
- `std::fs` (stdlib)
- `thiserror` (already in Cargo.toml)

No new crate dependencies are needed.
