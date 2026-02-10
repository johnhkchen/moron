# T-004-01 Plan: Frame State Serialization

## Implementation Steps

### Step 1: Add Serialize/Deserialize to TechniqueOutput

**File:** `moron-techniques/src/technique.rs`
**Changes:**
- Add `use serde::{Serialize, Deserialize};`
- Add `#[derive(Serialize, Deserialize)]` to TechniqueOutput

**Verify:** `cargo check -p moron-techniques`

---

### Step 2: Create frame.rs with type definitions

**File:** `moron-core/src/frame.rs` (new)
**Changes:**
- Define `ElementKind` enum with Serialize/Deserialize, camelCase rename
- Define `ElementState` struct with Serialize/Deserialize, camelCase rename
- Define `ThemeState` struct with Serialize/Deserialize, camelCase rename
- Define `FrameState` struct with Serialize/Deserialize, camelCase rename
- Stub `compute_frame_state()` that returns an empty FrameState

**Verify:** `cargo check -p moron-core`

---

### Step 3: Add element metadata tracking to facade.rs

**File:** `moron-core/src/facade.rs`
**Changes:**
- Import `ElementKind` from frame module
- Define `ElementRecord` struct (pub(crate))
- Add `elements: Vec<ElementRecord>` field to M
- Update `M::new()` to initialize empty elements vec
- Replace `mint_element()` with `mint_element_with_meta(kind, content, items)`
- Update title(), show(), section(), metric(), steps() to pass metadata
- Add `pub(crate) fn elements(&self) -> &[ElementRecord]` accessor

**Verify:** `cargo check -p moron-core` (existing tests must still pass)

---

### Step 4: Implement compute_frame_state()

**File:** `moron-core/src/frame.rs`
**Changes:**
- Implement the full computation logic:
  1. Clamp time to valid range
  2. Compute frame number via timeline
  3. Walk element records, determine visibility (created_at <= time)
  4. Set default visual properties for visible elements
  5. Find active narration from overlapping segments
  6. Build ThemeState from current theme
  7. Assemble and return FrameState

**Verify:** `cargo check -p moron-core`

---

### Step 5: Update lib.rs with module and re-exports

**File:** `moron-core/src/lib.rs`
**Changes:**
- Add `pub mod frame;`
- Add re-exports: `FrameState, ElementState, ElementKind, ThemeState, compute_frame_state`
- Add to prelude: `FrameState, compute_frame_state`

**Verify:** `cargo check -p moron-core`

---

### Step 6: Write unit tests

**File:** `moron-core/src/frame.rs`
**Tests:**
1. Empty scene produces FrameState with no elements, no narration
2. Elements created at t=0 are visible at t=0
3. Elements created after narration are not visible at t=0
4. Active narration text appears during narration segment
5. No narration during silence segments
6. FrameState serializes to valid JSON
7. JSON contains expected camelCase keys
8. Metric element preserves direction
9. Steps element preserves items
10. Frame number computed correctly

**Verify:** `cargo test -p moron-core`

---

### Step 7: Run full validation

- `cargo check` (entire workspace)
- `cargo test` (entire workspace)
- `cargo clippy` (entire workspace)

Fix any issues found.

---

## Testing Strategy

- **Unit tests in frame.rs:** Core logic — visibility, narration, serialization.
- **Existing tests in facade.rs:** Must continue to pass. Element minting behavior
  is unchanged from the caller's perspective; we only add internal metadata tracking.
- **Serialization round-trip:** Verify JSON structure matches expected React contract.
- **Edge cases:** t=0, t=total_duration, t beyond bounds, empty scene.

---

## Risk Mitigation

- Adding derives to TechniqueOutput is backward-compatible (additive).
- ElementRecord is pub(crate) — no public API breakage.
- FrameState is entirely additive — new module, new types, new function.
- Existing facade tests verify backward compatibility of M's public API.
