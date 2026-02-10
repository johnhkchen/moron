# T-004-04 Design: Frame Rendering Loop

## Problem

Given a built scene (`M` with a recorded timeline), implement an async function that:
1. Computes total frame count from timeline duration and FPS
2. Launches the Chromium bridge (or accepts an already-launched one)
3. For each frame: computes timestamp, builds FrameState, serializes to JSON, captures PNG
4. Saves numbered PNG files (`frame_000000.png`, etc.) to an output directory
5. Reports progress (frame N of total)
6. Handles empty timelines gracefully (0 frames = no output)

---

## Design Decisions

### 1. Public API Shape

**Options considered:**

A) **Single async function** -- `render_scene(m, config).await`
B) **Renderer struct** -- `Renderer::new(config).render(m).await`
C) **Builder pattern** -- `RenderBuilder::new().scene(m).config(c).render().await`

**Decision: (A) Single async function with a RenderConfig struct.**

Rationale: The render operation is a one-shot batch process. There is no state to carry
between calls. A struct adds no value here -- the only state is configuration, which is
better modeled as a config struct passed to a function. The function creates ephemeral
resources (ChromiumBridge), uses them, and tears them down.

```rust
pub async fn render(m: &M, config: RenderConfig) -> Result<RenderResult, RenderError>;
```

This is the simplest API that satisfies the requirements. If state accumulation is needed
later (e.g., for incremental re-rendering), a struct can wrap this function.

### 2. Configuration

**Decision: RenderConfig struct with BridgeConfig embedded.**

```rust
pub struct RenderConfig {
    pub output_dir: PathBuf,
    pub bridge_config: BridgeConfig,
}
```

The renderer needs two things: where to write frames and how to launch Chrome. Rather than
duplicating BridgeConfig fields, embed it directly. The caller constructs BridgeConfig
(which already has sensible defaults via `BridgeConfig::new(html_path)`) and wraps it
in RenderConfig.

Constructor: `RenderConfig::new(output_dir, bridge_config)`. No defaults for either
field since both are mandatory (you must specify where output goes and where the React
app lives).

### 3. Bridge Lifecycle

**Options considered:**

A) **Renderer launches and closes the bridge** -- bridge is an implementation detail
B) **Caller passes a pre-launched bridge** -- renderer borrows it
C) **Both** -- accept either config or pre-launched bridge

**Decision: (A) Renderer manages the bridge lifecycle.**

Rationale: The render function is a self-contained batch operation. It launches Chrome
at the start, uses it for all frames, and closes it at the end. This keeps the API simple
and prevents resource leaks. If the caller needs to share a bridge across multiple renders
(e.g., for preview mode), that can be added as a separate API later.

The function handles cleanup even on error: if an error occurs mid-render, the bridge
is still closed (via a guard pattern or explicit error handling).

### 4. Frame Iteration

**Decision: Simple for loop with computed timestamps.**

```rust
let total_frames = m.timeline().total_frames();
let fps = m.timeline().fps();

for frame_num in 0..total_frames {
    let time = frame_num as f64 / fps as f64;
    let state = compute_frame_state(m, time);
    let json = serde_json::to_string(&state)?;
    let png = bridge.capture_frame(&json).await?;
    // write to disk
}
```

Time computation: `frame_num / fps` gives the timestamp for each frame. Frame 0 is at
t=0.0, frame 1 at t=1/30 (~0.033s), etc. This matches the standard video convention.

### 5. File Output

**Decision: Synchronous write with `std::fs::write`.**

Writing PNG files to disk does not benefit from async I/O in this context. Each frame
is written sequentially after capture. The PNG bytes are already in memory. `std::fs::write`
is simpler and faster than `tokio::fs::write` for small sequential writes. Using
`tokio::fs::write` would be appropriate if we needed concurrent writes, but frames are
produced one at a time.

The output directory is created with `std::fs::create_dir_all` at the start of rendering.

File naming: `format!("frame_{:06}.png", frame_num)` produces `frame_000000.png` through
`frame_999999.png`.

### 6. Progress Reporting

**Options considered:**

A) **Callback closure** -- `config.on_progress: Option<Box<dyn Fn(u32, u32)>>`
B) **Progress trait** -- `trait ProgressReporter { fn report(&self, current, total); }`
C) **Channel-based** -- `mpsc::Sender<ProgressEvent>`
D) **Simple eprintln** -- print to stderr directly
E) **Callback with eprintln fallback**

