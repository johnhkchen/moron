# T-003-03 Progress: Implement Pacing Primitives

## Completed

- [x] Added Timeline field to M, initialized in new()
- [x] Implemented beat() (0.3s), breath() (0.8s), wait(d) as Silence segments
- [x] Implemented play() as Animation segments on timeline
- [x] Implemented narrate() with word-count duration estimation
- [x] Added timeline() getter
- [x] 6 new tests (21 total in moron-core), all pass
- [x] cargo clippy clean, 45 tests total across workspace

## Deviations from Plan

Also implemented play() and narrate() since they require the same Timeline wiring.
