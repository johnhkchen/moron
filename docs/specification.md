# moron

**Motion graphics, Obviously in Rust, Offline Natively**

Technical Specification & Maintenance Plan\
v1.1 --- February 9, 2026

Sprint-Based Build Plan | Quarterly Maintenance Cadence

## Target Hardware

- Apple M5 (32GB unified)

## Rust Edition

- 2024 (stable 1.93+)

## Key Dependencies

- Bevy 0.18.x
- React 19.2.x
- FFmpeg 8.0.x

## License

- MIT / Apache 2.0

---

## 1. Executive Summary

moron is a Rust-based motion graphics engine designed to produce professional explainer videos from LLM-authored scene files. It combines a Rust director layer (timeline, pacing, frame sequencing) with a React rendering layer (visual composition, typography, data visualization) and an integrated voiceover pipeline. A single scene file describes visuals, narration, pacing, and clips. A single command produces a finished `.mp4`.

The project targets solo maintainability over a 3-year horizon. The codebase is kept under 15,000 lines of hand-maintained Rust and React. All heavy lifting is delegated to mature, well-maintained external dependencies: Bevy for the scene graph and ECS, headless Chromium for React rendering, FFmpeg for encoding, and Kokoro/Piper for offline TTS.

This specification locks dependency versions as of February 9, 2026, defines the build sequence across 12 quarters, and establishes maintenance protocols for each quarter.

---

## 2. Design Constraints

### 2.1 Non-Negotiable Constraints

- **Air-gapped operation.** Binary takes file input, produces video output. No network, no cloud, no APIs required for core operation.
- **Solo maintainable.** One person builds and maintains indefinitely. Codebase under 15K lines.
- **LLM-first authoring.** Scene files are Rust code written by LLMs from natural language prompts. The API optimizes for LLM fluency.
- **Convention over configuration.** Sensible defaults for everything. No duration arguments, no explicit styling, no layout coordinates unless overridden.
- **Offline TTS.** Voiceover generation runs locally. API-quality TTS available as optional upgrade path.

### 2.2 Target Hardware

Apple M5 chip (announced October 15, 2025). 10-core CPU (4P + 6E), 10-core GPU with Neural Accelerators, 16-core Neural Engine. 32GB unified memory at 153.6 GB/s bandwidth. 3nm process. Metal 3 GPU backend via wgpu.

Memory budget: Chromium (~2--3 GB) + TTS model (~1--2 GB) + moron + frame buffers (~1 GB) + FFmpeg encoding (~1--2 GB) = ~6--8 GB working set, leaving 24+ GB headroom on 32GB configurations.

### 2.3 Output Specifications

| Property   | Value                                           |
| ---------- | ----------------------------------------------- |
| Resolution | 1920x1080 (default), 3840x2160 (4K)            |
| Frame rate | 30 fps (default), 60 fps (optional)             |
| Codec      | H.264 (default), H.265/HEVC (optional, via VideoToolbox) |
| Container  | MP4                                             |
| Audio      | AAC 128kbps stereo                              |

---

## 3. Architecture

### 3.1 Three-Layer Architecture

- **Director (Rust):** Timeline management, frame sequencing, narration synchronization, technique dispatch, camera control. This is moron-core.
- **Cinematographer (React):** Visual composition, typography, layout, data visualization, theming. React components render each frame's visual state via headless Chromium.
- **Voice (TTS Pipeline):** Text-to-speech generation from narration text in the scene file. Audio clips + duration metadata feed back into the timeline. Narration is the timeline's heartbeat---visuals synchronize to audio durations.

### 3.2 Rendering Pipeline

For each frame in the timeline:

1. Rust computes the visual state from the timeline position
2. State is serialized as JSON props
3. Props are sent to headless Chromium running the React component
4. React renders the frame
5. Chromium captures the frame as a pixel buffer (CDP screenshot)
6. Any video clips are composited underneath the React layer
7. Frame is piped to FFmpeg for encoding

After all frames, FFmpeg muxes the video stream with the audio track (narration + optional music) into the final `.mp4`.

### 3.3 Voiceover Pipeline

The scene file contains narration text inline. `m.narrate()` sends text to the TTS engine, receives audio + duration, and uses that duration to pace the visual timeline. `m.beat()` and `m.breath()` insert silence in both the audio track and visual pacing.

TTS backends (pluggable via Voice config):

