# T-004-03 Structure: Chromium Bridge

## Overview

Three files change. Two Cargo.toml files get a line uncommented. One Rust source
file is replaced entirely. No new files are created. No modules are added or removed.

---

## File Changes

### 1. Cargo.toml (workspace root)

**Action:** Uncomment one line.

```
Before: # chromiumoxide = "0.7"
After:  chromiumoxide = "0.7"
```

Also add `futures` as a workspace dependency (needed for `StreamExt` on the Handler).

```toml
futures = "0.3"
```

### 2. moron-core/Cargo.toml

**Action:** Uncomment one line, add one line.

```
Before: # chromiumoxide = { workspace = true }
After:  chromiumoxide = { workspace = true }
```

Add:
```toml
futures = { workspace = true }
```

### 3. moron-core/src/chromium.rs

**Action:** Replace the single-line stub with the full module (~180 lines).

---

## Module Structure: chromium.rs

### Imports

```
chromiumoxide::{Browser, BrowserConfig, Page}
chromiumoxide::handler::viewport::Viewport
chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat
chromiumoxide::page::ScreenshotParams
futures::StreamExt                     (for Handler::next())
std::path::PathBuf
std::time::Duration
tokio::task::JoinHandle
thiserror::Error
```

### Public Types (in declaration order)

#### BridgeError (enum)

Custom error type with `#[derive(Debug, thiserror::Error)]`.

| Variant | Source | When |
|---------|--------|------|
| `LaunchFailed(#[source] anyhow::Error)` | Browser::launch | Chrome process won't start |
| `ChromeNotFound` | -- | No Chrome binary detected |
| `PageLoadFailed { path: String, source: anyhow::Error }` | Page::goto | HTML file missing or bad |
| `JsEvalFailed(#[source] anyhow::Error)` | evaluate_expression | JS throws or bad expression |
| `ScreenshotFailed(#[source] anyhow::Error)` | Page::screenshot | CDP screenshot command fails |
| `RenderTimeout { timeout_secs: u64 }` | tokio::time::timeout | Double rAF didn't resolve |
| `AlreadyClosed` | -- | Methods called after close() |

#### BridgeConfig (struct)

Configuration consumed by `ChromiumBridge::launch()`.

| Field | Type | Default |
|-------|------|---------|
| `width` | `u32` | `1920` |
| `height` | `u32` | `1080` |
| `html_path` | `PathBuf` | (required) |
| `chrome_executable` | `Option<PathBuf>` | `None` |
| `headless` | `bool` | `true` |
| `launch_timeout` | `Duration` | `20s` |

Implements `Default`-like constructor: `BridgeConfig::new(html_path: impl Into<PathBuf>)`.

#### ChromiumBridge (struct)

Owns the Chrome process lifetime.

| Field | Type | Visibility |
|-------|------|------------|
| `browser` | `Browser` | private |
| `page` | `Page` | private |
| `handler_handle` | `JoinHandle<()>` | private |

### Public Methods on ChromiumBridge

#### `launch(config: BridgeConfig) -> Result<Self, BridgeError>`

Async constructor. Sequence:
1. Build `BrowserConfig` from `BridgeConfig` fields
2. `Browser::launch(browser_config).await`
3. `tokio::spawn` handler polling loop
4. `browser.new_page(file_url).await`
5. Verify `__moron_setFrame` exists on page
6. Return `ChromiumBridge`

#### `capture_frame(&self, frame_json: &str) -> Result<Vec<u8>, BridgeError>`

Hot-path method called once per frame:
1. `evaluate_expression(format!("window.__moron_setFrame({})", frame_json))`
2. `wait_for_render()` (double rAF with 5s timeout)
3. `page.screenshot(png_params)`
4. Return PNG bytes

#### `close(mut self) -> Result<(), BridgeError>`

Graceful shutdown:
1. `browser.close().await`
2. `handler_handle.await`

### Private Methods on ChromiumBridge

#### `wait_for_render(&self) -> Result<(), BridgeError>`

Double requestAnimationFrame wrapped in a Promise, with 5-second tokio timeout.

### Drop Implementation

`impl Drop for ChromiumBridge` calls `self.handler_handle.abort()` as fallback.
The Browser's own Drop kills the Chrome process.

---

## Module Boundary

`chromium.rs` exports:
- `ChromiumBridge` (struct)
- `BridgeConfig` (struct)
- `BridgeError` (enum)

All three are `pub`. No other types are exported from this module.

`lib.rs` already has `pub mod chromium` -- no change needed there.

The prelude in `lib.rs` does NOT re-export chromium types. The bridge is an
internal engine component, not a scene-authoring type. Consumers import explicitly:

```rust
use moron_core::chromium::{ChromiumBridge, BridgeConfig, BridgeError};
```

---

## Dependency Graph Impact

```
moron-core
  +-- chromiumoxide 0.7  (NEW active dependency)
  +-- futures 0.3        (NEW dependency)
  +-- tokio 1            (existing)
  +-- thiserror 2        (existing)
  +-- anyhow 1           (existing)
```

No other workspace crates gain dependencies. The chromiumoxide crate pulls in
tokio, futures, serde, and websocket crates transitively, but these are already
in the dependency graph.

---

## What Does NOT Change

- `moron-core/src/lib.rs` -- already declares `pub mod chromium`
- `moron-core/src/frame.rs` -- FrameState is consumed by the bridge, not modified
- `moron-core/src/renderer.rs` -- will consume the bridge in T-004-04, not this ticket
- `packages/ui/` -- the React side is T-004-02's domain
- No test files are created (integration tests require a real Chrome install)
