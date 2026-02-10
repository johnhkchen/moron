# T-004-03 Design: Chromium Bridge

## Problem

Connect Rust to headless Chromium so that the rendering loop can: (1) inject FrameState
JSON into a page running the React MoronFrame component, (2) capture a PNG screenshot of
the rendered frame, and (3) return the PNG bytes to the caller. The bridge must manage the
Chrome process lifecycle (launch, use, shutdown) and handle errors cleanly.

---

## Design Decisions

### 1. Public API Shape

**Options considered:**

A) **Struct with async methods** -- `ChromiumBridge::new().await`, `bridge.capture_frame(json).await`
B) **Free functions** -- `launch_chrome().await`, `capture_frame(page, json).await`
C) **Trait-based** -- `trait FrameCapture { async fn capture(...) }` with Chromium impl

**Decision: (A) Struct with async methods.**

Rationale: The bridge owns long-lived resources (Browser handle, handler JoinHandle, Page).
A struct naturally encapsulates this state. Free functions would require passing these around
manually. A trait is premature -- there is no alternative implementation yet (the spec
mentions only Chromium for rendering). If a mock is needed for testing, it can be a second
struct that implements the same interface without the trait overhead.

```rust
pub struct ChromiumBridge {
    browser: Browser,
    page: Page,
    handler_handle: JoinHandle<()>,
}
```

Public methods:
- `ChromiumBridge::launch(config: BridgeConfig) -> Result<Self>`
- `ChromiumBridge::capture_frame(&self, frame_json: &str) -> Result<Vec<u8>>`
- `ChromiumBridge::close(self) -> Result<()>`

### 2. Configuration

**Decision: Dedicated BridgeConfig struct.**

Rather than exposing raw chromiumoxide `BrowserConfig` in the public API, wrap it in a
moron-specific config that captures the parameters the rendering loop cares about:

```rust
pub struct BridgeConfig {
    pub width: u32,              // viewport width, default 1920
    pub height: u32,             // viewport height, default 1080
    pub html_path: PathBuf,      // path to built React app index.html
    pub chrome_executable: Option<PathBuf>,  // override Chrome detection
    pub headless: bool,          // default true, false for debugging
    pub launch_timeout: Duration, // default 20s
}
```

`BridgeConfig::default()` uses 1920x1080, headless true, 20s timeout. The `html_path`
has no default and must be provided.

This insulates downstream code from chromiumoxide API changes. If the underlying crate
is replaced, only the bridge internals change.

### 3. Browser Launch Strategy

**Decision: Always launch a new Chrome process (no connect-to-existing by default).**

`Browser::launch` spawns a fresh headless Chrome. This is the right default for
`moron build` which runs as a batch job. A `connect` option can be added later for
`moron preview` which may want to reuse a long-running Chrome instance.

The launch sequence:

1. Build `BrowserConfig` from `BridgeConfig` fields
2. Call `Browser::launch(config).await`
3. Spawn handler task: `tokio::spawn(async move { while handler.next().await.is_some() {} })`
4. Create a page: `browser.new_page(&format!("file://{}", html_path)).await`
5. Wait for page load (goto resolves after load)
6. Verify the page has the `__moron_setFrame` function via a JS check
7. Return `ChromiumBridge { browser, page, handler_handle }`

### 4. Frame Injection Method

**Options considered:**

A) **String interpolation into evaluate** -- `page.evaluate(format!("window.__moron_setFrame({})", json))`
B) **evaluate_function with argument binding** -- pass JSON as a CDP argument object
C) **set_content per frame** -- replace entire HTML each frame

**Decision: (A) String interpolation with evaluate_expression.**

Rationale: FrameState JSON is well-formed (produced by serde_json) so it is safe to
interpolate directly into a JS expression. This is the simplest approach and avoids the
complexity of CDP argument binding. The JSON is already valid JavaScript literal syntax.

We use `evaluate_expression` (not `evaluate`) to avoid the auto-detection overhead of
the `evaluate` convenience method.

```rust
let js = format!("window.__moron_setFrame({})", frame_json);
self.page.evaluate_expression(js).await?;
```

