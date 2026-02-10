# T-002-05 Progress: Wire Types Into Facade

## Completed

- [x] Step 1: Removed placeholder Technique trait, Theme struct, Voice struct from facade.rs
- [x] Step 2: Added Theme/Voice fields to M, updated constructor, setters, added getters
- [x] Step 3: Updated lib.rs re-exports, added prelude module
- [x] Step 4: Updated default_m test, added theme_and_voice_setters and play_accepts_real_techniques tests
- [x] Step 5: cargo check, cargo test (20 pass), cargo clippy (clean) — all green
- [x] Step 6: Ready for commit

## Changes Made

### moron-core/src/facade.rs
- Removed 3 placeholder types (Technique trait, Theme struct, Voice struct)
- Added `use moron_themes::Theme` and `use moron_voice::Voice`
- M struct now holds `current_theme: Theme` and `current_voice: Voice`
- M::new() initializes with Theme::default() and Voice::kokoro()
- Added `current_theme()` and `current_voice()` getters
- `theme()` and `voice()` setters now store values instead of todo!()
- `play()` uses `impl moron_techniques::Technique` (body still todo!())
- 3 new tests: default_m updated, theme_and_voice_setters, play_accepts_real_techniques

### moron-core/src/lib.rs
- Re-exports now use real types from sibling crates
- Added `pub mod prelude` with Scene, M, Element, Direction, Theme, Voice, Technique, TechniqueExt, Ease

## Deviations from Plan

None.
