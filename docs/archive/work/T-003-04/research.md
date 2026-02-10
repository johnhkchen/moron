# T-003-04 Research: Integration Tests

## Objective

Write integration tests that verify the full M facade -> timeline -> technique pipeline works end-to-end.

## Test Locations (from ticket)

- `moron-core/tests/integration.rs` — M facade integration tests
- `moron-techniques/tests/composition.rs` — technique composition tests

## What's Now Available

After T-003-01/02/03, all building blocks are in place:
- M stores timeline, records all segments
- Techniques have apply() with real interpolation
- Easing functions produce correct curves
- Pacing methods (beat/breath/wait) add silence segments
- Timeline has frame_at() for frame mapping

## Test Coverage Gaps

Existing unit tests cover individual components. Integration tests should verify:
1. A realistic scene builds a valid timeline
2. Technique composition (Stagger wrapping eased FadeUp) works through facade
3. Pacing inserts correct durations in sequence
4. Frame mapping accuracy at both 30fps and 60fps
5. Multiple independent scenes produce independent timelines
6. Technique apply() produces correct output through WithEase composition