The React host page defines `window.__moron_setFrame` as a function that:
1. Parses the JSON (or receives it as an object since it is interpolated as a literal)
2. Calls `ReactDOM.render` or updates state to re-render `<MoronFrame>` with new props
3. Sets `window.__moron_ready = true` after render completes

**Why not (B):** `evaluate_function` with `CallFunctionOnParams` requires constructing
CDP argument objects, which adds complexity for no benefit when the JSON is already valid
JS syntax. The argument binding approach would be better if we needed to pass binary data
or non-JSON-safe values, neither of which applies here.

**Why not (C):** Replacing the entire page HTML for every frame means re-parsing HTML,
re-loading scripts, and re-initializing React. Thousands of times slower than updating
state in an already-loaded page.

### 5. Render Synchronization

**Decision: Double requestAnimationFrame with a timeout fallback.**

After injecting FrameState, wait for the React component to render and the browser to
paint before capturing the screenshot.

```rust
async fn wait_for_render(&self) -> Result<()> {
    let js = r#"new Promise(resolve => {
        requestAnimationFrame(() => {
            requestAnimationFrame(() => resolve(true))
        })
    })"#;
    self.page.evaluate_expression(js).await?;
    Ok(())
}
```

Double rAF is the standard pattern: the first rAF fires before the next paint, the second
fires after it, guaranteeing the DOM mutations from React have been painted to the screen.

**Alternative considered: explicit ready signal.** The MoronFrame component could set
`window.__moron_ready = true` after rendering and the bridge could poll for it. This is
more robust for complex async rendering but adds coupling between the bridge and the
React component. Not needed for v1 where frames are synchronous renders.

**Timeout:** If double rAF does not resolve within 5 seconds, return an error. This
catches cases where Chrome is hung or the page has a JS error that prevents rendering.

### 6. Screenshot Capture

**Decision: PNG format, viewport-only (not full-page), opaque background.**

```rust
let params = ScreenshotParams::builder()
    .format(CaptureScreenshotFormat::Png)
    .full_page(false)
    .omit_background(false)
    .build();
let png_bytes: Vec<u8> = self.page.screenshot(params).await?;
```

- PNG for lossless frame capture. FFmpeg encodes the final video; frames must be lossless.
- Viewport-only because the viewport IS the frame (1920x1080). No scrolling content.
- Opaque background because video frames need a solid background. Transparency is
  meaningless for MP4 output.

The returned `Vec<u8>` is the complete PNG file bytes. The caller (T-004-04 renderer)
writes these to disk or pipes them directly to FFmpeg.

### 7. Error Handling Strategy

**Decision: Custom error enum wrapping chromiumoxide errors, using thiserror.**

```rust
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("failed to launch Chrome: {0}")]
    LaunchFailed(#[source] anyhow::Error),

    #[error("Chrome executable not found")]
    ChromeNotFound,

    #[error("failed to load page: {path}")]
    PageLoadFailed { path: String, #[source] source: anyhow::Error },

    #[error("JavaScript evaluation failed: {0}")]
    JsEvalFailed(#[source] anyhow::Error),

    #[error("screenshot capture failed: {0}")]
    ScreenshotFailed(#[source] anyhow::Error),

    #[error("render timed out after {timeout_secs}s")]
    RenderTimeout { timeout_secs: u64 },

    #[error("bridge already closed")]
    AlreadyClosed,
}
```

The error variants map to the failure modes the renderer cares about:
- Chrome not installed / wrong path -> actionable error message
- Page not found / failed to load -> check React app build
- JS errors -> check MoronFrame component
- Screenshot failures -> Chrome process issue
- Timeout -> frame too complex or Chrome hung

Using `thiserror` because the errors need specific variants the renderer can match on.
The `#[source]` attribute preserves the original chromiumoxide error chain.

### 8. Lifecycle and Drop

**Decision: Implement Drop to kill Chrome if close() is not called.**

```rust
impl Drop for ChromiumBridge {
    fn drop(&mut self) {
        // Best-effort: abort the handler task.
        // The Browser's own Drop will kill the Chrome process.
        self.handler_handle.abort();
    }
}
```

Proper shutdown uses `close(self)` which awaits graceful termination:

