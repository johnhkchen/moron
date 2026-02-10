# T-004-01 Structure: Frame State Serialization

## File Changes Overview

### New Files

| File | Purpose |
|------|---------|
| `moron-core/src/frame.rs` | FrameState, ElementState, ThemeState, ElementKind, compute function |

### Modified Files

| File | Changes |
|------|---------|
| `moron-core/src/facade.rs` | Add ElementRecord, Vec<ElementRecord> to M, track metadata in mint calls |
| `moron-core/src/lib.rs` | Add `pub mod frame;` declaration and re-exports |
| `moron-techniques/src/technique.rs` | Add Serialize, Deserialize derives to TechniqueOutput |

### No Changes

| File | Reason |
|------|--------|
| `moron-core/src/timeline.rs` | Existing query API is sufficient |
| `moron-themes/src/theme.rs` | Already has Serialize/Deserialize |
| `moron-voice/` | Not involved in frame state |

---

## Detailed Structure

### moron-core/src/frame.rs (NEW)

```
Module: frame
Dependencies: serde, crate::facade, crate::timeline, std::collections::HashMap

Types:
  - ElementKind (enum, Serialize/Deserialize)
      Title
      Show
      Section
      Metric { direction: String }
      Steps { count: usize }

  - ElementState (struct, Serialize/Deserialize)
      id: u64
      kind: ElementKind
      content: String
      items: Vec<String>
      visible: bool
      opacity: f64
      translate_x: f64
      translate_y: f64
      scale: f64
      rotation: f64

  - ThemeState (struct, Serialize/Deserialize)
      name: String
      css_properties: HashMap<String, String>

  - FrameState (struct, Serialize/Deserialize)
      time: f64
      frame: u32
      total_duration: f64
      fps: u32
      elements: Vec<ElementState>
      active_narration: Option<String>
      theme: ThemeState

Functions:
  - compute_frame_state(m: &M, time: f64) -> FrameState
      Public function. Walks M's element records and timeline to produce
      the complete visual state at the given timestamp.

Tests (in #[cfg(test)] mod tests):
  - test_empty_scene_frame_state
  - test_elements_visible_after_creation
  - test_elements_not_visible_before_creation
  - test_active_narration_during_segment
  - test_no_narration_during_silence
  - test_frame_state_serializes_to_json
  - test_frame_state_json_structure
  - test_metric_element_direction
  - test_steps_element_items
```

### moron-core/src/facade.rs (MODIFIED)

```
New types:
  - ElementRecord (struct, pub(crate))
      id: u64
      kind: ElementKind  (from frame module)
      content: String
      items: Vec<String>
      created_at: f64

Modified:
  - M struct: add field `elements: Vec<ElementRecord>`
  - M::new(): initialize elements as empty Vec
  - mint_element() -> mint_element_with_meta(kind, content, items)
    Internal helper that stores metadata alongside the element handle.
  - title(): call mint_element_with_meta(Title, text, vec![])
  - show(): call mint_element_with_meta(Show, text, vec![])
  - section(): call mint_element_with_meta(Section, text, vec![])
  - metric(): call mint_element_with_meta(Metric{direction}, label + value, vec![])
  - steps(): call mint_element_with_meta(Steps{count}, joined text, items)
  - Add pub(crate) fn elements() -> &[ElementRecord] accessor
```

### moron-core/src/lib.rs (MODIFIED)

```
Add:
  - pub mod frame;
  - Re-export: FrameState, ElementState, ElementKind, ThemeState, compute_frame_state
  - Add to prelude: FrameState, compute_frame_state
```

### moron-techniques/src/technique.rs (MODIFIED)

```
Change:
  - TechniqueOutput: add #[derive(Serialize, Deserialize)] alongside existing derives
  - Add `use serde::{Serialize, Deserialize};` import
```

---

## Module Dependency Graph

```
frame.rs
  imports from: facade (M, ElementRecord, ElementKind, Direction)
  imports from: timeline (Segment)
  imports from: serde
  imports from: std::collections::HashMap

facade.rs
  imports from: frame (ElementKind)  -- circular?
```

**Circular dependency note:** frame.rs needs ElementKind, and facade.rs needs
ElementKind for ElementRecord. Solution: define ElementKind in frame.rs, and
facade.rs imports from frame. This works because both are in the same crate.

---

## Public API Surface

After changes, moron-core exports:
- `FrameState` — the complete frame snapshot
- `ElementState` — per-element visual state
- `ElementKind` — element type discriminator
- `ThemeState` — theme as CSS properties
- `compute_frame_state(m: &M, time: f64) -> FrameState` — the main computation function
