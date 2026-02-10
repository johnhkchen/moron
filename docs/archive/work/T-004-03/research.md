# T-004-03 Research: Chromium Bridge

## Objective

Implement the headless Chromium bridge in `moron-core/src/chromium.rs`. The bridge connects
Rust to the React rendering layer via Chrome DevTools Protocol (CDP), enabling the core
render loop: inject FrameState JSON into a page, trigger React re-render, capture a PNG
screenshot of the result.

## Current State

### Stub File

`moron-core/src/chromium.rs` contains a single line:

```rust
//! Chromium bridge: headless browser orchestration via chromiumoxide.
```

It is declared as `pub mod chromium` in `moron-core/src/lib.rs` and compiles as an empty module.

### Dependency Status

`chromiumoxide = "0.7"` is listed in both the workspace root `Cargo.toml` and
`moron-core/Cargo.toml`, but commented out in both places with the annotation
"Heavy dependencies -- uncomment when integration code is ready."

No other crate in the workspace references chromiumoxide.

### Related Stubs

`moron-core/src/renderer.rs` and `moron-core/src/ffmpeg.rs` are also single-line stubs.
The renderer (T-004-04) will be the primary consumer of the Chromium bridge.

## Upstream Dependencies (What This Ticket Receives)

### T-004-01: FrameState Serialization

Defines `FrameState` struct with `Serialize` derive. Contains:
- `elements: Vec<ElementState>` (id, kind, content, visual properties)
- `theme: ThemeState` (CSS custom property key-value pairs)
- `active_narration: Option<String>`
- Timeline metadata (time, frame, total_duration, fps)

The bridge receives `FrameState` already serialized to a JSON string via `serde_json::to_string()`.
The bridge does not need to understand the FrameState internals -- it passes opaque JSON to
the page's JavaScript.

### T-004-02: React Frame Component (`<MoronFrame>`)

A React component in `packages/ui/` that accepts FrameState as props and renders a static
frame. The component is loaded into an HTML host page that the bridge navigates to. The
bridge calls a JavaScript function (e.g., `window.__moron_setFrame(json)`) to update props,
React re-renders, and then the bridge captures the screenshot.

The HTML host page is a built artifact -- either a static `index.html` with bundled JS,
or a page served by a local dev server. The bridge needs to know its path/URL.

## chromiumoxide Crate API (v0.7)

### Core Types

| Type | Role |
|------|------|
| `Browser` | Manages the Chromium process or connection. Creates pages. |
| `BrowserConfig` / `BrowserConfigBuilder` | Configuration for launching headless Chrome. |
| `Handler` | Async stream that drives CDP WebSocket communication. Must be polled continuously. |
| `Page` | A browser tab. Navigation, JS evaluation, screenshot capture. |
| `ScreenshotParams` | Configuration for `Page::screenshot()`. Format, full-page, background. |

### Browser Launch Pattern

```
let (browser, mut handler) = Browser::launch(
    BrowserConfig::builder()
        .window_size(1920, 1080)
        .build()?
).await?;

// Handler MUST be polled in a background task
let handle = tokio::spawn(async move {
    while let Some(h) = handler.next().await {
        if h.is_err() { break; }
    }
});
```

Key: `Browser::launch` returns a tuple of `(Browser, Handler)`. The Handler is a `Stream`
(from `futures::StreamExt`) that must be continuously polled via `handler.next().await` in a
spawned tokio task. If the handler is not polled, all CDP communication stalls.

### BrowserConfigBuilder Options

| Method | Effect |
|--------|--------|
| `.window_size(w, h)` | Sets `--window-size=WxH` Chrome arg |
| `.viewport(Some(Viewport { width, height, .. }))` | Device viewport emulation |
| `.no_sandbox()` | Adds `--no-sandbox` flag |
| `.with_head()` | Runs headful (for debugging) |
| `.new_headless_mode()` | Uses Chrome's new headless mode |
| `.chrome_executable(path)` | Custom Chrome/Chromium binary |
| `.launch_timeout(Duration)` | Max wait for Chrome to start (default 20s) |
| `.request_timeout(Duration)` | CDP request timeout |
| `.arg(String)` | Add arbitrary Chrome CLI argument |
| `.disable_default_args()` | Skip built-in args |
| `.incognito()` | Launch in incognito context |

Default viewport: 800x600. Default headless: true (old mode). Default launch timeout: 20s.

### Page Navigation and Content

| Method | Signature | Notes |
|--------|-----------|-------|
| `goto(url)` | `async fn goto(impl Into<NavigateParams>) -> Result<&Self>` | Navigate to URL, resolves after load |
| `set_content(html)` | `async fn set_content(impl AsRef<str>) -> Result<&Self>` | Set page HTML directly |
| `content()` | `async fn content() -> Result<String>` | Get page HTML |
| `reload()` | `async fn reload() -> Result<&Self>` | Reload current page |

For our use case, `goto("file:///path/to/index.html")` navigates to the built React app.

### JavaScript Evaluation

| Method | Input | Notes |
|--------|-------|-------|
| `evaluate(impl Into<Evaluation>)` | Expression or function string | Auto-detects type |
| `evaluate_expression(impl Into<EvaluateParams>)` | Strict expression | No function detection |
| `evaluate_function(impl Into<CallFunctionOnParams>)` | Function declaration + args | Handles promises |

For frame injection, we need to call a JS function that updates React state:
```
page.evaluate("window.__moron_setFrame({json_string})").await?
```

