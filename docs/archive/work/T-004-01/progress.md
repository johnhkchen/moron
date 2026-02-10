# T-004-01 Progress: Frame State Serialization

## Status: COMPLETE

All implementation steps have been executed and verified.

---

## Steps Completed

### Step 1: Add Serialize/Deserialize to TechniqueOutput
- **File:** `moron-techniques/src/technique.rs`
- Added `use serde::{Deserialize, Serialize};`
- Added `#[derive(Serialize, Deserialize)]` and `#[serde(rename_all = "camelCase")]` to TechniqueOutput
- **Verified:** `cargo check -p moron-techniques` passes

### Step 2: Create frame.rs with type definitions
- **File:** `moron-core/src/frame.rs` (new, ~280 lines)
- Defined `ElementKind` enum (Title, Show, Section, Metric, Steps) with serde tagged union
- Defined `ElementState` struct with all visual properties (opacity, translate, scale, rotation)
- Defined `ThemeState` struct with name and CSS properties HashMap
- Defined `FrameState` struct as the complete frame snapshot
- All types derive `Serialize, Deserialize` with `rename_all = "camelCase"`

### Step 3: Add element metadata tracking to facade.rs
- **File:** `moron-core/src/facade.rs`
- Added `ElementRecord` struct (pub(crate)) with id, kind, content, items, created_at
- Added `elements: Vec<ElementRecord>` field to M struct
- Replaced `mint_element()` with `mint_element_with_meta(kind, content, items)`
- Updated all five content methods: title(), show(), section(), metric(), steps()
- Added `pub(crate) fn elements()` accessor
- All existing tests continue to pass

### Step 4: Implement compute_frame_state()
- **File:** `moron-core/src/frame.rs`
- Implemented `compute_frame_state(m: &M, time: f64) -> FrameState`
- Algorithm: clamp time, compute frame, walk elements for visibility, find active narration, build theme state
- Element visibility based on `created_at <= clamped_time`
- Narration lookup via `segments_in_range` with epsilon window
- Helper function `find_active_narration()` extracted

### Step 5: Update lib.rs with module and re-exports
- **File:** `moron-core/src/lib.rs`
- Added `pub mod frame;`
- Re-exported: `FrameState, ElementState, ElementKind, ThemeState, compute_frame_state`
- Added all to prelude

### Step 6: Write unit tests
- **File:** `moron-core/src/frame.rs` (14 tests)
- `empty_scene_frame_state` — empty M produces valid empty FrameState
- `elements_visible_at_creation_time` — elements at t=0 are visible at t=0
- `elements_not_visible_before_creation` — elements after narration not visible at t=0
- `active_narration_during_segment` — narration text active during segment
- `no_narration_during_silence` — no narration during silence
- `frame_state_serializes_to_json` — JSON serialization succeeds
- `frame_state_json_has_camel_case_keys` — camelCase in output
- `frame_state_json_round_trip` — serialize/deserialize round trip
- `metric_element_preserves_direction` — direction stored correctly
- `steps_element_preserves_items` — items list preserved
- `frame_number_computed_correctly` — frame math is right
- `time_clamped_to_valid_range` — negative and overflow times clamped
- `theme_state_contains_css_properties` — theme CSS properties present
- `section_element_kind` — section kind correct
- `multiple_elements_with_timing` — progressive visibility over time

### Step 7: Full validation
- `cargo check` — passes (zero errors)
- `cargo test` — 68 tests pass (14 new + 54 existing), zero failures
- `cargo clippy` — passes (zero warnings)

---

## Deviations from Plan

None. All steps executed as planned.

---

## Files Changed

| File | Action | Lines |
|------|--------|-------|
| `moron-techniques/src/technique.rs` | Modified | +3 lines (import + derives) |
| `moron-core/src/frame.rs` | Created | ~280 lines |
| `moron-core/src/facade.rs` | Modified | ~40 lines added |
| `moron-core/src/lib.rs` | Modified | +4 lines |
