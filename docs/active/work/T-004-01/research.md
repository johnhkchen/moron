# T-004-01 Research: Frame State Serialization

## Objective

Map the codebase to understand what exists, where, and how it connects — specifically
regarding computing a complete visual state at a given timestamp.

---

## Current Element System

### Element Handle (facade.rs)

`Element` is an opaque `u64` wrapper: `pub struct Element(pub(crate) u64)`.
Minted by `M::mint_element()` via a monotonic counter `next_element_id`.

Five facade methods mint elements:
- `title(text) -> Element`
- `show(text) -> Element`
- `section(text) -> Element`
- `metric(label, value, direction) -> Element`
- `steps(items) -> Element`

**Critical gap:** All five methods discard their text arguments (prefixed with `_`).
No metadata (text content, element type, creation time) is stored anywhere.
The `Element` handle is returned to the caller but has no backing data.

### M Struct (facade.rs)

```rust
pub struct M {
    next_element_id: u64,
    current_theme: Theme,
    current_voice: Voice,
    timeline: Timeline,
}
```

No element registry. No mapping from Element ID to content or type.

---

## Timeline System (timeline.rs)

### Segment Enum

```rust
pub enum Segment {
    Narration { text: String, duration: f64 },
    Animation { name: String, duration: f64 },
    Silence { duration: f64 },
    Clip { path: PathBuf, duration: f64 },
}
```

Segments are purely temporal. No reference to which Element they affect.

### Timeline Queries

- `total_duration() -> f64` — sum of all segment durations
- `total_frames() -> u32` — ceil(duration * fps)
- `frame_at(time) -> u32` — time to frame number, clamped
- `segments_in_range(start, end) -> Vec<(f64, &Segment)>` — overlapping segments

`segments_in_range` is the key query for FrameState computation: given a timestamp,
find which segments are active at that instant.

---

## Technique System (moron-techniques/)

### TechniqueOutput

```rust
pub struct TechniqueOutput {
    pub opacity: f64,
    pub translate_x: f64,
    pub translate_y: f64,
    pub scale: f64,
    pub rotation: f64,
}
```

Default: opacity=1.0, translate=0, scale=1.0, rotation=0.
This is exactly the per-element visual state we need in FrameState.

**Note:** TechniqueOutput does NOT derive Serialize. It's in moron-techniques which
has serde as a dependency but doesn't use derive on this struct. We'll need to either
add Serialize there or create a parallel serializable struct in moron-core.

### Technique Trait

```rust
pub trait Technique {
    fn name(&self) -> &str;
    fn duration(&self) -> f64;
    fn apply(&self, progress: f64) -> TechniqueOutput;
}
```

`apply(progress)` takes 0.0-1.0 and returns the visual state. This is how we'll
compute element visual state during Animation segments.

---

## Theme System (moron-themes/)

### Theme Struct

`Theme` already derives `Serialize, Deserialize`. Has `to_css_properties() -> Vec<(String, String)>`.
This is ready to include in FrameState as-is.

---

## Dependencies

### moron-core/Cargo.toml

Already depends on: serde, serde_json, moron-techniques, moron-themes, moron-voice.
Everything needed for FrameState serialization is available.

### moron-techniques/Cargo.toml

Has serde as dependency but TechniqueOutput doesn't derive Serialize.

---

## Gap Analysis

1. **No element metadata storage.** M mints IDs but stores nothing about them.
2. **No element-to-segment mapping.** Timeline segments don't reference elements.
3. **No element-to-content mapping.** Text content passed to title/show/section is discarded.
4. **TechniqueOutput is not serializable.** Needs Serialize derive added.
5. **No FrameState type exists.** Must be created from scratch.
6. **Animation segments don't reference elements.** `play()` takes a technique but doesn't
   associate it with any Element. This is a design question — for now we can treat
   animations as ambient (applying to most recently created elements or all visible ones).

---

## Constraints

- FrameState is the Rust-to-React JSON contract. Must be clean, flat, easily consumed.
- serde Serialize + Deserialize required.
- Must be computable from `&M` + timestamp alone.
- Element tracking should be minimal — Vec of records, not a complex ECS.
- Keep under 15K lines total (currently well under).

---

## Key Files

| File | Role | Changes Needed |
|------|------|----------------|
| `moron-core/src/frame.rs` | NEW — FrameState types + compute function | Create |
| `moron-core/src/facade.rs` | M struct, element minting | Add element registry |
| `moron-core/src/lib.rs` | Module declarations, re-exports | Add frame module |
| `moron-core/src/timeline.rs` | Timeline queries | No changes needed |
| `moron-techniques/src/technique.rs` | TechniqueOutput | Add Serialize derive |
