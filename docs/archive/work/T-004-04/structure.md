# T-004-04 Structure: Frame Rendering Loop

## Files Modified

### 1. `moron-core/src/renderer.rs` (replace stub)

This is the primary deliverable. The stub is replaced with the full renderer module.

#### Types Defined

```
RenderConfig          -- configuration for the render function
  output_dir: PathBuf
  bridge_config: BridgeConfig
  progress: Option<Box<dyn Fn(RenderProgress)>>

RenderProgress        -- progress report passed to callback
  current_frame: u32
  total_frames: u32

RenderResult          -- summary returned on success
  total_frames: u32
  output_dir: PathBuf

RenderError           -- error enum (thiserror)
  Bridge(BridgeError)
  OutputDir { path, source }
  Serialize { frame, source }
  WriteFrame { frame, path, source }
```

#### Functions Defined

```
pub async fn render(m: &M, config: RenderConfig) -> Result<RenderResult, RenderError>
```

The single public function. All rendering logic lives here.

#### Internal Helpers

```
fn frame_path(output_dir: &Path, frame_num: u32) -> PathBuf
```

Formats the output path for a given frame number: `output_dir/frame_000000.png`.

#### Module Documentation

The module doc comment is updated from the stale "Bevy + wgpu + vello" reference to
describe the actual Chromium bridge rendering pipeline.

### 2. `moron-core/src/lib.rs` (modify re-exports)

Add re-exports for the new renderer types:

```rust
pub use renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
```

Add to prelude:

```rust
pub use crate::renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
```

## Module Boundaries

### renderer.rs imports

From `crate`:
- `crate::chromium::{BridgeConfig, ChromiumBridge, BridgeError}` -- bridge types
- `crate::frame::compute_frame_state` -- frame state computation
- `crate::facade::M` -- scene facade

From std:
- `std::path::{Path, PathBuf}` -- path types
- `std::fs` -- directory creation, file writing

From external:
- `thiserror::Error` -- error derive macro
- `serde_json` -- FrameState serialization

### renderer.rs exports

All public types and the `render` function are exported. No internal/private types
beyond the `frame_path` helper.

### Dependency Direction

```
renderer.rs  -->  chromium.rs   (launches bridge, calls capture_frame)
renderer.rs  -->  frame.rs      (calls compute_frame_state)
renderer.rs  -->  facade.rs     (reads M for timeline access)
renderer.rs  -->  timeline.rs   (reads fps, total_frames via M.timeline())
```

No circular dependencies. renderer.rs is a leaf consumer of the other modules.

## Type Relationships

```
RenderConfig ----contains----> BridgeConfig (from chromium.rs)
RenderError  ----wraps------> BridgeError  (from chromium.rs, via #[from])
RenderError  ----wraps------> std::io::Error (for filesystem operations)
RenderError  ----wraps------> serde_json::Error (for serialization)
render()     ----accepts----> &M (from facade.rs)
render()     ----calls------> compute_frame_state (from frame.rs)
render()     ----creates----> ChromiumBridge (from chromium.rs)
```

## Public Interface

The renderer module exposes exactly:
- 1 function: `render`
- 4 types: `RenderConfig`, `RenderProgress`, `RenderResult`, `RenderError`

This is the minimum viable surface area. All complexity is internal to the `render`
function.

## File Organization Within renderer.rs

```
//! Module doc comment

// Imports

// --- RenderError ---
// #[derive(Debug, thiserror::Error)]
// pub enum RenderError { ... }

// --- RenderProgress ---
// pub struct RenderProgress { ... }

// --- RenderConfig ---
// pub struct RenderConfig { ... }
// impl RenderConfig { pub fn new(...) -> Self }

// --- RenderResult ---
// pub struct RenderResult { ... }

// --- render function ---
// pub async fn render(...) -> Result<RenderResult, RenderError>

// --- Internal helpers ---
// fn frame_path(...) -> PathBuf

// --- Tests ---
// #[cfg(test)] mod tests { ... }
```

Follows the same section-comment style used in `chromium.rs` and `frame.rs` with
`// ---------------------------------------------------------------------------` dividers.

## Testing Structure

Tests in `renderer.rs` cover non-async, non-Chrome logic:

1. `RenderConfig::new` construction
2. `frame_path` output formatting
3. Empty timeline produces 0 total_frames (via timeline, no render call)
4. RenderError Display trait messages
5. RenderProgress field access

No integration tests that require Chrome. Those belong in a separate test suite
or are verified via `cargo check` (the code compiles and type-checks against the
real ChromiumBridge).

## Changes NOT Made

- `chromium.rs`: No changes. The bridge API is used as-is.
- `frame.rs`: No changes. `compute_frame_state` is used as-is.
- `facade.rs`: No changes. `M` public API is sufficient.
- `timeline.rs`: No changes. Timeline API is sufficient.
- `Cargo.toml`: No changes. All dependencies already available.
