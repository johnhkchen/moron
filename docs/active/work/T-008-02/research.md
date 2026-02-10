# T-008-02 Research: clippy-pedantic-techniques

## Current State

`cargo clippy -p moron-techniques -- -W clippy::pedantic` produces **34 warnings** across 8 files.

## Warning Inventory

### 1. `unreadable_literal` (1 warning)
- `technique.rs:63` — `0.984375` in `ease_out_bounce()` needs digit separators

### 2. `must_use_candidate` (2 warnings)
- `technique.rs:24` — `pub fn ease()` return value should be marked `#[must_use]`
- `data.rs:28` — `pub fn current_value()` return value should be marked `#[must_use]`

### 3. `return_self_not_must_use` (2 warnings)
- `staging.rs:26` — `with_delay()` returns `Self`, needs `#[must_use]`
- `staging.rs:32` — `with_count()` returns `Self`, needs `#[must_use]`

### 4. `doc_markdown` (23 warnings)
CamelCase type names in module-level doc comments need backticks:
- `reveals.rs:1` — TypeWriter, SweepIn, PixelDissolve, MaskWipe (4)
- `motion.rs:1` — SlideIn, ArcPath, SpringPop (3)
- `staging.rs:1` — GridLayout, StackReveal, SplitScreen (3)
- `emphasis.rs:1` — ColorShift (1)
- `camera.rs:1` — ZoomIn, PanTo, DollyZoom, FocusPull (4)
- `transitions.rs:1` — CrossFade, CutTo, IrisWipe (3)
- `data.rs:1` — BarChartRace, CountUp, FlowDiagram (3)
Note: names that are plain English words (Morph, Orbit, Shake, Pulse, Highlight, Underline, Annotate) are not flagged.

### 5. `unnecessary_literal_bound` (6 warnings)
`fn name(&self) -> &str` returns string literals in impls; should use `&'static str`:
- `reveals.rs:18` (FadeIn), `reveals.rs:51` (FadeUp)
- `motion.rs:24` (Slide), `motion.rs:61` (Scale)
- `staging.rs:66` (Stagger)
- `data.rs:35` (CountUp)

### 6. `cast_precision_loss` (2 warnings)
`usize` to `f64` casts in Stagger:
- `staging.rs:48` — `index as f64`
- `staging.rs:73` — `(self.count - 1) as f64`

## Files Affected

| File | Warnings | Categories |
|------|----------|------------|
| technique.rs | 2 | unreadable_literal, must_use_candidate |
| reveals.rs | 6 | doc_markdown (4), unnecessary_literal_bound (2) |
| motion.rs | 5 | doc_markdown (3), unnecessary_literal_bound (2) |
| staging.rs | 6 | doc_markdown (3), return_self_not_must_use (2), unnecessary_literal_bound (1), cast_precision_loss (2) — note some overlap |
| emphasis.rs | 1 | doc_markdown |
| camera.rs | 4 | doc_markdown |
| transitions.rs | 3 | doc_markdown |
| data.rs | 5 | doc_markdown (3), must_use_candidate (1), unnecessary_literal_bound (1) |

## Constraints

- The `Technique` trait in technique.rs defines `fn name(&self) -> &str`. Changing impl return types to `&'static str` is valid Rust (more specific lifetime satisfies the trait bound).
- The `WithEase<T>` impl of `name()` delegates to `self.inner.name()` — returns whatever the inner technique returns, which will be `&'static str` from concrete impls, but the trait bound is `&str`.
- The `cast_precision_loss` warnings are false positives for our use case (element counts/indices, always small).
- No behavioral changes permitted — all fixes are annotations and formatting only.
