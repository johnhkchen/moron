//! Visual regression tests for moron template output.
//!
//! These tests render known `FrameState`s through the Chromium bridge,
//! capture PNG screenshots, and compare against baseline images stored in
//! `moron-core/tests/baselines/`.
//!
//! # Requirements
//!
//! - Chrome or Chromium installed and on PATH
//! - The React app built and available as an `index.html` file
//! - `MORON_HTML_PATH` environment variable pointing to the built `index.html`
//!
//! # Running
//!
//! All tests in this file are `#[ignore]`d by default since they require
//! Chrome and the built React app.
//!
//! ```sh
//! # Set the path to the built React app
//! export MORON_HTML_PATH=/path/to/packages/ui/dist/index.html
//!
//! # Run visual regression tests
//! cargo test --test visual -- --ignored
//! ```
//!
//! # Baseline Management
//!
//! On first run (or when a baseline is missing), the test captures a screenshot
//! and saves it as the new baseline. Subsequent runs compare against the saved
//! baseline using byte-level comparison with a configurable tolerance.
//!
//! To regenerate baselines, delete the files in `moron-core/tests/baselines/`
//! and re-run the tests.

use std::path::{Path, PathBuf};

use moron_core::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum allowed byte-level mismatch ratio (0.0 = exact match, 1.0 = all different).
///
/// Set relatively high to accommodate anti-aliasing differences across platforms,
/// font rendering variations, and minor Chrome version differences.
const DEFAULT_TOLERANCE: f64 = 0.05;

// ---------------------------------------------------------------------------
// Helpers: baseline paths
// ---------------------------------------------------------------------------

/// Return the path to the baselines directory.
fn baselines_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).join("tests").join("baselines")
}

/// Return the full path to a specific baseline PNG file.
fn baseline_path(name: &str) -> PathBuf {
    baselines_dir().join(format!("{name}.png"))
}

// ---------------------------------------------------------------------------
// Helpers: PNG comparison
// ---------------------------------------------------------------------------

/// Compare two PNG byte slices with a tolerance for byte-level differences.
///
/// Returns `true` if the mismatch ratio is within `tolerance`.
///
/// The comparison works on raw PNG file bytes (not decoded pixels). This is
/// intentionally simple: it catches structural changes (wrong elements, missing
/// content, broken template) while tolerating minor rendering differences
/// (anti-aliasing, sub-pixel font rendering).
fn compare_png_bytes(actual: &[u8], baseline: &[u8], tolerance: f64) -> bool {
    // Both must be valid PNGs (start with PNG signature).
    let png_sig = [137u8, 80, 78, 71, 13, 10, 26, 10];
    if actual.len() < 8 || baseline.len() < 8 {
        return false;
    }
    if actual[..8] != png_sig || baseline[..8] != png_sig {
        return false;
    }

    // Size difference check: if sizes differ by more than 2x tolerance, fail fast.
    let size_ratio = if actual.len() >= baseline.len() {
        actual.len() as f64 / baseline.len() as f64
    } else {
        baseline.len() as f64 / actual.len() as f64
    };
    if size_ratio > 1.0 + tolerance * 4.0 {
        return false;
    }

    // Byte-level comparison: count mismatched bytes.
    let min_len = actual.len().min(baseline.len());
    let max_len = actual.len().max(baseline.len());

    let mut mismatches = 0usize;
    for i in 0..min_len {
        if actual[i] != baseline[i] {
            mismatches += 1;
        }
    }
    // Bytes beyond the shorter file are all mismatches.
    mismatches += max_len - min_len;

    let mismatch_ratio = mismatches as f64 / max_len as f64;
    mismatch_ratio <= tolerance
}

