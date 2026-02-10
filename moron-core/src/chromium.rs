//! Chromium bridge: headless browser orchestration via chromiumoxide.
//!
//! Connects Rust to the React rendering layer via Chrome DevTools Protocol (CDP).
//! The bridge launches headless Chrome, loads the React app, and for each frame:
//! injects FrameState JSON, waits for React to render, captures a PNG screenshot.

use std::path::PathBuf;
use std::time::Duration;

use chromiumoxide::browser::BrowserConfig;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use tokio::task::JoinHandle;

// ---------------------------------------------------------------------------
// BridgeError
// ---------------------------------------------------------------------------

/// Errors produced by the Chromium bridge.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    /// Chrome process failed to start.
    #[error("failed to launch Chrome: {0}")]
    LaunchFailed(#[source] anyhow::Error),

    /// No Chrome/Chromium binary found on the system.
    #[error("Chrome executable not found")]
    ChromeNotFound,

    /// The HTML host page could not be loaded.
    #[error("failed to load page: {path}")]
    PageLoadFailed {
        /// Path or URL that failed to load.
        path: String,
        /// Underlying error.
        #[source]
        source: anyhow::Error,
    },

    /// A JavaScript evaluation failed.
    #[error("JavaScript evaluation failed: {0}")]
    JsEvalFailed(#[source] anyhow::Error),

    /// Screenshot capture failed.
    #[error("screenshot capture failed: {0}")]
    ScreenshotFailed(#[source] anyhow::Error),

    /// The page did not finish rendering within the timeout.
    #[error("render timed out after {timeout_secs}s")]
    RenderTimeout {
        /// Timeout duration in seconds.
        timeout_secs: u64,
    },

    /// Methods were called on a bridge that has already been closed.
    #[error("bridge already closed")]
    AlreadyClosed,
}

// ---------------------------------------------------------------------------
// BridgeConfig
// ---------------------------------------------------------------------------

/// Configuration for launching the Chromium bridge.
pub struct BridgeConfig {
    /// Viewport and screenshot width in pixels.
    pub width: u32,
    /// Viewport and screenshot height in pixels.
    pub height: u32,
    /// Path to the built React app's `index.html`.
    pub html_path: PathBuf,
    /// Override automatic Chrome/Chromium detection with a specific binary path.
    pub chrome_executable: Option<PathBuf>,
    /// Run Chrome in headless mode. Set to `false` for debugging.
    pub headless: bool,
    /// Maximum time to wait for Chrome to start.
    pub launch_timeout: Duration,
}

impl BridgeConfig {
    /// Create a new config with sensible defaults.
    ///
    /// The `html_path` is required -- it must point to the built React app's
    /// `index.html` file. All other fields use defaults:
    /// - 1920x1080 viewport
    /// - headless mode
    /// - 20-second launch timeout
    /// - automatic Chrome detection
    pub fn new(html_path: impl Into<PathBuf>) -> Self {
        Self {
            width: 1920,
            height: 1080,
            html_path: html_path.into(),
            chrome_executable: None,
            headless: true,
            launch_timeout: Duration::from_secs(20),
        }
    }
}

// ---------------------------------------------------------------------------
// ChromiumBridge
// ---------------------------------------------------------------------------

/// Headless Chromium bridge for capturing rendered frames as PNG screenshots.
///
/// Owns a Chrome process (via `Browser`), a single page/tab, and the background
/// handler task that drives CDP WebSocket communication.
///
/// # Usage
///
/// ```ignore
/// let config = BridgeConfig::new("path/to/index.html");
/// let bridge = ChromiumBridge::launch(config).await?;
///
/// for frame_json in frame_jsons {
///     let png_bytes = bridge.capture_frame(&frame_json).await?;
///     // write png_bytes to disk or pipe to FFmpeg
/// }
///
/// bridge.close().await?;
/// ```
pub struct ChromiumBridge {
    browser: Browser,
    page: Page,
    /// Wrapped in `Option` so `close()` can take ownership without conflicting
    /// with the `Drop` implementation. Always `Some` until `close()` is called.
    handler_handle: Option<JoinHandle<()>>,
}

impl ChromiumBridge {
    /// Launch headless Chrome and load the React app page.
    ///
    /// This spawns a new Chrome process, creates a page, navigates to the
    /// HTML file specified in `config.html_path`, and verifies that the page
    /// exposes the `window.__moron_setFrame` function.
    pub async fn launch(config: BridgeConfig) -> Result<Self, BridgeError> {
        // Build chromiumoxide BrowserConfig from our BridgeConfig.
        let mut builder = BrowserConfig::builder()
            .window_size(config.width, config.height)
            .viewport(Viewport {
                width: config.width,
                height: config.height,
                device_scale_factor: Some(1.0),
                emulating_mobile: false,
                is_landscape: true,
                has_touch: false,
            })
            .no_sandbox()
            .launch_timeout(config.launch_timeout);

        if !config.headless {
            builder = builder.with_head();
        }

        if let Some(ref executable) = config.chrome_executable {
            builder = builder.chrome_executable(executable);
        }

        let browser_config = builder
            .build()
            .map_err(|e| BridgeError::LaunchFailed(anyhow::anyhow!(e)))?;

        // Launch Chrome. Returns (Browser, Handler) where Handler is a Stream
        // that must be polled continuously to drive CDP communication.
        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| BridgeError::LaunchFailed(anyhow::anyhow!(e)))?;

        // Spawn the handler polling loop in a background task.
        let handler_handle = tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        // Build file:// URL from the HTML path.
        let canonical_path = config
            .html_path
            .canonicalize()
            .map_err(|e| BridgeError::PageLoadFailed {
                path: config.html_path.display().to_string(),
                source: anyhow::anyhow!("failed to resolve HTML path: {}", e),
            })?;

        let file_url = format!("file://{}", canonical_path.display());

        // Navigate to the React app page.
        let page = browser.new_page(&file_url).await.map_err(|e| {
            BridgeError::PageLoadFailed {
                path: file_url.clone(),
                source: anyhow::anyhow!(e),
            }
        })?;

        // Verify the page exposes the __moron_setFrame function.
        let check_result = page
            .evaluate_expression("typeof window.__moron_setFrame === 'function'")
            .await
            .map_err(|e| BridgeError::PageLoadFailed {
                path: file_url.clone(),
                source: anyhow::anyhow!("failed to check page contract: {}", e),
            })?;

        let has_set_frame: bool = check_result.into_value().unwrap_or(false);

        if !has_set_frame {
            return Err(BridgeError::PageLoadFailed {
                path: file_url,
                source: anyhow::anyhow!(
                    "page does not expose window.__moron_setFrame function; \
                     the React app must define this global function"
                ),
            });
        }

        Ok(Self {
            browser,
            page,
            handler_handle: Some(handler_handle),
        })
    }

    /// Capture a single rendered frame as PNG bytes.
    ///
    /// Injects `frame_json` into the page via `window.__moron_setFrame()`,
    /// waits for React to render (double requestAnimationFrame), and captures
    /// a viewport-sized PNG screenshot.
    ///
    /// `frame_json` must be a valid JSON string (produced by `serde_json::to_string`
    /// on a `FrameState`). It is interpolated directly into a JavaScript expression.
    pub async fn capture_frame(&self, frame_json: &str) -> Result<Vec<u8>, BridgeError> {
        // Inject the FrameState JSON into the page.
        let js = format!("window.__moron_setFrame({})", frame_json);
        self.page
            .evaluate_expression(js)
            .await
            .map_err(|e| BridgeError::JsEvalFailed(anyhow::anyhow!(e)))?;

        // Wait for React to render and the browser to paint.
        self.wait_for_render().await?;

        // Capture the viewport as a PNG screenshot.
        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(false)
            .omit_background(false)
            .build();

        let png_bytes = self
            .page
            .screenshot(params)
            .await
            .map_err(|e| BridgeError::ScreenshotFailed(anyhow::anyhow!(e)))?;

        Ok(png_bytes)
    }

    /// Gracefully shut down Chrome and clean up resources.
    ///
    /// This closes the browser, waits for the Chrome process to exit, and
    /// joins the handler task. After calling `close()`, the bridge is consumed
    /// and cannot be used again.
    pub async fn close(mut self) -> Result<(), BridgeError> {
        self.browser
            .close()
            .await
            .map_err(|e| BridgeError::LaunchFailed(anyhow::anyhow!(e)))?;

        // Wait for the handler task to finish. Ignore JoinError (task may have
        // already exited when the browser closed).
        if let Some(handle) = self.handler_handle.take() {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Wait for the browser to finish rendering after a state update.
    ///
    /// Uses the double requestAnimationFrame pattern: the first rAF fires before
    /// the next paint, the second fires after it, guaranteeing that DOM mutations
    /// from React have been painted.
    ///
    /// Times out after 5 seconds to catch hung pages or JS errors.
    async fn wait_for_render(&self) -> Result<(), BridgeError> {
        let js = r#"new Promise(resolve => {
            requestAnimationFrame(() => {
                requestAnimationFrame(() => resolve(true))
            })
        })"#;

        const RENDER_TIMEOUT_SECS: u64 = 5;

        let result = tokio::time::timeout(
            Duration::from_secs(RENDER_TIMEOUT_SECS),
            self.page.evaluate_expression(js),
        )
        .await;

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(BridgeError::JsEvalFailed(anyhow::anyhow!(e))),
            Err(_elapsed) => Err(BridgeError::RenderTimeout {
                timeout_secs: RENDER_TIMEOUT_SECS,
            }),
        }
    }
}

impl Drop for ChromiumBridge {
    fn drop(&mut self) {
        // Best-effort cleanup: abort the handler task if it hasn't been taken
        // by close(). The Browser's own Drop implementation kills the Chrome
        // child process.
        if let Some(handle) = self.handler_handle.take() {
            handle.abort();
        }
    }
}
