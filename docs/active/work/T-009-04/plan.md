# T-009-04 Plan: what-is-moron-showcase

## Step 1: Rewrite scene build() method

Replace the body of `WhatIsMoronScene::build()` in `what_is_moron.rs`.

New structure per slide:
1. **Cold Open**: title → FadeIn(0.8s) → beat → narrate → breath
2. **The Problem**: clear → section → Slide(EaseOut) → narrate → show → FadeUp → breath
3. **The Solution**: clear → section → Slide(EaseOut) → narrate → steps(3) → Stagger(FadeUp+OutBack) → breath
4. **Key Features**: clear → section → Slide(EaseOut) → narrate → steps(3) → Stagger(FadeUp+OutBack) → breath
5. **The Metric**: clear → section → Slide(EaseOut) → narrate → metric → CountUp → beat
6. **Closing**: clear → title → FadeIn(0.8s) → show → FadeIn(0.6s) → narrate → beat

Narration text (trimmed):
1. "What if making explainer videos was as simple as writing Rust?"
2. "Motion graphics today means complex tools, expensive licenses, and manual labor."
3. "Write a scene. Run one command. Get a video."
4. "No internet. No cloud. Everything runs on your machine."
5. "All of this, in under fifteen thousand lines of code."
6. "moron. Motion graphics, obviously in Rust, offline natively."

Update imports to include `Slide` and `Scale`.

Verify: `cargo check`

## Step 2: Update tests

Adjust test assertions in the `tests` module:
- `what_is_moron_has_multiple_segments`: verify segment count is >= expected
  (count new segments: ~20+ across all slides)
- `what_is_moron_clears_between_slides`: keep checking for 2 visible elements
  at end (title + show in closing slide)
- Other tests should pass unchanged

Verify: `cargo test -p moron-core --lib`

## Step 3: Validate animation behavior with frame state checks

Add 2-3 new test assertions to verify animations actually execute:
- At animation midpoint, element opacity should be between 0 and 1
- After all animations complete, elements should be fully visible
- Stagger items should have different opacities at midpoint

Verify: `cargo test -p moron-core --lib`

## Step 4: Full test suite verification

Run complete test suite including e2e tests:
- `cargo test` (all workspace tests)
- `cargo clippy` (lint clean)

Verify all ~170+ tests pass, clippy clean.

## Testing Strategy

- Unit tests in `what_is_moron.rs::tests` verify scene structure
- Frame state tests verify animation execution produces expected visual state
- No manual visual verification needed — the animation/layout systems are
  already tested in T-009-01 through T-009-03
- E2e tests use DemoScene (not affected by this change)