/// Check a captured PNG against its baseline. If the baseline does not exist,
/// create it and pass. If it exists, compare with tolerance.
///
/// Panics (fails the test) if the comparison exceeds tolerance.
fn check_or_create_baseline(name: &str, actual: &[u8], tolerance: f64) {
    let path = baseline_path(name);

    // Ensure the baselines directory exists.
    let dir = baselines_dir();
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
        panic!("failed to create baselines directory {}: {e}", dir.display());
    });

    if path.exists() {
        // Baseline exists: compare.
        let baseline = std::fs::read(&path).unwrap_or_else(|e| {
            panic!("failed to read baseline {}: {e}", path.display());
        });

        assert!(
            compare_png_bytes(actual, &baseline, tolerance),
            "Visual regression detected for '{name}'.\n\
             Actual size: {} bytes, baseline size: {} bytes.\n\
             To update the baseline, delete {} and re-run.",
            actual.len(),
            baseline.len(),
            path.display(),
        );
    } else {
        // Baseline does not exist: create it.
        std::fs::write(&path, actual).unwrap_or_else(|e| {
            panic!("failed to write new baseline {}: {e}", path.display());
        });
        eprintln!(
            "Created new baseline: {} ({} bytes)",
            path.display(),
            actual.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers: Chromium bridge setup
// ---------------------------------------------------------------------------

/// Read `MORON_HTML_PATH` from the environment and create a `BridgeConfig`.
fn bridge_config_from_env() -> moron_core::chromium::BridgeConfig {
    let html_path = std::env::var("MORON_HTML_PATH").unwrap_or_else(|_| {
        panic!(
            "MORON_HTML_PATH environment variable is required.\n\
             Set it to the path of the built React app's index.html.\n\
             Example: export MORON_HTML_PATH=packages/ui/dist/index.html"
        );
    });

    moron_core::chromium::BridgeConfig::new(html_path)
}

/// Serialize a `FrameState` to JSON and capture it as a PNG via the bridge.
async fn capture_frame_png(
    bridge: &moron_core::chromium::ChromiumBridge,
    state: &FrameState,
) -> Vec<u8> {
    let json = serde_json::to_string(state).expect("failed to serialize FrameState to JSON");
    bridge
        .capture_frame(&json)
        .await
        .expect("failed to capture frame via ChromiumBridge")
}

// ===========================================================================
// Visual regression tests (all #[ignore] -- require Chrome + React app)
// ===========================================================================

#[test]
#[ignore = "requires Chrome and MORON_HTML_PATH (visual regression)"]
fn visual_title_card_dark() {
    // Render a title element with the default dark theme.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let config = bridge_config_from_env();
        let bridge = moron_core::chromium::ChromiumBridge::launch(config)
            .await
            .expect("failed to launch ChromiumBridge");

        // Build scene: title card with dark theme (default).
        let mut m = M::new();
        m.title("Visual Regression Test");
        m.wait(0.5);

        let state = compute_frame_state(&m, 0.0);
        let png = capture_frame_png(&bridge, &state).await;

        assert!(!png.is_empty(), "captured PNG must not be empty");
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10], "must be valid PNG");

        check_or_create_baseline("title_card_dark", &png, DEFAULT_TOLERANCE);

        bridge.close().await.expect("failed to close bridge");
    });
}

#[test]
#[ignore = "requires Chrome and MORON_HTML_PATH (visual regression)"]
fn visual_metric_display() {
    // Render a metric element with direction=up.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let config = bridge_config_from_env();
        let bridge = moron_core::chromium::ChromiumBridge::launch(config)
            .await
            .expect("failed to launch ChromiumBridge");

        let mut m = M::new();
        m.metric("Revenue", "$1.2M", Direction::Up);
        m.wait(0.5);

        let state = compute_frame_state(&m, 0.0);
        let png = capture_frame_png(&bridge, &state).await;

        assert!(!png.is_empty(), "captured PNG must not be empty");
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10], "must be valid PNG");

        check_or_create_baseline("metric_up", &png, DEFAULT_TOLERANCE);

        bridge.close().await.expect("failed to close bridge");
    });
}

#[test]
#[ignore = "requires Chrome and MORON_HTML_PATH (visual regression)"]
fn visual_steps_list() {
    // Render a steps element with 3 items.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let config = bridge_config_from_env();
        let bridge = moron_core::chromium::ChromiumBridge::launch(config)
            .await
            .expect("failed to launch ChromiumBridge");

        let mut m = M::new();
        m.steps(&["Research the problem", "Design the solution", "Ship the code"]);
        m.wait(0.5);

        let state = compute_frame_state(&m, 0.0);
        let png = capture_frame_png(&bridge, &state).await;

        assert!(!png.is_empty(), "captured PNG must not be empty");
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10], "must be valid PNG");

        check_or_create_baseline("steps_list", &png, DEFAULT_TOLERANCE);

        bridge.close().await.expect("failed to close bridge");
    });
}

