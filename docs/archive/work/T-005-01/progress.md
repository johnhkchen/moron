# T-005-01 Progress: FFmpeg Encoding

## Status: Complete

All steps executed. No deviations from plan.

## Completed Steps

### Step 1: FfmpegError Enum
Implemented three-variant error enum with thiserror:
- `NotFound` -- clear message with install instructions and URL
- `InvalidInput { reason }` -- descriptive validation errors
- `EncodeFailed { message, stderr }` -- captures FFmpeg exit code and stderr

### Step 2: EncodeConfig Struct
Implemented with:
- `new(input_dir, output_path)` constructor with defaults (30fps, 1920x1080, CRF 23)
- Builder-style `fps()`, `resolution()`, `crf()` methods
- Public constants: `DEFAULT_CRF`, `DEFAULT_WIDTH`, `DEFAULT_HEIGHT`, `DEFAULT_FPS`

### Step 3: detect_ffmpeg()
Public function runs `ffmpeg -version` via `std::process::Command`.
Returns `Ok(())` or `Err(FfmpegError::NotFound)`.

### Step 4: validate_input() Helper
Private function validates:
- Input directory exists and is a directory
- Contains at least one `frame_*.png` file
- CRF is 0-51
- FPS is non-zero
- Resolution is non-zero

### Step 5: build_ffmpeg_args() Helper
Private function produces the full argument vector:
`-y -framerate {fps} -i {dir}/frame_%06d.png -c:v libx264 -pix_fmt yuv420p -crf {crf} -vf scale={w}:{h} {output}`

### Step 6: encode() Function
Main public function: validate_input -> detect_ffmpeg -> build_args -> Command::new("ffmpeg").args().output() -> check status.

### Step 7: Re-exports in lib.rs
Added to crate root and prelude:
`pub use ffmpeg::{detect_ffmpeg, encode as encode_video, EncodeConfig, FfmpegError};`

### Step 8: Unit Tests (19 tests)
- EncodeConfig: defaults, builder methods, chaining
- FfmpegError: display messages for all variants
- build_ffmpeg_args: default and custom configs
- validate_input: missing dir, not-a-dir, empty dir, valid dir, zero fps, zero resolution
- has_frame_files: with frames, without frames, nonexistent dir
- detect_ffmpeg: conditional (works whether ffmpeg installed or not)
- Constants: all defaults correct

### Step 9: Final Verification
- `cargo check` -- clean
- `cargo test` -- 96 tests pass (69 moron-core, 14 techniques, 5 themes, 5 voice, 5 integration, 3 ignored doc-tests)
- `cargo clippy` -- no warnings
- Ticket frontmatter updated: status: done, phase: done

## Files Modified
- `moron-core/src/ffmpeg.rs` -- replaced stub with full implementation (600 lines)
- `moron-core/src/lib.rs` -- added ffmpeg re-exports to crate root and prelude

## Deviations from Plan
None.
