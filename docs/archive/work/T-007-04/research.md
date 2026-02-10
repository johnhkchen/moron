# T-007-04 Research: Visual Regression Tests

## What Exists

### Test Infrastructure
- `moron-core/tests/e2e.rs` -- end-to-end pipeline tests
  - Pattern: `#[test]` + `#[ignore]` for tests requiring system dependencies (FFmpeg, Chrome)
  - Uses `moron_core::prelude::*` for all imports
  - Helper functions for synthetic PNG generation, temp dirs, ffprobe validation
  - Tests range from pure-Rust (no ignore) to full pipeline (ignored)
- `moron-core/tests/integration.rs` -- exists alongside e2e

### Frame State System
- `moron-core/src/frame.rs` -- `FrameState`, `ElementState`, `ElementKind`, `ThemeState`
  - `compute_frame_state(m: &M, time: f64) -> FrameState`
  - All types derive `Serialize`, `Deserialize`, `Clone`, `PartialEq`
  - ElementKind variants: Title, Show, Section, Metric{direction}, Steps{count}
  - FrameState contains: time, frame, total_duration, fps, elements, active_narration, theme

### Chromium Bridge
- `moron-core/src/chromium.rs` -- `ChromiumBridge`, `BridgeConfig`
  - `BridgeConfig::new(html_path)` -- defaults to 1920x1080, headless, 20s timeout
  - `ChromiumBridge::launch(config).await` -- spawns headless Chrome
  - `bridge.capture_frame(frame_json).await` -- injects JSON, waits for render, captures PNG
  - `bridge.close().await` -- graceful shutdown
  - Requires Chrome binary + built React app HTML

### Facade (M) and Scene Trait
- `moron-core/src/facade.rs` -- `M::new()`, `m.title()`, `m.show()`, `m.section()`, `m.metric()`, `m.steps()`, `m.theme()`
- `Direction` enum: Up, Down, Neutral
- `Theme::default()` = moron-dark, `Theme::light()` = moron-light

### Theme System
- `moron-themes/src/defaults.rs` -- `Theme::default()` (dark), `Theme::light()`
- 56 CSS custom properties per theme
- Dark: bg=#0f172a, Light: bg=#ffffff

### Renderer
- `moron-core/src/renderer.rs` -- `render(m, config).await`
  - Full pipeline: compute_frame_state -> serialize JSON -> ChromiumBridge.capture_frame -> save PNG
  - `RenderConfig` wraps `BridgeConfig` + output_dir

### React Template
- `packages/ui/src/templates/ExplainerTemplate.tsx` -- renders all 5 element types
- Self-registers as "explainer" in template registry

### Dependencies
- `moron-core/Cargo.toml` -- depends on moron-techniques, moron-voice, moron-themes, serde, serde_json, tokio, chromiumoxide

## Key Observations

1. ChromiumBridge is async -- tests need tokio runtime
2. Tests requiring Chrome use `#[ignore]` -- same pattern needed here
3. e2e.rs already has PNG generation helpers (CRC32, zlib compression)
4. Frame state JSON is the input to Chrome; PNG bytes are the output
5. No existing visual regression or pixel comparison infrastructure
6. The e2e tests use synthetic PNGs, never actual Chrome renders

## Constraints
- Chrome + built React app required for real renders -- must be #[ignore]d
- No external image comparison crate in workspace -- keep it simple
- Baselines don't exist yet -- first run creates them
- Anti-aliasing differences across platforms -- need tolerance
