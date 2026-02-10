# T-003-03 Research: Implement Pacing Primitives

## Objective

Implement beat(), breath(), wait() in M facade. These add Silence segments to the timeline.

## Current State

- M has no Timeline field — pacing methods are `todo!()`
- Timeline and Segment exist in moron-core/src/timeline.rs (T-003-01 complete)
- beat/breath/wait/play/narrate all have `todo!()` bodies

## Requirements (from ticket)

- `m.beat()` → ~0.3s silence
- `m.breath()` → ~0.8s silence
- `m.wait(d)` → d seconds silence
- Default durations should be constants

## Additional Opportunity

Since we're adding Timeline to M, we can also wire up `play()` to record Animation segments and `narrate()` to record Narration segments. This naturally belongs here since the timeline is the recording mechanism for all facade methods.

## Key Insight

M needs a `timeline: Timeline` field. The pacing methods call `self.timeline.add_segment(Segment::Silence { ... })`. Play adds Animation segments. Narrate adds Narration segments (duration placeholder for now since TTS duration estimation is future work).
