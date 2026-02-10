# T-007-04 Design: Visual Regression Tests

## Approach Options

### Option A: External image comparison library (e.g., image crate + pixel diff)
- Pros: Proper perceptual comparison, structural similarity
- Cons: Adds heavy dependency (image crate), over-engineered for initial tests

### Option B: Byte-level PNG comparison with size tolerance
- Pros: Zero dependencies, simple
- Cons: Too brittle -- any rendering difference fails

### Option C: Raw byte comparison with configurable tolerance percentage
- Pros: Simple, no new deps, captures structural changes while allowing minor diffs
- Cons: Not perceptually aware
- Mitigation: Use file-size difference as coarse metric + exact match as ideal

### Decision: Option C (with refinement)

The comparison strategy:
1. If baseline does not exist, save the capture as baseline, pass the test
2. If baseline exists and sizes match within tolerance, pass
3. Additionally compare byte-by-byte with a mismatch percentage threshold

This is intentionally simple. The goal is catching regressions (wrong element, broken template, theme not applied), not pixel-perfect matching across platforms.

## Test Architecture

- New file: `moron-core/tests/visual.rs`
- All tests `#[ignore]` (need Chrome + React app)
- Each test constructs a specific `M` -> `compute_frame_state` -> serialize JSON -> `ChromiumBridge::capture_frame` -> compare PNG against baseline
- Baseline directory: `moron-core/tests/baselines/`
- Baselines named after test case: `title_card_dark.png`, `metric_up.png`, etc.

## Test Cases

1. **title_card_dark** -- Title element with dark theme
2. **metric_display_up** -- Metric element with direction=up
3. **steps_list** -- Steps element with 3 items
4. **theme_switching** -- Same title element, dark vs light theme (two baselines, verify they differ)

## Environment Requirements

- `MORON_HTML_PATH` env var pointing to built React app index.html
- Chrome/Chromium installed and on PATH
- Run command: `cargo test --test visual -- --ignored`

## Rejected Alternatives

- **Snapshot testing crate (insta)**: Would add dependency, and insta is for text snapshots not PNGs
- **Headless rendering without Chrome**: Would require wgpu/vello, not available yet
- **Screenshot-only (no comparison)**: Provides no regression detection
