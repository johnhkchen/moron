# T-009-04 Structure: what-is-moron-showcase

## Files Modified

### 1. `moron-core/src/what_is_moron.rs` (rewrite)

The only file with substantive changes. Entire `WhatIsMoronScene::build()` body
replaced. Module structure, struct, trait impl, and test module remain.

**Imports**: Add `Slide`, `Scale` to the existing import from `moron_techniques`.

**Scene structure** (6 slides, same as current):

```
Slide 1: title → play(FadeIn) → beat → narrate → breath
Slide 2: clear → section → play(Slide) → narrate → show → play(FadeUp) → breath
Slide 3: clear → section → play(Slide) → narrate → steps → play(Stagger) → breath
Slide 4: clear → section → play(Slide) → narrate → steps → play(Stagger) → breath
Slide 5: clear → section → play(Slide) → narrate → metric → play(CountUp) → beat
Slide 6: clear → title → play(FadeIn) → show → play(FadeIn) → narrate → beat
```

**Narration text**: Trimmed for pacing. Each narration 1-2 short sentences.
Target 4-8 words per narration → 1.6-3.2s each at 150 WPM. Total narration
target: ~14-18s. With animations + pacing: 25-35s total.

**Tests section**: Same 4 tests, adjusted assertions:
- `what_is_moron_has_multiple_segments`: may need count update
- `what_is_moron_clears_between_slides`: same check (2 elements at end)

## Files NOT Modified

- `facade.rs` — no API changes needed
- `frame.rs` — animation execution and layout already work
- `moron-techniques/` — all needed techniques already exist
- `packages/ui/` — React renderer already handles all element kinds/transforms
- `moron-cli/src/main.rs` — already supports `--scene what-is-moron`
- `moron-core/tests/e2e.rs` — e2e tests use DemoScene, not WhatIsMoronScene

## No New Files

This is a content change to a single existing file. No new modules, types, or
exports needed.

## Public Interface

No changes. `WhatIsMoronScene` struct and `Scene` impl remain identical.
Re-exports in `lib.rs` and `prelude` module unchanged.