**Decision: (E) Optional callback with eprintln fallback.**

The progress callback is `Option<Box<dyn Fn(RenderProgress)>>` on the RenderConfig. If
provided, it is called for each frame. If not provided, the renderer prints to stderr.

```rust
pub struct RenderProgress {
    pub current_frame: u32,
    pub total_frames: u32,
}
```

This is the most flexible approach without over-engineering. The callback can be used by
a CLI progress bar, a GUI, or silenced entirely (provide a no-op closure). The eprintln
fallback ensures that batch usage always shows progress.

### 7. Error Handling

**Decision: Custom RenderError enum using thiserror.**

```rust
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("bridge error: {0}")]
    Bridge(#[from] BridgeError),

    #[error("failed to create output directory {path}: {source}")]
    OutputDir { path: PathBuf, source: std::io::Error },

    #[error("failed to serialize frame {frame}: {source}")]
    Serialize { frame: u32, source: serde_json::Error },

    #[error("failed to write frame {frame} to {path}: {source}")]
    WriteFrame { frame: u32, path: PathBuf, source: std::io::Error },
}
```

Each variant captures the context needed for debugging: which frame failed, what path
was involved, what the underlying error was. `BridgeError` is wrapped via `#[from]` for
ergonomic `?` usage.

### 8. Return Value

**Decision: RenderResult struct with summary statistics.**

```rust
pub struct RenderResult {
    pub total_frames: u32,
    pub output_dir: PathBuf,
}
```

Returns the number of frames rendered and the output directory path. This is useful for
the caller (T-005 FFmpeg encoding) which needs both to construct the FFmpeg command.

### 9. Empty Timeline Handling

**Decision: Return immediately with RenderResult { total_frames: 0 }.**

If `m.timeline().total_frames() == 0`, skip bridge launch entirely. No output directory
created, no Chrome process spawned. Return `Ok(RenderResult { total_frames: 0, output_dir })`.

This is the correct behavior: an empty timeline produces zero frames. Launching Chrome
for zero frames would waste resources and risk unnecessary errors (e.g., Chrome not
installed).

### 10. Output Directory Creation

**Decision: Create if not exists, error if creation fails.**

The render function calls `std::fs::create_dir_all(output_dir)` before rendering. If the
directory already exists, this is a no-op. If it cannot be created (permissions, invalid
path), return `RenderError::OutputDir`.

Files are not cleaned up on error. If rendering fails at frame 50 of 100, frames 0-49
remain on disk. This is the correct behavior: partial output is useful for debugging and
avoids data loss.

---

## What Was Rejected

- **Renderer struct with state**: No state persists between renders. A struct would just
  be a wrapper around config with a single method. A function is simpler and more honest.

- **Async file I/O**: `tokio::fs::write` adds overhead (task spawning, scheduling) for
  sequential operations. PNG writes are fast (~50-200ms for 1080p). Not a bottleneck.

- **Parallel frame capture**: Chromium can only render one frame at a time in a single
  page. The sequential loop is the correct approach. Parallelism would require multiple
  pages or browsers, which is a future optimization.

- **Streaming to FFmpeg**: Piping PNG bytes directly to FFmpeg's stdin instead of writing
  to disk. This is the S-005 pipeline optimization. T-004-04 produces the image sequence
  on disk as the baseline approach.

- **Progress trait**: Over-abstraction for a single use case. A closure is simpler and
  equally flexible.

---

## Complete Render Flow

1. Check `total_frames`. If 0, return early.
2. Create output directory via `create_dir_all`.
3. Launch ChromiumBridge with the BridgeConfig from RenderConfig.
4. For each frame 0..total_frames:
   a. Compute `time = frame_num as f64 / fps as f64`
   b. Call `compute_frame_state(m, time)` to get FrameState
   c. Serialize FrameState to JSON via `serde_json::to_string`
   d. Call `bridge.capture_frame(&json).await` to get PNG bytes
   e. Write PNG bytes to `output_dir/frame_NNNNNN.png`
   f. Report progress via callback or eprintln
5. Close the bridge via `bridge.close().await`.
6. Return `RenderResult { total_frames, output_dir }`.

Error at any step short-circuits and returns the error. The bridge is closed in a cleanup
path (explicitly before returning the error).
