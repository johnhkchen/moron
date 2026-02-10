# T-003-04 Design: Integration Tests

## Test Plan

### moron-core/tests/integration.rs (5 tests)

1. **simple_scene_produces_valid_timeline** — Build a scene with title, narrate, beat, play, show. Verify segment count, types, total duration > 0.

2. **pacing_inserts_correct_durations** — Call beat, breath, wait in sequence. Verify exact durations match constants.

3. **frame_mapping_at_30fps** — Build a 3-second timeline, verify frame counts and frame_at() values at key points.

4. **frame_mapping_at_60fps** — Same but construct timeline at 60fps (via TimelineBuilder) to verify different fps handling.

5. **multiple_scenes_independent_timelines** — Create two M instances, build different scenes, verify timelines are independent.

### moron-techniques/tests/composition.rs (3 tests)

1. **stagger_fade_up_with_easing** — Stagger(FadeUp.with_ease(OutBack)), verify apply() at progress points.

2. **eased_slide_midpoint** — Slide.with_ease(EaseIn), verify translate_x at midpoint matches eased value.

3. **technique_output_identity** — TechniqueOutput::default() is identity transform.