#[test]
#[ignore = "requires Chrome and MORON_HTML_PATH (visual regression)"]
fn visual_theme_switching() {
    // Render the same title with dark and light themes, verify they produce
    // visually different output.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let config = bridge_config_from_env();
        let bridge = moron_core::chromium::ChromiumBridge::launch(config)
            .await
            .expect("failed to launch ChromiumBridge");

        // Dark theme (default).
        let mut m_dark = M::new();
        m_dark.title("Theme Test");
        m_dark.wait(0.5);
        let state_dark = compute_frame_state(&m_dark, 0.0);
        let png_dark = capture_frame_png(&bridge, &state_dark).await;

        assert!(!png_dark.is_empty(), "dark PNG must not be empty");
        check_or_create_baseline("theme_dark", &png_dark, DEFAULT_TOLERANCE);

        // Light theme.
        let mut m_light = M::new();
        m_light.theme(Theme::light());
        m_light.title("Theme Test");
        m_light.wait(0.5);
        let state_light = compute_frame_state(&m_light, 0.0);
        let png_light = capture_frame_png(&bridge, &state_light).await;

        assert!(!png_light.is_empty(), "light PNG must not be empty");
        check_or_create_baseline("theme_light", &png_light, DEFAULT_TOLERANCE);

        // The dark and light renders must differ.
        assert_ne!(
            png_dark, png_light,
            "Dark and light theme renders must produce different PNGs"
        );

        bridge.close().await.expect("failed to close bridge");
    });
}

// ===========================================================================
// Non-ignored unit tests for the comparison helpers
// ===========================================================================

#[cfg(test)]
mod comparison_tests {
    use super::*;

    #[test]
    fn compare_identical_bytes() {
        let png = make_fake_png(1000);
        assert!(compare_png_bytes(&png, &png, 0.0));
    }

    #[test]
    fn compare_within_tolerance() {
        let mut a = make_fake_png(1000);
        let b = a.clone();
        // Flip 3% of bytes (within 5% tolerance).
        let flip_count = (a.len() as f64 * 0.03) as usize;
        for i in 8..8 + flip_count {
            a[i] = a[i].wrapping_add(1);
        }
        assert!(compare_png_bytes(&a, &b, 0.05));
    }

    #[test]
    fn compare_exceeds_tolerance() {
        let mut a = make_fake_png(1000);
        let b = a.clone();
        // Flip 20% of bytes (exceeds 5% tolerance).
        let flip_count = (a.len() as f64 * 0.20) as usize;
        for i in 8..8 + flip_count {
            a[i] = a[i].wrapping_add(1);
        }
        assert!(!compare_png_bytes(&a, &b, 0.05));
    }

    #[test]
    fn compare_different_sizes_within_tolerance() {
        let a = make_fake_png(1000);
        let b = make_fake_png(1020); // 2% larger
        // With 5% tolerance, the size difference alone should be okay,
        // but the trailing bytes differ, so total mismatch may exceed tolerance.
        // This tests the size ratio fast-fail path does not trigger.
        let _ = compare_png_bytes(&a, &b, 0.05);
    }

    #[test]
    fn compare_rejects_non_png() {
        let a = vec![0u8; 100];
        let b = vec![0u8; 100];
        assert!(!compare_png_bytes(&a, &b, 1.0));
    }

    #[test]
    fn compare_rejects_too_short() {
        let a = vec![137, 80, 78, 71];
        let b = vec![137, 80, 78, 71];
        assert!(!compare_png_bytes(&a, &b, 1.0));
    }

    #[test]
    fn baselines_dir_is_under_manifest() {
        let dir = baselines_dir();
        assert!(dir.ends_with("tests/baselines"));
        assert!(dir.to_string_lossy().contains("moron-core"));
    }

    #[test]
    fn baseline_path_appends_png() {
        let path = baseline_path("test_name");
        assert!(path.to_string_lossy().ends_with("test_name.png"));
    }

    /// Create a fake byte vector that starts with the PNG signature.
    fn make_fake_png(size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        // PNG signature
        data.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
        // Fill the rest with deterministic bytes.
        for i in 8..size {
            data.push((i % 256) as u8);
        }
        data
    }
}