Or using `evaluate_expression` for simpler expressions without function wrapping.

### Screenshot Capture

```
page.screenshot(
    ScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png)
        .full_page(false)
        .omit_background(false)
        .build()
) -> Result<Vec<u8>>
```

Returns PNG bytes as `Vec<u8>`. The `CaptureScreenshotFormat::Png` is from the CDP types
in `chromiumoxide_cdp`. The format defaults to PNG if not specified.

Screenshot captures the viewport at its configured size. For 1920x1080 output, the
viewport must be set to 1920x1080.

### Browser Shutdown

```
browser.close().await?;
handle.await?;  // wait for handler task to finish
```

`browser.close()` triggers graceful shutdown. The handler task exits when the browser
closes. Must await the JoinHandle to clean up.

### Alternative: Browser::connect

`Browser::connect(ws_url)` connects to an already-running Chrome instance via its
WebSocket debugger URL. Useful for development/debugging but not the primary path.

## Async Architecture Considerations

### tokio Runtime

moron-core already depends on `tokio = { version = "1", features = ["full"] }`. The
chromiumoxide crate is built on tokio. No runtime conflict.

### Handler Task Lifetime

The handler task must live for the entire duration of Chrome usage. It is spawned when
the browser launches and joined when the browser closes. The pattern is:

1. `Browser::launch` -> `(Browser, Handler)`
2. `tokio::spawn` the handler polling loop
3. Use `browser` and `page` for all CDP operations
4. `browser.close()` -> handler loop exits
5. `handle.await` -> join the task

This means the bridge struct must own both the `Browser` and the `JoinHandle<()>`.

### Concurrency Model

For moron's use case, operations are sequential: inject frame -> wait for render -> capture
screenshot -> next frame. No concurrent page operations. Single page, single tab.

However, the handler task runs concurrently with the main rendering loop. This is
managed automatically by tokio's cooperative scheduler.

### Error Propagation

chromiumoxide methods return `Result<T, CdpError>` (the crate's error type). The bridge
should convert these to `anyhow::Error` or a custom error type. The crate already uses
`anyhow` and `thiserror` in the workspace.

## Chrome Process Lifecycle

### Finding Chrome

chromiumoxide includes a detection module that automatically finds Chrome/Chromium on the
system. On macOS, it looks for:
- `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
- `/Applications/Chromium.app/Contents/MacOS/Chromium`
- Homebrew-installed chromium

The `BrowserConfigBuilder::chrome_executable()` overrides automatic detection.

### Process Management

`Browser::launch` spawns a Chrome child process. The process is killed when `Browser` is
dropped or `browser.close()` is called. If the Rust process panics without closing, the
Chrome process may be orphaned. A `Drop` impl on the bridge struct should call close.

### Memory Footprint

Per the specification (section 2.2): Chromium ~2-3 GB. The bridge should launch a single
browser instance and reuse it for all frames. Creating a new browser per frame would be
prohibitively expensive.

## Page Update Strategy

### Option A: Evaluate JS expression per frame

For each frame:
1. Serialize FrameState to JSON string
2. Call `page.evaluate(format!("window.__moron_setFrame({})", json))`
3. Wait for React to re-render (requestAnimationFrame or explicit signal)
4. Capture screenshot

Pros: Simple. No file I/O per frame.
Cons: Large JSON strings passed through CDP. Need to handle JSON escaping carefully.

### Option B: Set frame via CDP Runtime.evaluate with argument binding

Use `evaluate_function` with `CallFunctionOnParams` to pass the JSON as an argument
rather than string-interpolating it into JS source code. Avoids escaping issues.

### Option C: Write frame JSON to a file, page reads via fetch

Avoids large CDP messages but adds file I/O and requires the page to poll or be notified.
More complex. Not recommended for initial implementation.

### Render Synchronization

After injecting new FrameState, the bridge must wait for React to finish rendering before
capturing the screenshot. Options:

1. **requestAnimationFrame**: `page.evaluate("new Promise(r => requestAnimationFrame(r))")`.
   Waits one frame tick. May not be enough if React batches updates.
2. **Explicit ready signal**: The MoronFrame component sets `window.__moron_ready = true`
   after render. Bridge polls or awaits a CDP event.
3. **Double rAF**: `requestAnimationFrame(() => requestAnimationFrame(resolve))`. Standard
   pattern for ensuring paint completion.
4. **MutationObserver**: Watch for DOM changes after state update. Complex.

Double rAF is the most reliable simple approach. The explicit ready signal is cleaner for
production but requires React-side cooperation.

## Downstream Consumer

### T-004-04: Frame Rendering Loop

The renderer will call the bridge in a loop:

```
for frame_num in 0..total_frames {
    let state = compute_frame_state(&m, time);
    let json = serde_json::to_string(&state)?;
    let png_bytes = bridge.capture_frame(&json).await?;
    write_png_to_disk(frame_num, &png_bytes)?;
}
```

The bridge's public API for the renderer is essentially:
- `new(config) -> Self` (launch Chrome, load the React app page)
- `capture_frame(json: &str) -> Result<Vec<u8>>` (inject + screenshot)
- `close()` or `Drop` (shutdown Chrome)

## What the Bridge Does NOT Do

- Parse or understand FrameState (opaque JSON passthrough)
- Manage the React app build process (assumes built HTML exists)
- Handle FFmpeg encoding (separate pipeline, T-005)
- Manage multiple pages or tabs (single page for all frames)
- Serve the React app (static file or external dev server)
