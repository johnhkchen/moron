# T-004-03 Progress: Chromium Bridge

## Status: Complete

All implementation steps are done. `cargo check`, `cargo clippy`, and `cargo test`
pass across the full workspace.

---

## Completed Steps

### Step 1: Activate Dependencies
- Uncommented `chromiumoxide = "0.7"` in workspace root `Cargo.toml`
- Added `futures = "0.3"` to workspace root `Cargo.toml`
- Uncommented `chromiumoxide = { workspace = true }` in `moron-core/Cargo.toml`
- Added `futures.workspace = true` to `moron-core/Cargo.toml`
- 162 transitive crates resolved and locked

### Step 2: Define BridgeError Enum
- Seven variants: LaunchFailed, ChromeNotFound, PageLoadFailed, JsEvalFailed,
  ScreenshotFailed, RenderTimeout, AlreadyClosed
- All variants use `#[error("...")]` from thiserror
- Source-wrapping variants use `#[source] anyhow::Error`

### Step 3: Define BridgeConfig Struct
- Six fields: width, height, html_path, chrome_executable, headless, launch_timeout
- `BridgeConfig::new(html_path)` sets sensible defaults (1920x1080, headless, 20s timeout)

### Step 4: Define ChromiumBridge Struct
- Three fields: browser (Browser), page (Page), handler_handle (Option<JoinHandle<()>>)
- handler_handle is `Option` to allow `close()` to take ownership without
  conflicting with the `Drop` impl

### Step 5: Implement ChromiumBridge::launch()
- Builds BrowserConfig from BridgeConfig with viewport, window size, sandbox,
  timeout, and optional headful/custom-executable settings
- Launches Chrome via Browser::launch, spawns handler polling task
- Canonicalizes html_path and navigates to file:// URL
- Verifies page exposes `window.__moron_setFrame` function
- Error-maps all failure modes to appropriate BridgeError variants

### Step 6: Implement wait_for_render()
- Double requestAnimationFrame wrapped in a Promise
- 5-second tokio::time::timeout as safety net
- Returns BridgeError::RenderTimeout on timeout, BridgeError::JsEvalFailed on
  JS evaluation error

### Step 7: Implement capture_frame()
- Injects frame JSON via `window.__moron_setFrame({json})` string interpolation
- Calls wait_for_render() for paint synchronization
- Captures viewport-only PNG screenshot via ScreenshotParams
- Returns Vec<u8> of PNG bytes

### Step 8: Implement close() and Drop
- close(mut self) calls browser.close().await, then awaits handler task
- Drop aborts handler task as fallback (Browser's own Drop kills Chrome process)

### Step 9: Full Workspace Verification
- `cargo check` -- passes (full workspace)
- `cargo check -p moron-core` -- passes
- `cargo clippy -p moron-core` -- zero warnings
- `cargo test` -- all 68 tests pass, zero failures

---

## Deviations from Plan

### handler_handle wrapped in Option
The plan specified `handler_handle: JoinHandle<()>` directly on the struct. During
implementation, `cargo check` revealed that `close(mut self)` cannot move
`self.handler_handle` out of a type that implements `Drop` (Rust E0509). Fixed by
wrapping in `Option<JoinHandle<()>>` and using `.take()` in both `close()` and `Drop`.
This is a standard pattern for types that need both a consuming shutdown method and
a Drop fallback.

### No unit tests
As planned. The bridge is pure I/O -- every method requires a running Chrome instance.
Integration tests will be added when CI has Chrome available.

---

## Files Modified

| File | Action |
|------|--------|
| `Cargo.toml` | Uncommented chromiumoxide, added futures |
| `moron-core/Cargo.toml` | Uncommented chromiumoxide, added futures |
| `moron-core/src/chromium.rs` | Replaced stub with ~230 lines of implementation |

## Files NOT Modified (as planned)

- `moron-core/src/lib.rs` -- already had `pub mod chromium`
- `moron-core/src/frame.rs` -- unchanged, consumed by bridge
- All other crates -- no changes