- **Offline:** Kokoro (82M params, Apache 2.0, runs on CPU at ~96x real-time). Primary recommendation. ONNX runtime via kokorox Rust crate.
- **Offline fallback:** Piper TTS via piper-rs. Lighter weight, broader language support, slightly lower quality.
- **API upgrade:** OpenAI TTS, ElevenLabs, or equivalent. Scene file unchanged---only Voice config changes.
- **Pre-recorded:** Human voiceover `.wav` files aligned to transcript via forced alignment. Visuals re-time automatically.

### 3.4 Scene File as Complete Production Script

A single `.rs` file describes the entire video: visuals, narration, pacing, clips, theme. The Rust type system enforces valid compositions at compile time. An LLM writes the scene from a natural language prompt; `cargo build` validates it; `moron build` renders it.

---

## 4. Dependency Manifest

All versions pinned as of February 9, 2026. Exact minor/patch versions will advance within semver bounds during each maintenance quarter.

### 4.1 Rust Ecosystem

| Dependency     | Version     | License      | Notes                                                      |
| -------------- | ----------- | ------------ | ---------------------------------------------------------- |
| Rust           | 1.93 stable | MIT/Apache   | Edition 2024 (stabilized in 1.85, Feb 2025)                |
| Bevy           | 0.18.0      | MIT/Apache   | Released Jan 13, 2026. 0.19 in dev on main.                |
| wgpu           | 28.0.0      | MIT/Apache   | Metal backend for M5. MSRV 1.92.                           |
| vello          | 0.6.0       | MIT/Apache   | GPU compute 2D renderer. Oct 2025 release.                 |
| bevy_vello     | latest      | MIT/Apache   | Vello as Bevy plugin for vector graphics.                  |
| chromiumoxide  | 0.7.0       | MIT/Apache   | Async headless Chrome via CDP. Tokio.                      |
| kokorox        | latest      | MIT/Apache   | Kokoro TTS in Rust. 82M param ONNX model.                 |
| piper-rs       | latest      | MIT          | Piper TTS fallback. espeak-ng phonemizer.                  |
| serde / serde_json | 1.x     | MIT/Apache   | JSON serialization for frame state props.                  |
| tokio          | 1.x         | MIT          | Async runtime for Chromium bridge.                         |
| anyhow / thiserror | 1.x / 2.x | MIT/Apache | Error handling.                                            |
| clap           | 4.x         | MIT/Apache   | CLI argument parsing.                                      |

### 4.2 JavaScript / React Ecosystem

| Dependency        | Version | License | Notes                                          |
| ----------------- | ------- | ------- | ---------------------------------------------- |
| React             | 19.2.4  | MIT     | Latest stable. Jan 26, 2026 patch release.     |
| React DOM         | 19.2.4  | MIT     | Server Components stable in 19.x.              |
| Tailwind CSS      | 4.x     | MIT     | Theme system. CSS custom properties.            |
| Recharts          | 2.x     | MIT     | Data visualization components.                  |
| D3                | 7.x     | ISC     | Low-level visualization when needed.            |
| react-three-fiber | 8.x     | MIT     | 3D scenes within React frames.                  |
| shadcn/ui         | latest  | MIT     | Polished UI components.                         |
| Framer Motion     | 11.x   | MIT     | CSS animation coexistence layer.                |

### 4.3 System Dependencies

| Dependency   | Version | License      | Notes                                                        |
| ------------ | ------- | ------------ | ------------------------------------------------------------ |
| FFmpeg       | 8.0.1   | LGPL/GPL     | Released Nov 2025. H.264/H.265, VideoToolbox HW accel.      |
| Chromium     | 131+    | BSD          | Headless mode. Bundled or system-installed.                  |
| macOS        | 26.x (Tahoe) | Proprietary | Metal 3 GPU, VideoToolbox, Apple Silicon.               |
| Node.js      | 22 LTS  | MIT          | React SSR for frame rendering.                               |
| Kokoro model | v1.0    | Apache 2.0   | 82M params. kokoro-v1.0.onnx (~330MB).                      |

---

## 5. Repository Structure

Cargo workspace with npm packages co-located:

