# Moron Roadmap

> Build roadmap for the Moron motion graphics engine, spanning 12 quarters (2026--2028).

Year 1 builds the foundation. Year 2 polishes and extends. Year 3 stabilizes for long-term maintenance.

**Current status:** Q1 2026 -- IN PROGRESS (Scaffold & proof of concept)

---

## Year 1: Foundation (Q1--Q4 2026)

### Q1 2026 -- Scaffold & Proof of Concept

Set up the project skeleton and prove the rendering pipeline end to end.

- Cargo workspace structure
- Bevy headless rendering
- Chromium bridge via CDP
- Single React frame to PNG output
- End-to-end pipeline validation

### Q2 2026 -- Timeline & Frame Sequencing

Implement the core animation model and video output pipeline.

- `M` facade and `m.play()` API
- Blocking animation model
- 10 core animation techniques
- FFmpeg frame-to-video pipeline
- First `moron build scene.rs` producing `.mp4`

### Q3 2026 -- Voiceover Integration

Add text-to-speech narration with audio-synced timelines.

- Kokoro TTS backend
- `m.narrate()` API
- Audio-synced timeline
- `m.beat()` / `m.breath()` timing primitives
- Audio muxing into final video
- First video with synchronized voiceover

### Q4 2026 -- Theme System & Templates

Deliver polished visual output through a theming and template layer.

- `@moron/ui` package with 10 polished templates
- Tailwind theme contract
- 20 animation techniques
- Convention-based data binding
- First "professional-looking" output

---

## Year 2: Polish & Extend (Q5--Q8 / 2027)

### Q5 2027 -- Data Visualization

Enable data-driven explainer videos with animated charts.

- Recharts / D3 integration
- `m.metric()` and `m.compare()` APIs
- Animated chart transitions
- Data-driven explainer video workflow

### Q6 2027 -- Clip Compositing & Stock

Layer video clips and AI-generated stock footage beneath React frames.

- Video clip overlay support
- `m.clip()` and `m.stock()` APIs
- AI generation hooks for stock footage
- B-roll layering beneath React frames

### Q7 2027 -- Pre-recorded Voice & Alignment

Support human voiceover recordings with automatic visual re-timing.

- Forced alignment engine
- `Voice::file()` mode for pre-recorded audio
- Human voiceover replacement
- Automatic visual re-timing to match audio

### Q8 2027 -- 30 Techniques, 20 Templates

Complete the technique and template libraries; establish the extension pattern.

- Full library of 30 animation techniques
- 20 production-ready templates
- Domain extension crate pattern (`moron-kinetic-type`, `moron-data-viz`)
- Gallery and documentation site

---

## Year 3: Stabilize & Maintain (Q9--Q12 / 2028)

### Q9 2028 -- API Freeze & Proc Macros

Lock the public API surface and add ergonomic Rust sugar.

- Stable public API
- Proc macro sugar layer: `#[scene]`, `reveal!()`, `stagger!()`
- LLM prompt engineering support for scene generation

### Q10 2028 -- 4K & Performance

Scale rendering to 4K and optimize throughput.

- 4K rendering pipeline
- Parallel frame capture
- VideoToolbox hardware encoding
- Performance profiling and optimization

### Q11 2028 -- Cross-platform Testing

Validate across hardware targets and establish CI regression testing.

- M5 Pro / Max / Ultra validation
- Intel Mac support with graceful degradation
- Linux CI support
- Headless CI pipeline for regression tests

### Q12 2028 -- 1.0 Release

Ship the stable release with complete documentation and examples.

- Stable `1.0.0` on crates.io
- Complete documentation
- Example library of 10+ full videos
- `moron-central` gallery live

---

## Quarterly Maintenance Protocol

Time budget: one weekend (2 days) per quarter for dependency maintenance.

### Every Quarter

| Dependency | Action |
|------------|--------|
| Rust toolchain | Update to latest stable. Run `cargo clippy`, fix warnings. |
| Bevy | Pin to latest minor. Follow migration guide. |
| npm packages | `npm audit`, update React / Tailwind within major version. |

### Every 6 Months

| Dependency | Action |
|------------|--------|
| FFmpeg | Update to latest stable branch. Test encoding pipeline. |
| Chromium | Update bundled / system Chrome. Test CDP screenshot capture. |
| wgpu / vello | Update within Bevy compatibility. |

### Annually

| Dependency | Action |
|------------|--------|
| Kokoro model | Check for new model releases. |
| macOS | Test on latest macOS. Verify Metal backend, VideoToolbox. |
