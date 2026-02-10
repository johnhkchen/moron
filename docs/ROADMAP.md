# Moron Roadmap

> Sprint-based milestones for the Moron motion graphics engine.

Work is organized into sprints (stories), not calendar quarters. Each sprint ships a meaningful capability. No artificial time gates — when a sprint is done, the next one starts.

**Current status:** Sprints 5 and 6 active (S-006: TTS Integration, S-007: Templates & Polish)

---

## Completed Sprints

### Sprint 1: Core Type System (S-002) ✓

Defined the foundational Rust types across all crates.

- Scene trait and M facade with method signatures
- Technique trait with easing curves and composability
- Voice/TTS backend trait with audio clip types
- Theme struct with CSS property contract
- All types wired through the M facade with prelude module

### Sprint 2: Timeline & Animation (S-003) ✓

Implemented the timeline and core animation techniques.

- Timeline with ordered segments (Narration, Animation, Silence, Clip)
- TimelineBuilder fluent API, frame_at() mapping, segments_in_range()
- 6 techniques with real interpolation: FadeIn, FadeUp, Slide, Scale, CountUp, Stagger
- 7 easing curves: Linear, EaseIn, EaseOut, EaseInOut, OutBack, OutBounce, Spring
- Pacing primitives: beat (0.3s), breath (0.8s), wait(d)
- M facade recording all methods to timeline
- 53 tests passing, clippy clean

### Sprint 3: Frame Rendering Pipeline (S-004) ✓

Bridged Rust timeline state to visual output via React and headless Chromium.

- Frame state serialization (timeline position → JSON props)
- React `<MoronFrame>` base component consuming frame state
- Headless Chromium bridge via CDP for screenshot capture
- Frame rendering loop: iterate timeline at FPS, output numbered PNGs

### Sprint 4: Video Output Pipeline (S-005) ✓

Turned rendered frames into a finished .mp4 video.

- FFmpeg integration for H.264 encoding
- Audio track assembly from timeline segments
- `moron build` CLI command wiring the full pipeline
- End-to-end validation: scene → .mp4

---

## Active Sprints

### Sprint 5: TTS Integration (S-006)

Wire actual text-to-speech into the narration pipeline.

- Kokoro TTS backend via kokorox crate
- Audio-synced timeline (TTS durations drive visual pacing)
- Pipeline integration (real audio in final .mp4)
- TTS validation tests

### Sprint 6: Templates & Polish (S-007)

Deliver polished visual output through themed React templates.

- Template system architecture and host page bundling
- Default explainer template with title cards, sections, metrics, steps
- Theme CSS integration end-to-end
- Visual regression testing

---

## Future Sprints

### Sprint 7: Data Visualization & Clips

Enable data-driven explainer videos and clip compositing.

- Recharts/D3 integration for animated charts
- m.metric(), m.compare() with real rendering
- Video clip overlay support (m.clip())
- B-roll layering beneath React frames

### Sprint 8: Pre-recorded Voice & Alignment

Support human voiceover with automatic visual re-timing.

- Forced alignment engine
- Voice::file() mode for pre-recorded audio
- Automatic visual re-timing to match audio durations

### Sprint 9: Performance & Scale

Optimize rendering throughput and support higher resolutions.

- 4K rendering pipeline
- Parallel frame capture
- VideoToolbox hardware encoding
- Performance profiling

### Sprint 10: API Stabilization & 1.0

Lock the public API and ship.

- Stable public API surface
- Proc macro sugar (#[scene], reveal!(), stagger!())
- Cross-platform CI (Linux, Intel Mac graceful degradation)
- Complete documentation and example library
- 1.0.0 release

---

## Ongoing Maintenance

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