- **moron-core/** --- Scene graph, timeline, camera, Bevy integration, Chromium bridge, FFmpeg pipeline. (5--8K lines)
- **moron-techniques/** --- ~30 composable animation techniques, each 20--50 lines. (1--2K lines)
- **moron-voice/** --- TTS abstraction, Kokoro/Piper backends, forced alignment, audio timeline. (1--2K lines)
- **moron-themes/** --- Default theme, CSS custom properties, Tailwind config contract.
- **moron-macros/** --- Proc macro sugar (future, after API stabilizes).
- **moron-cli/** --- Binary wrapper. `moron build`, `moron preview`.
- **packages/ui/** --- @moron/ui npm package. Base React components, templates, data-moron conventions.
- **packages/themes/** --- @moron/themes npm package. Visual identities as CSS + Tailwind configs.
- **examples/** --- Scene files. Each is a complete, runnable video.
- **gallery/** --- moron-central. Technique documentation rendered by moron itself.

Total hand-maintained code: ~10--15K lines Rust + ~3--5K lines React/TypeScript.

---

## 6. API Surface

### 6.1 Scene File Structure

Scene files are Rust code implementing the Scene trait. The `M` facade struct hides all internal machinery (Bevy ECS, renderer, timeline, FFmpeg, TTS).

Key `M` methods:

- `m.narrate(text)` --- Send text to TTS, get audio + duration, pace visuals to match.
- `m.narrate_over(element, text)` --- Narrate while element is on screen.
- `m.show(text)` --- Context-aware text display.
- `m.title(text)`, `m.section(text)` --- Structural narrative elements.
- `m.diagram(name, props)` --- Maps to named React component.
- `m.metric(label, value, direction)` --- Impactful stat display.
- `m.compare(before, after, chart)` --- Side-by-side comparison.
- `m.steps([...])` --- Staggered sequential reveal.
- `m.focus(element, annotation)` --- Highlight with annotation.
- `m.clip(path)` / `m.stock(description)` --- Video clips / AI-generated B-roll.
- `m.beat()` / `m.breath()` / `m.wait(duration)` --- Pacing primitives.
- `m.play(technique)` --- Execute animation technique (blocking).
- `m.theme(Theme)` / `m.voice(Voice)` --- Configuration.

### 6.2 Technique Library (~30 techniques)

Techniques are Rust structs implementing traits. Composable via combinators:

- **Reveals:** FadeIn, FadeUp, Wipe, MaskReveal, DrawOn
- **Motion:** Slide, Scale, Overshoot, Spring, PathFollow
- **Morphing:** ShapeInterp, LayoutTransition
- **Staging:** Stagger, Cascade, Sequential, Parallel
- **Emphasis:** Pulse, Shake, Glow, ColorShift
- **Camera:** Pan, Zoom, Parallax
- **Transitions:** Crossfade, Iris, Push, MorphCut
- **Data:** BarGrow, LineDraw, CountUp

Composition example:

```rust
Stagger(FadeUp.with_ease(Ease::OutBack))
```

### 6.3 React Component Convention

Templates use `data-moron` attributes for technique binding:

- `data-moron="container"` --- Root layout wrapper.
- `data-moron="title"` --- Title text target.
- `data-moron="sequence"` --- Parent of staggerable items.
- `data-moron="item"` --- Individual item within sequence.

Techniques discover elements by convention. No explicit wiring. Swap the theme file, the entire visual identity changes.

---

## 7. Build Roadmap

See [ROADMAP.md](ROADMAP.md) for the sprint-based milestone plan. Work is organized into sprints (stories), not calendar quarters. Each sprint ships a meaningful capability with no artificial time gates.

---

## 8. Maintenance Protocol

Time budget: one weekend (2 days) per quarter for dependency maintenance; ongoing creative time for technique acquisition.

### 8.1 Dependency Update Cadence

| Cadence        | Dependency       | Action                                                                      |
| -------------- | ---------------- | --------------------------------------------------------------------------- |
| Every quarter  | Rust toolchain   | Update to latest stable. Run `cargo clippy`, fix warnings. Edition 2024 stays. |
| Every quarter  | Bevy             | Pin to latest minor. Bevy releases ~every 3 months. Follow migration guide. |
| Every quarter  | npm packages     | `npm audit`, update React/Tailwind within major. Test frame rendering.      |
| Every 6 months | FFmpeg           | Update to latest stable branch. Test encoding pipeline.                     |
| Every 6 months | Chromium         | Update bundled/system Chrome. Test CDP screenshot capture.                  |
| Every 6 months | wgpu / vello     | Update within Bevy compatibility. Vello tracks Bevy releases.               |
| Annually       | Kokoro model     | Check for new model releases. Update ONNX file if improved.                |
| Annually       | macOS            | Test on latest macOS. Verify Metal backend, VideoToolbox.                   |

### 8.2 Bevy Migration Strategy

Bevy releases breaking changes every ~3 months. This is the highest-maintenance dependency. Mitigation:

- **Thin Bevy surface.** moron-core uses Bevy for scene graph, headless rendering, and asset loading only. No gameplay systems, no UI, no input handling. Migration surface is small.
- **Isolation layer.** All Bevy types are behind the `M` facade. Scene files never touch Bevy directly. A Bevy migration only touches moron-core internals.
- **Migration budget.** One day per Bevy release. Follow the official migration guide. Run the test suite. Bevy's guides are comprehensive and the breaking changes are well-documented.
- **Skip policy.** If a Bevy release introduces no features moron needs, it is acceptable to skip one release and update on the next. Never fall more than two releases behind.

### 8.3 Technique Acquisition

Ongoing creative work, not tied to maintenance cadence. Watch professional videos (Kurzgesagt, Vox, corporate explainers), extract techniques, implement as 20--50 line Rust functions. Target: 1--2 new techniques per month initially, then as-needed.

### 8.4 Risk Register

| Risk                        | Likelihood | Impact | Mitigation                                                               |
| --------------------------- | ---------- | ------ | ------------------------------------------------------------------------ |
| Bevy API churn              | High       | Medium | Thin surface area, isolation layer, skip policy.                         |
| Chromium CDP changes        | Low        | High   | chromiumoxide abstracts CDP. Screenshot API is stable.                   |
| Kokoro model abandoned      | Low        | Medium | Piper fallback. TTS backend is pluggable trait.                          |
| Vello stalls in alpha       | Medium     | Low    | Only used for optional vector overlays. Not on critical path.            |
| React major version         | Low        | Medium | React 19 is current. Major bumps are rare. Templates are simple.         |
| FFmpeg breaking changes     | Very Low   | Low    | FFmpeg CLI is extremely stable. Encoding flags rarely change.            |
| Apple Silicon arch change   | Very Low   | High   | wgpu abstracts GPU. Rust compiles to ARM natively.                       |
| LLM scene quality           | Medium     | Medium | Strong types + compiler = invalid scenes don't compile. Iterate on prompts. |

---

## 9. Maintenance Boundary

### 9.1 What You Maintain (~15K lines)

- **moron-core:** Timeline, frame sequencer, Chromium bridge, FFmpeg pipeline, M facade (5--8K lines)
- **moron-techniques:** ~30 techniques at 20--50 lines each (1--2K lines)
- **moron-voice:** TTS abstraction, Kokoro/Piper backends (1--2K lines)
- **@moron/ui:** 15--20 React templates at 50--80 lines each (1--2K lines)
- **@moron/themes:** CSS + Tailwind theme configs (~500 lines)
- **moron-cli:** Binary wrapper (~500 lines)

### 9.2 What You Do Not Maintain

- Rendering engine (Chromium --- Google)
- Component library (React ecosystem --- Meta)
- Design system (Tailwind --- Tailwind Labs)
- Video encoding (FFmpeg --- FFmpeg project)
- GPU abstraction (wgpu --- gfx-rs)
- 2D renderer (Vello --- Linebender)
- Game engine / ECS (Bevy --- Bevy Foundation)
- TTS model (Kokoro --- hexgrad)
- Chart libraries (Recharts, D3 --- respective maintainers)
- Text shaping (cosmic-text, parley --- Linebender)

> **Thesis: The machinery exists. The taste doesn't. Package the taste.**

---

## 10. CLI Interface & Distribution

### 10.1 Commands

```
moron build scene.rs -o output.mp4       # Full render
moron build scene.rs --preview            # Low-res fast render for iteration
moron build scene.rs --frame 120          # Render single frame for debugging
moron build scene.rs --no-voice           # Visual only, no TTS
moron init my-video                       # Scaffold new project with template
moron gallery                             # Open technique gallery in browser
```

### 10.2 Distribution

- `cargo install moron` --- installs the binary
- System prerequisites: Chromium, FFmpeg, Node.js (detected automatically, errors with clear instructions if missing)
- Kokoro model downloaded on first use (~330 MB, cached locally)
- Or: as a library dependency for LLM coding agents. Agent writes scene file, builds it, returns `.mp4`.

### 10.3 Success Criteria

- LLM can generate professional output from a single natural language prompt
- Output quality indistinguishable from After Effects for common explainer use cases
- Codebase under 15K lines of hand-maintained code
- Single `cargo install`, works offline after initial setup
- Renders 1080p at 30fps faster than real-time on M5
- Solo maintainable indefinitely on the quarterly cadence
