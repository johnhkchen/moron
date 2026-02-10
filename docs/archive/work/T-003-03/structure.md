# T-003-03 Structure: Implement Pacing Primitives

## Files Modified

### moron-core/src/facade.rs
- Add `use crate::timeline::{Segment, Timeline}` import
- Add pacing constants: BEAT_DURATION, BREATH_DURATION, DEFAULT_NARRATION_WPM
- Add `timeline: Timeline` field to M
- Initialize timeline in M::new()
- Implement beat(), breath(), wait()
- Implement play() (record Animation segment)
- Implement narrate() (record Narration segment with estimated duration)
- Add timeline() getter
- Update and add tests

### moron-core/src/lib.rs
- No changes needed (Timeline already in prelude)
