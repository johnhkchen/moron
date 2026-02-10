# T-004-04 Plan: Frame Rendering Loop

## Step 1: Define RenderError

Write the `RenderError` enum in `renderer.rs` with thiserror derives.

Variants:
- `Bridge(BridgeError)` with `#[from]`
- `OutputDir { path: PathBuf, source: std::io::Error }`
- `Serialize { frame: u32, source: serde_json::Error }`
- `WriteFrame { frame: u32, path: PathBuf, source: std::io::Error }`

Verification: `cargo check` passes.

## Step 2: Define RenderProgress and RenderResult

Add the two data structs:

```rust
pub struct RenderProgress {
    pub current_frame: u32,
    pub total_frames: u32,
}

pub struct RenderResult {
    pub total_frames: u32,
    pub output_dir: PathBuf,
}
```

Both are simple data containers. No methods needed.

Verification: `cargo check` passes.

## Step 3: Define RenderConfig

Add the configuration struct:

```rust
pub struct RenderConfig {
    pub output_dir: PathBuf,
    pub bridge_config: BridgeConfig,
    pub progress: Option<Box<dyn Fn(RenderProgress)>>,
}

impl RenderConfig {
    pub fn new(output_dir: impl Into<PathBuf>, bridge_config: BridgeConfig) -> Self {
        Self {
            output_dir: output_dir.into(),
            bridge_config,
            progress: None,
        }
    }
}
```

Verification: `cargo check` passes.

## Step 4: Implement frame_path helper

```rust
fn frame_path(output_dir: &Path, frame_num: u32) -> PathBuf {
    output_dir.join(format!("frame_{:06}.png", frame_num))
}
```

Add a unit test:

```rust
#[test]
fn frame_path_formatting() {
    let p = frame_path(Path::new("/tmp/out"), 0);
    assert_eq!(p, PathBuf::from("/tmp/out/frame_000000.png"));
    let p = frame_path(Path::new("/tmp/out"), 42);
    assert_eq!(p, PathBuf::from("/tmp/out/frame_000042.png"));
    let p = frame_path(Path::new("/tmp/out"), 999999);
    assert_eq!(p, PathBuf::from("/tmp/out/frame_999999.png"));
}
```

Verification: `cargo test` for this test passes.

## Step 5: Implement the render function

Write the main `render` async function:

```rust
pub async fn render(m: &M, config: RenderConfig) -> Result<RenderResult, RenderError>
```

Logic:
1. Get `total_frames` and `fps` from `m.timeline()`
2. If `total_frames == 0`, return early with `RenderResult { total_frames: 0, output_dir }`
3. Create output directory with `std::fs::create_dir_all`
4. Launch `ChromiumBridge` from `config.bridge_config`
5. Loop `for frame_num in 0..total_frames`:
   a. Compute `time = frame_num as f64 / fps as f64`
   b. Call `compute_frame_state(m, time)`
   c. Serialize to JSON
   d. Call `bridge.capture_frame(&json).await`
   e. Write PNG to `frame_path(output_dir, frame_num)`
   f. Report progress
6. Close bridge
7. Return `RenderResult`

Error handling: if an error occurs in the loop, close the bridge before returning.

Verification: `cargo check` passes.

## Step 6: Update lib.rs re-exports

Add to `moron-core/src/lib.rs`:

```rust
pub use renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
```

Add to the prelude module:

```rust
pub use crate::renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
```

Verification: `cargo check` passes.

## Step 7: Add unit tests

Add tests that do not require Chrome:

1. `render_config_new` -- verify constructor sets fields correctly
2. `frame_path_formatting` -- already added in step 4
3. `render_error_display` -- verify error messages are human-readable
4. `render_progress_fields` -- verify struct fields are accessible
5. `empty_timeline_frame_count` -- verify `total_frames() == 0` for empty M

Verification: `cargo test` passes for all tests.

## Step 8: Update module doc comment

Replace the stale "Bevy + wgpu + vello" doc comment with accurate documentation
reflecting the Chromium bridge rendering pipeline.

Verification: `cargo doc` would pass (verified via cargo check).

## Step 9: Final verification

Run `cargo check` and `cargo test` across the entire workspace to verify:
- No compilation errors
- All existing tests still pass
- New tests pass

## Step 10: Update ticket frontmatter

Set `status: done` and `phase: done` in `docs/active/tickets/T-004-04.md`.

## Testing Strategy

### Unit tests (in renderer.rs)
- Config construction
- Path formatting
- Error message formatting
- Progress struct access
- Empty timeline check (via Timeline/M, no bridge)

### Not tested in this ticket
- Full integration (requires Chrome installed, React app built)
- FFmpeg consumption of output (T-005 scope)
- Performance characteristics

### Verification criteria
- `cargo check` passes (all code compiles)
- `cargo test` passes (all tests pass)
- `cargo clippy` passes (no lint warnings)
