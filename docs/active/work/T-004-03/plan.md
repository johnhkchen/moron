# T-004-03 Plan: Chromium Bridge

## Implementation Steps

### Step 1: Activate Dependencies

**Files:** `Cargo.toml` (root), `moron-core/Cargo.toml`

Actions:
1. In workspace root `Cargo.toml`: uncomment `chromiumoxide = "0.7"`, add `futures = "0.3"`
2. In `moron-core/Cargo.toml`: uncomment `chromiumoxide = { workspace = true }`, add `futures = { workspace = true }`

**Verification:** `cargo check -p moron-core` compiles (chromium.rs is still a stub,
so it won't use the new deps yet, but they must resolve).

---

### Step 2: Define BridgeError Enum

**File:** `moron-core/src/chromium.rs`

Write the error enum with all seven variants. Each variant uses `#[error("...")]`
from thiserror. Source-wrapping variants use `#[source] anyhow::Error` to preserve
the chromiumoxide error chain.

Variants:
- `LaunchFailed(anyhow::Error)`
- `ChromeNotFound`
- `PageLoadFailed { path: String, source: anyhow::Error }`
- `JsEvalFailed(anyhow::Error)`
- `ScreenshotFailed(anyhow::Error)`
- `RenderTimeout { timeout_secs: u64 }`
- `AlreadyClosed`

**Verification:** Types compile with `cargo check`.

---

### Step 3: Define BridgeConfig Struct

**File:** `moron-core/src/chromium.rs`

Write the config struct with six fields:
- `width: u32` (default 1920)
- `height: u32` (default 1080)
- `html_path: PathBuf` (required)
- `chrome_executable: Option<PathBuf>` (default None)
- `headless: bool` (default true)
- `launch_timeout: Duration` (default 20s)

Implement `BridgeConfig::new(html_path)` that sets defaults for everything else.

**Verification:** `cargo check`.

---

### Step 4: Define ChromiumBridge Struct

**File:** `moron-core/src/chromium.rs`

Write the struct with three private fields:
- `browser: Browser`
- `page: Page`
- `handler_handle: JoinHandle<()>`

No methods yet -- just the struct definition.

**Verification:** `cargo check`.

---

### Step 5: Implement ChromiumBridge::launch()

**File:** `moron-core/src/chromium.rs`

Async constructor:
1. Build `BrowserConfig` from `BridgeConfig`:
   - Set `window_size(width, height)`
   - Set `viewport(Viewport { width, height, device_scale_factor: Some(1.0), .. })`
   - Conditionally call `.with_head()` if `!config.headless`
   - Conditionally call `.chrome_executable(path)` if set
   - Set `.launch_timeout(config.launch_timeout)`
   - Call `.no_sandbox()` for compatibility
2. `Browser::launch(browser_config).await` -- map error to `BridgeError::LaunchFailed`
3. Spawn handler: `tokio::spawn(async move { while handler.next().await.is_some() {} })`
4. Build file URL: `format!("file://{}", config.html_path.canonicalize()?.display())`
5. `browser.new_page(url).await` -- map error to `PageLoadFailed`
6. Verify contract: evaluate `typeof window.__moron_setFrame === 'function'`
   - If false, return `PageLoadFailed` with descriptive message
7. Return `ChromiumBridge { browser, page, handler_handle }`

**Verification:** `cargo check`. No runtime test (requires Chrome).

---

### Step 6: Implement wait_for_render()

**File:** `moron-core/src/chromium.rs`

Private async method on `ChromiumBridge`:
1. Build JS expression: double rAF wrapped in a Promise
2. Wrap `evaluate_expression` call in `tokio::time::timeout(Duration::from_secs(5), ...)`
3. On timeout, return `BridgeError::RenderTimeout { timeout_secs: 5 }`
4. On eval error, return `BridgeError::JsEvalFailed`

**Verification:** `cargo check`.

---

### Step 7: Implement capture_frame()

**File:** `moron-core/src/chromium.rs`

Public async method:
1. Format JS: `format!("window.__moron_setFrame({})", frame_json)`
2. `self.page.evaluate_expression(js).await` -- map error to `JsEvalFailed`
3. `self.wait_for_render().await?`
4. Build `ScreenshotParams` with PNG format, not full page, not omitting background
5. `self.page.screenshot(params).await` -- map error to `ScreenshotFailed`
6. Return `Vec<u8>`

**Verification:** `cargo check`.

---

### Step 8: Implement close() and Drop

**File:** `moron-core/src/chromium.rs`

`close(mut self) -> Result<(), BridgeError>`:
1. `self.browser.close().await` -- map error to `LaunchFailed` (reuse variant)
2. `let _ = self.handler_handle.await` -- ignore JoinError
3. Return `Ok(())`

`Drop for ChromiumBridge`:
1. `self.handler_handle.abort()` -- best-effort cleanup

**Verification:** `cargo check`.

---

### Step 9: Full Workspace Verification

Run `cargo check` on the full workspace to ensure:
- No compile errors in any crate
- No unused import warnings that break the build
- The chromium module integrates cleanly with lib.rs

Run `cargo clippy -p moron-core` if possible for lint cleanliness.

---

## Testing Strategy

### What Gets Unit Tests

Nothing in this ticket. The bridge is pure I/O -- every method talks to a real
Chrome process. Unit tests would require either:
- A running Chrome instance (integration test territory)
- Mocking chromiumoxide internals (brittle and low value)

### What Gets Integration Tests

Integration tests belong in a later ticket or CI setup where Chrome is available.
The acceptance criteria "can launch headless Chrome" is verified manually or in CI.

### Verification Criteria

1. `cargo check` passes with zero errors
2. `cargo check -p moron-core` passes
3. The public API surface matches the design:
   - `ChromiumBridge::launch(BridgeConfig) -> Result<Self, BridgeError>`
   - `ChromiumBridge::capture_frame(&self, &str) -> Result<Vec<u8>, BridgeError>`
   - `ChromiumBridge::close(self) -> Result<(), BridgeError>`
4. All seven `BridgeError` variants are defined
5. `BridgeConfig::new(path)` works with sensible defaults
6. Drop impl provides fallback cleanup

---

## Step Ordering Rationale

Steps 1 (deps) must come first -- nothing compiles without chromiumoxide.
Steps 2-4 (types) are independent of each other but establish the foundation.
Steps 5-8 (methods) depend on the types and build sequentially on each other:
launch creates the bridge, wait_for_render is used by capture_frame, close
tears it down.
Step 9 (verification) is the final gate.

All steps modify the same file (chromium.rs) except Step 1. In practice, Steps
2-8 will be written as a single coherent file and compiled once. The step
breakdown is for logical clarity, not separate commits.
