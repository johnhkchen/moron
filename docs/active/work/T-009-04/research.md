# T-009-04 Research: what-is-moron-showcase

## Current Scene State

`moron-core/src/what_is_moron.rs` contains `WhatIsMoronScene` — 6 slides using
FadeIn, FadeUp, Stagger(FadeUp+OutBack), and CountUp. Written before T-009-01
through T-009-03 landed, so the animation calls were timeline placeholders. Now
that animation execution, per-item stagger, and layout are wired, the scene
*does* animate — but the ordering and pacing produce poor visual results.

## Critical Problem: Animation-Before-Narration Ordering

The current scene creates elements then narrates then plays animations:

```rust
m.title("What is moron?");
m.narrate("What if...");          // narration segment first
m.play(FadeIn::default());       // animation segment after narration
```

Because `apply_animations()` returns `progress=0.0` before the animation segment
starts, elements are **invisible during their own narration**. The title sits at
opacity 0 while TTS reads the narration, then fades in after. Every slide has
this problem.

Fix: reorder so `m.play()` comes before `m.narrate()`, allowing the element to
animate in first, then narration plays with the element fully visible.

## Section Headers: No Animation

Slides 2–5 create section headers via `m.section()` but never animate them. They
pop in instantly at `opacity: 1.0`. The ticket draft calls for section headers to
"slide in from left".

Available: `Slide { offset_x, offset_y, duration }` in `moron_techniques::motion`.
Re-exported from `moron_techniques` as `Slide`.

## Available Techniques

| Technique | Module | What it does |
|-----------|--------|-------------|
| FadeIn | reveals | opacity 0→1 |
| FadeUp | reveals | opacity 0→1 + translate_y distance→0 |
| Slide | motion | translate from offset to 0 |
| Scale | motion | scale from→to |
| CountUp | data | opacity 0→1 (value interpolation via current_value) |
| Stagger\<T\> | staging | per-item delayed application of inner technique |

Easing combinator: `.with_ease(Ease::*)` wraps any technique.
7 curves: Linear, EaseIn, EaseOut, EaseInOut, OutBack, OutBounce, Spring.

## Layout System (T-009-03)

`assign_layout_positions()` in frame.rs:
- Headers (Title, Section) sort before bodies (Show, Steps, Metric)
- 1 element → centered at 0.5
- 2 elements → 0.3, 0.65
- 3+ elements → evenly spaced 0.2 to 0.8

Works with `clear()` — after clear, only new elements are visible, layout resets.

## Per-Item Stagger (T-009-02)

`Stagger::apply_items()` returns per-item `TechniqueOutput` vectors.
`apply_animations()` in frame.rs applies per-item transforms to Steps elements.
Items animate sequentially with configurable delay between each.

## Facade API Surface (for scene authoring)

Content: `title()`, `section()`, `show()`, `metric()`, `steps()`
Pacing: `beat()` (0.3s), `breath()` (0.8s), `wait(duration)`
Narration: `narrate(text)` — WPM-estimated duration
Animation: `play(technique)` — targets last created element
Scene: `clear()` — marks all visible elements as ended

## Duration Estimation

Current scene narrations (word count at 150 WPM):
1. "What if making professional..." — 16 words → 6.4s
2. "Creating motion graphics..." — 17 words → 6.8s
3. "moron is a code-driven..." — 19 words → 7.6s
4. "No internet required..." — 12 words → 4.8s
5. "And all of this..." — 11 words → 4.4s
6. "Motion graphics..." — 7 words → 2.8s

Total narration: ~32.8s. Plus animations (~3-4s), beats (~1.5s), breaths (~3.2s).
Estimated total: ~40s. Near upper bound of 20-40s target. Narrations may need
trimming or the target should be understood as approximate.

## Existing Tests

4 tests in what_is_moron.rs:
- `what_is_moron_builds_without_panic` — basic build check
- `what_is_moron_has_nonzero_duration` — duration > 0
- `what_is_moron_has_multiple_segments` — >= 10 segments
- `what_is_moron_clears_between_slides` — final frame has only 2 visible elements

These tests will need updating if scene structure changes (segment count, final
slide composition).

## Constraints

- `play()` always targets the most recently created element
- No way to play animation on a previously created element
- No way to play two animations simultaneously on different elements
- Each `play()` adds one timeline segment (sequential, not parallel)
