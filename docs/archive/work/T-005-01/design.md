# T-005-01 Design: FFmpeg Encoding

## Decision Summary

Implement a synchronous FFmpeg encoding module using `std::process::Command` with an `EncodeConfig` struct, `FfmpegError` enum, and a single `encode()` function. FFmpeg detection via `Command::new("ffmpeg").arg("-version")`.

## Option A: Single encode() Function (Chosen)

A flat module with:
- `EncodeConfig` struct for all encoding parameters
- `FfmpegError` enum with thiserror
- `encode(config: &EncodeConfig) -> Result<(), FfmpegError>` top-level function
- `detect_ffmpeg() -> Result<(), FfmpegError>` helper

Pros:
- Matches the existing codebase pattern (renderer.rs has `render()` as top-level fn)
- Simple, no unnecessary abstraction
- Easy for LLM-generated code to call

Cons:
- If we ever need streaming/pipe mode, we'd add a second function

## Option B: FfmpegEncoder Struct with Methods

An `FfmpegEncoder` struct that holds config and provides `encode()`, `detect()`, etc.

Pros:
- Could hold state for future pipe-based encoding
- OOP style

Cons:
- Unnecessary indirection for a subprocess wrapper
- Doesn't match the flat function pattern in renderer.rs
- More boilerplate for no benefit

## Option C: Async with tokio::process::Command

Use `tokio::process::Command` for async subprocess management.

Pros:
- Non-blocking; could show progress during encoding
- Consistent with the async renderer

Cons:
- Ticket explicitly says "synchronous (std::process::Command)"
- FFmpeg encoding is a blocking wait anyway
- Adds complexity for no user-facing benefit

## Chosen: Option A

### EncodeConfig

```rust
pub struct EncodeConfig {
    pub input_dir: PathBuf,    // directory containing frame_NNNNNN.png files
    pub output_path: PathBuf,  // path to output .mp4 file
    pub fps: u32,              // frames per second (from timeline)
    pub width: u32,            // output width (default 1920)
    pub height: u32,           // output height (default 1080)
    pub crf: u8,               // quality 0-51, default 23
}
```

Constructor `EncodeConfig::new(input_dir, output_path)` provides defaults:
- fps: 30
- width: 1920, height: 1080
- crf: 23

### FfmpegError

```rust
pub enum FfmpegError {
    NotFound,                              // ffmpeg binary not on PATH
    InvalidInput { reason: String },       // bad config (missing dir, etc.)
    EncodeFailed { message: String, stderr: String }, // ffmpeg process failed
}
```

Using `thiserror::Error` derive, consistent with `RenderError` and `BridgeError`.

### encode() Function

1. Validate input_dir exists and contains at least one frame PNG
2. Call `detect_ffmpeg()?` to verify FFmpeg is available
3. Build the `Command`:
   ```
   ffmpeg -y -framerate {fps} -i {input_dir}/frame_%06d.png \
     -c:v libx264 -pix_fmt yuv420p -crf {crf} \
     -vf scale={width}:{height} {output_path}
   ```
4. Run with `.output()` (captures stdout/stderr)
5. Check exit status; on failure, return `EncodeFailed` with stderr

### detect_ffmpeg() Function

Public function so callers can check upfront:
```rust
pub fn detect_ffmpeg() -> Result<(), FfmpegError>
```
Runs `ffmpeg -version`, checks for successful exit. Returns `FfmpegError::NotFound` on failure.

### Resolution Handling

The `-vf scale=W:H` flag is only added when resolution differs from the input frames.
To keep it simple and always correct, we always include it. FFmpeg handles the no-op case efficiently.

### Re-exports

Add to lib.rs:
```rust
pub use ffmpeg::{encode, EncodeConfig, FfmpegError};
```
And to the prelude.

## Rejected Alternatives

- **Pipe-based encoding**: Out of scope for T-005-01. The ticket says "directory of numbered PNGs."
- **Progress callbacks**: FFmpeg progress parsing is fragile. Can be added in a follow-up ticket.
- **Custom FFmpeg binary path**: Could be added to EncodeConfig later. For now, PATH lookup suffices.
- **Multiple output formats**: H.264/MP4 only, per acceptance criteria.
