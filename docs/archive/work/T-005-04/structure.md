# T-005-04 Structure: End-to-End Validation Test

## File Changes

### New: `moron-core/tests/e2e.rs`

Single new file containing the end-to-end test suite.

### Modified: `docs/active/tickets/T-005-04.md`

Update frontmatter: status -> done, phase -> done.

## Module Layout of `e2e.rs`

```
// Header comment: how to run (cargo test --test e2e -- --ignored)
//
// Imports: moron_core::prelude::*, std::fs, std::path, std::process

// --- Helper functions ---

fn create_minimal_png(width: u32, height: u32) -> Vec<u8>
    // Produces a valid, minimal PNG with a single solid color.
    // Uses raw PNG construction (signature + IHDR + IDAT + IEND).
    // No external image crate dependency needed.

fn write_synthetic_frames(dir: &Path, count: u32, width: u32, height: u32)
    // Writes `count` numbered PNGs (frame_000000.png, ...) to `dir`.

fn ffprobe_duration(path: &Path) -> Option<f64>
    // Runs ffprobe to extract duration. Returns None if ffprobe unavailable.

fn ffprobe_has_video_stream(path: &Path) -> Option<bool>
    // Runs ffprobe to check for video stream. Returns None if unavailable.

fn ffprobe_has_audio_stream(path: &Path) -> Option<bool>
    // Runs ffprobe to check for audio stream. Returns None if unavailable.

// --- Ignored tests (need FFmpeg) ---

#[test]
#[ignore]
fn e2e_full_pipeline()
    // The main end-to-end test:
    // 1. Build DemoScene
    // 2. Verify timeline properties
    // 3. Compute and serialize FrameStates
    // 4. Write synthetic PNGs
    // 5. Encode to video-only mp4
    // 6. Assemble audio and mux
    // 7. Validate output file
    // 8. Optional ffprobe checks

#[test]
#[ignore]
fn e2e_encode_and_mux_roundtrip()
    // Focused test: write frames, encode, mux, validate.
    // Simpler than full pipeline -- no scene building.

// --- Non-ignored tests (no system deps) ---

#[test]
fn e2e_demo_scene_frame_states_serialize()
    // Build DemoScene, compute FrameState for every frame,
    // verify all serialize to valid JSON.

#[test]
fn e2e_empty_scene_produces_error()
    // Verify build_video rejects a scene with 0 frames.
    // Uses tokio::runtime::Runtime::new() for sync test.
```

## Dependencies

No new crate dependencies. The test file uses:
- `moron_core` (already a dependency via workspace)
- `std::fs`, `std::path`, `std::process` (stdlib)
- `tokio::runtime::Runtime` for running async in sync test context

## PNG Generation Strategy

Use `flate2` or manual DEFLATE? No -- we use an uncompressed PNG approach.
Actually, PNG requires DEFLATE-compressed IDAT. The simplest valid approach:
use zlib stored blocks (compression level 0) which is trivial to construct.

Alternative: write a small constant PNG (e.g., 8x8 blue) as a hardcoded
byte array. This is simpler and less error-prone.

Decision: Use a hardcoded minimal valid PNG byte array for a small image.
The exact pixel content doesn't matter; FFmpeg just needs valid PNG files.
We'll encode a 2x2 solid-blue PNG as a constant.
