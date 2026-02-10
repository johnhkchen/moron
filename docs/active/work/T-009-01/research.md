# T-009-01 Research: Animation Execution Engine

## The Gap

Three things are missing between "technique exists" and "technique output reaches the rendered frame":

### 1. Technique objects are discarded at `play()` time

`M::play()` in `facade.rs:255-260` consumes the technique, extracts `name` and `duration`, and records `Segment::Animation { name, duration }` on the timeline. The technique object itself — and its `apply()` method — is dropped immediately.

```rust
pub fn play(&mut self, technique: impl moron_techniques::Technique) {
    self.timeline.add_segment(Segment::Animation {
        name: technique.name().to_string(),
        duration: technique.duration(),
    });
}
```

### 2. No element-to-animation binding

When a scene author writes `m.show("text"); m.play(FadeIn::default())`, the implicit convention is that the FadeIn applies to the most recently created element. But nothing records this association. The `Segment::Animation` has no element IDs, and `ElementRecord` has no animation references.

### 3. `compute_frame_state()` ignores animations

In `frame.rs:128-148`, every visible element gets hardcoded defaults:

```rust
opacity: if visible { 1.0 } else { 0.0 },
translate_x: 0.0,
translate_y: 0.0,
scale: if visible { 1.0 } else { 0.0 },
rotation: 0.0,
```

There's no code that looks at `Segment::Animation` segments or calls any technique's `apply()` method.

## Existing Data Flow

```
Scene::build(&mut M)
  → m.title("X")     → mints Element(0), pushes ElementRecord { created_at, kind, ... }
  → m.play(FadeIn)   → pushes Segment::Animation { name: "FadeIn", duration: 0.5 }
  → m.narrate("...")  → pushes Segment::Narration { text, duration }

compute_frame_state(&m, time)
  → walks m.elements() to build Vec<ElementState>
  → walks m.timeline().segments_in_range() for active narration
  → returns FrameState { elements, active_narration, theme, ... }
```

The `elements` loop and the `segments_in_range` loop are entirely independent — animations on the timeline never influence element visual state.

## Key Types and Their Locations

| Type | File | Role |
|------|------|------|
| `Technique` trait | `moron-techniques/src/technique.rs:108-117` | `name()`, `duration()`, `apply(progress) -> TechniqueOutput` |
| `TechniqueOutput` | `moron-techniques/src/technique.rs:75-86` | opacity, translate_x/y, scale, rotation |
| `ElementState` | `moron-core/src/frame.rs:44-67` | Same five visual fields + id, kind, content, items, visible |
| `ElementRecord` | `moron-core/src/facade.rs:39-60` | id, kind, content, items, created_at, ended_at, segments_at_* |
| `Segment::Animation` | `moron-core/src/timeline.rs:19` | `{ name: String, duration: f64 }` — no technique object |
| `M` struct | `moron-core/src/facade.rs:113-124` | next_element_id, theme, voice, timeline, elements |
| `Timeline` | `moron-core/src/timeline.rs:47-50` | `segments: Vec<Segment>`, `fps: u32` |

## Object Safety of Technique Trait

The `Technique` trait is object-safe: all three methods take `&self` and return sized types. `Box<dyn Technique>` will work. The `TechniqueExt` trait (with `with_ease`) is NOT object-safe (has `Sized` bound), but that's fine — it's a build-time combinator, not needed at query time.

## Concrete Techniques

Six concrete types implement `Technique`:

| Technique | Module | Visual Effect |
|-----------|--------|---------------|
| `FadeIn` | reveals.rs | opacity: 0→1 |
| `FadeUp` | reveals.rs | opacity: 0→1 + translate_y: distance→0 |
| `Slide` | motion.rs | translate_x/y: offset→0 |
| `Scale` | motion.rs | scale: from→to |
| `CountUp` | data.rs | opacity: 0→1 (value exposed separately) |
| `Stagger<T>` | staging.rs | wraps inner technique with per-item delays |
| `WithEase<T>` | technique.rs | wraps inner technique with easing curve |

All produce `TechniqueOutput`. Stagger has special `apply_item(index, progress)` for per-item state (T-009-02 scope).

## `play()` Usage Patterns in Existing Scenes

`demo.rs`:
```rust
m.title("moron Demo");
m.narrate("...");
m.play(FadeIn::default());   // applies to "moron Demo" title? Ambiguous.
m.beat();
m.section("Pipeline");
m.narrate("...");
m.play(FadeUp::default());   // applies to "Pipeline" section? Ambiguous.
```

Convention: `play()` is called after `narrate()`, but the animation is meant for the most recently created *element* (title/show/section), not the narration itself. The most recently created element is tracked by `elements.last()`.

## Timeline Segment Ordering

Segments accumulate linearly. The segment index of the animation tells us its timeline position:

```
[Narration(0.8), Animation(0.5), Silence(0.3)]
 starts: 0.0      starts: 0.8      starts: 1.3
```

The animation's time window is `[segment_start, segment_start + duration)`. Progress within that window is `(time - segment_start) / duration`, clamped to [0, 1].

## Duration Resolution Impact

`resolve_narration_durations()` can change narration segment durations after scene build. This already recomputes element `created_at`/`ended_at` by walking `segments_at_creation`. Animation records would need to store their segment index (not absolute start time) so their time window can be recomputed the same way.

## Cross-Crate Boundary

`M` is in `moron-core`, `Technique` is in `moron-techniques`. Currently `play()` takes `impl Technique` (generic, monomorphized). Storing `Box<dyn Technique>` means `moron-core` needs to depend on `moron-techniques` for the trait — which it already does (see `moron-core/Cargo.toml` and `lib.rs` re-exports). No new dependency needed.

## Existing Test Surface

- `facade::tests::play_records_animation_segments` — verifies segment count and duration
- `frame::tests::*` — ~15 tests verifying element visibility, narration, theme, serialization
- `e2e.rs` — 4 non-ignored tests exercising full pipeline with DemoScene
- None of these tests assert on animation visual state (opacity/translation/scale during animation segments)

## Constraints

- `ElementRecord` is `pub(crate)` — can add fields freely without breaking external API
- `M`'s fields are all private — adding a new `Vec<AnimationRecord>` is internal
- `Segment::Animation` is `pub` in `timeline.rs` — changing its fields would affect `TimelineBuilder` and tests
- `compute_frame_state()` takes `&M` (immutable borrow) — animation lookup must be read-only
- Stagger per-item animation is explicitly T-009-02 scope; this ticket handles whole-element animation only
