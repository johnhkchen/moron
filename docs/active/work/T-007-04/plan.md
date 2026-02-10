# T-007-04 Plan: Visual Regression Tests

## Steps

### Step 1: Create baselines directory
- Create `moron-core/tests/baselines/` with a `.gitkeep` file
- Verify directory exists

### Step 2: Write visual.rs test file
- Module doc comment with running instructions
- Import moron_core::prelude::*, moron_themes::Theme
- Helper: baselines_dir() -- path relative to CARGO_MANIFEST_DIR
- Helper: baseline_path(name) -- full path to a baseline PNG
- Helper: compare_png_bytes(actual, baseline, tolerance) -- compare with byte tolerance
- Helper: check_or_create_baseline(name, actual, tolerance) -- create-or-compare logic
- Helper: build_bridge() -> async setup ChromiumBridge from MORON_HTML_PATH
- Helper: capture_frame_state(bridge, frame_state) -> async capture PNG bytes

### Step 3: Implement test_visual_title_card_dark
- Build M with m.title("Visual Test") + m.wait(0.5)
- compute_frame_state at t=0.0
- Capture via bridge, compare/create baseline "title_card_dark.png"

### Step 4: Implement test_visual_metric_display
- Build M with m.metric("Revenue", "$1.2M", Direction::Up) + m.wait(0.5)
- compute_frame_state at t=0.0
- Compare/create baseline "metric_up.png"

### Step 5: Implement test_visual_steps_list
- Build M with m.steps(&["First", "Second", "Third"]) + m.wait(0.5)
- compute_frame_state at t=0.0
- Compare/create baseline "steps_list.png"

### Step 6: Implement test_visual_theme_switching
- Build M with dark theme, m.title("Theme Test") + m.wait(0.5)
- Capture dark baseline
- Build M with light theme, same content
- Capture light baseline
- Verify dark and light PNGs differ (byte content not equal)

### Step 7: Update ticket frontmatter
- Set status: done, phase: done

### Step 8: Verify
- `cargo check` passes
- `cargo test` (non-ignored) passes
- `cargo test --test visual` compiles (even though tests are ignored by default)

## Testing Strategy
- All visual tests are `#[ignore]` -- they need Chrome + built React app
- Non-ignored tests unaffected
- CI without Chrome: tests simply skipped
- Local run: `MORON_HTML_PATH=/path/to/index.html cargo test --test visual -- --ignored`
