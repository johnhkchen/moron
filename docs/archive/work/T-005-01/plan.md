# T-005-01 Plan: FFmpeg Encoding

## Step 1: Implement FfmpegError Enum

Write the error enum with three variants using thiserror.

Verify: `cargo check` passes.

## Step 2: Implement EncodeConfig Struct

Write the config struct with `new()` constructor and builder-style setters.

Verify: `cargo check` passes, unit tests for defaults and builder methods.

## Step 3: Implement detect_ffmpeg()

Write the detection function using `Command::new("ffmpeg").arg("-version")`.

Verify: `cargo check` passes.

## Step 4: Implement validate_input() Helper

Private function to check input_dir exists and contains at least one `frame_*.png` file.

Verify: unit tests with tempdir.

## Step 5: Implement build_ffmpeg_args() Helper

Private function to construct the full argument list for the FFmpeg command.

Verify: unit tests checking argument structure.

## Step 6: Implement encode() Function

The main public function that:
1. Validates input
2. Detects FFmpeg
3. Builds and runs the command
4. Checks exit status

Verify: `cargo check` passes.

## Step 7: Add Re-exports to lib.rs

Add `pub use ffmpeg::...` to both crate root and prelude.

Verify: `cargo check` passes.

## Step 8: Write Unit Tests

Tests for:
- EncodeConfig defaults and builder
- FfmpegError display messages
- build_ffmpeg_args output
- validate_input with missing dir, empty dir, valid dir
- detect_ffmpeg (conditional)

Verify: `cargo test` passes (all existing 80+ tests + new ones).

## Step 9: Final Verification

- `cargo check` -- clean compilation
- `cargo test` -- all tests pass
- `cargo clippy` -- no warnings
- Update ticket frontmatter to status: done, phase: done

## Testing Strategy

Unit tests (always run):
- Config construction and defaults
- Error display messages
- Argument building
- Input validation (using temp directories)

Conditional tests (run only if ffmpeg is installed):
- detect_ffmpeg() succeeds
- Full encode test with tiny PNG frames (skipped if no ffmpeg)

No integration tests requiring actual video playback.