```rust
pub async fn close(self) -> Result<()> {
    self.browser.close().await?;
    let _ = self.handler_handle.await;
    Ok(())
}
```

Drop cannot call async functions, so it uses `abort()` on the handler task as a fallback.
chromiumoxide's `Browser` Drop implementation kills the Chrome child process, so even
without graceful shutdown the process is cleaned up.

The `close()` method takes `self` by value (consuming the bridge) to enforce the
pattern: close is the last thing you do with the bridge.

### 9. Viewport Configuration

**Decision: Set viewport via BrowserConfig window_size AND CDP viewport emulation.**

Two separate concepts:
- `window_size(w, h)`: Chrome window dimensions (the OS-level window)
- `viewport(Viewport { width, height })`: Device emulation viewport

For headless rendering, both should match the target resolution. Set window_size to
1920x1080 and viewport to `Some(Viewport { width: 1920, height: 1080, device_scale_factor: 1.0, .. })`.

`device_scale_factor: 1.0` ensures pixel-perfect output. A factor of 2.0 would produce
3840x2160 screenshots from a 1920x1080 viewport (useful for 4K output later).

### 10. HTML Host Page Contract

The bridge expects the HTML page at `html_path` to:

1. Load and mount the `<MoronFrame>` React component
2. Define `window.__moron_setFrame(frameState)` -- a function that accepts a FrameState
   object (already parsed from the JS literal) and triggers a React re-render
3. Render synchronously (no lazy loading, no code splitting, no suspense boundaries)

This contract is minimal. The bridge does not care about the page's internal structure
beyond the existence of `__moron_setFrame`. The T-004-02 React component implementation
owns the host page.

The bridge verifies the contract on startup:

```rust
let check = self.page.evaluate_expression(
    "typeof window.__moron_setFrame === 'function'"
).await?;
```

If the check fails, return `BridgeError::PageLoadFailed` with a message explaining
that the React app must expose `__moron_setFrame`.

---

## What Was Rejected

- **Trait-based abstraction** (`trait FrameCapture`): No second implementation exists.
  Adding a trait now would be speculative abstraction. If a WebGPU-based renderer is
  added later, the trait can be extracted at that time.

- **Connection pooling / multiple pages**: moron renders one video at a time, sequentially.
  Multiple pages would add concurrency complexity with no benefit. A single page is reused
  for all frames.

- **CDP direct (without chromiumoxide)**: Raw WebSocket + JSON-RPC against the CDP
  protocol. Massive implementation effort for no benefit. chromiumoxide is mature,
  well-maintained, and abstracts the protocol cleanly.

- **headless_chrome crate**: Alternative Rust CDP crate. It is synchronous (blocking),
  not async. moron's pipeline is async (tokio). chromiumoxide is the natural fit.

- **Puppeteer/Playwright via subprocess**: Spawning a Node.js process that drives Chrome.
  Adds a Node.js runtime dependency for the Rust side, IPC complexity, and another
  failure mode. chromiumoxide communicates directly with Chrome from Rust.

- **File-based frame injection**: Writing FrameState JSON to a file and having the page
  poll for changes. Adds file I/O latency per frame and requires a polling mechanism.
  Direct JS evaluation is simpler and faster.

- **Persistent WebSocket for frame updates**: Maintaining a separate WebSocket between
  Rust and the page for frame data. Unnecessary since CDP already provides JS evaluation
  as a communication channel.

---

## Complete capture_frame Flow

1. Serialize FrameState JSON (caller's responsibility, passed as `&str`)
2. `page.evaluate_expression(format!("window.__moron_setFrame({})", json))` -- inject frame
3. `wait_for_render()` -- double rAF to ensure paint completion
4. `page.screenshot(png_params)` -- capture viewport as PNG bytes
5. Return `Vec<u8>` to caller

Steps 2-4 are the hot path, called once per frame (thousands of times per video).
Step 2 is the dominant cost: CDP round-trip + React render + browser paint.

Expected per-frame time: 20-50ms (dominated by React render + screenshot capture),
yielding 20-50 fps throughput for the rendering pipeline, comfortably above real-time
for 30fps output on M5 hardware.
