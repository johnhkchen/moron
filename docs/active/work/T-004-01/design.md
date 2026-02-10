# T-004-01 Design: Frame State Serialization

## Problem

Given a built scene (M with populated timeline and elements) and a timestamp,
compute the complete visual state for React to render. This state must serialize
to JSON.

---

## Design Decisions

### 1. Element Metadata Storage

**Options considered:**

A) **HashMap<u64, ElementRecord> on M** — standard registry pattern.
B) **Vec<ElementRecord> on M** — simpler, indexed by element ID since IDs are sequential.
C) **Store metadata in Segment variants** — extend Segment to carry element references.

**Decision: (B) Vec<ElementRecord> on M.**

Rationale: Element IDs are minted sequentially starting from 0, so a Vec naturally
indexes by ID. Simpler than HashMap, no hashing overhead. Option C would require
changing the Segment enum which other code depends on, and conflates temporal data
with identity data.

### 2. Element Type Taxonomy

```rust
enum ElementKind {
    Title,
    Show,
    Section,
    Metric { direction: Direction },
    Steps { count: usize },
}
```

Each variant captures the structural type and any extra metadata specific to that type.
Direction is already defined in facade.rs. Steps needs to know item count for rendering.

### 3. ElementRecord Structure

```rust
struct ElementRecord {
    id: u64,
    kind: ElementKind,
    content: String,       // primary text content
    items: Vec<String>,    // for Steps variant; empty for others
    segment_index: usize,  // which timeline segment introduces this element
}
```

`segment_index` links the element to the timeline. When the facade mints an element,
it also pushes an Animation segment for the element's entrance. The segment_index
records which segment that is, enabling time-based visibility queries.

**Revision:** Actually, the current facade methods (title, show, etc.) do NOT push
segments. They only mint elements. Segments come from separate calls: narrate(),
play(), beat(), etc. The element lifecycle is decoupled from the timeline.

**Revised approach:** Instead of segment_index, record the timeline cursor position
(cumulative time) when the element was created. This gives us a `created_at` timestamp.
Elements become visible at their creation time and remain visible until the end
(or until explicitly hidden, which we can add later).

```rust
struct ElementRecord {
    id: u64,
    kind: ElementKind,
    content: String,
    items: Vec<String>,
    created_at: f64,  // timeline cursor when element was minted
}
```

### 4. Timeline Cursor on M

Add a `cursor: f64` field to M that tracks the current position as segments are added.
When `add_segment` advances the timeline, cursor moves forward. When an element is
minted, it records the current cursor as its `created_at`.

Wait — the current facade doesn't advance a cursor. The timeline just appends segments.
We need to compute cursor from the timeline's current total_duration at mint time.

Simpler: `created_at = self.timeline.total_duration()` at the moment of minting.
This means elements appear at the point in the timeline where they were declared.

### 5. FrameState Structure

```rust
struct FrameState {
    time: f64,
    frame: u32,
    total_duration: f64,
    fps: u32,
    elements: Vec<ElementState>,
    active_narration: Option<String>,
    theme: ThemeState,
}
```

### 6. ElementState (per-element visual state)

```rust
struct ElementState {
    id: u64,
    kind: ElementKind,
    content: String,
    items: Vec<String>,
    visible: bool,
    opacity: f64,
    translate_x: f64,
    translate_y: f64,
    scale: f64,
    rotation: f64,
}
```

Flattened visual properties from TechniqueOutput. This avoids React needing to
understand TechniqueOutput nesting.

### 7. ThemeState

Rather than embedding the full Theme struct (which is large and has Rust-specific
naming), convert to CSS properties for the JSON contract:

```rust
struct ThemeState {
    name: String,
    css_properties: Vec<(String, String)>,
}
```

Actually, for simplicity and since Theme already has Serialize, we can just include
the Theme directly. React can consume either representation. Let's use
`HashMap<String, String>` for css_properties to make JSON cleaner.

### 8. Computing FrameState

```rust
fn compute_frame_state(m: &M, time: f64) -> FrameState
```

Algorithm:
1. Clamp time to [0, total_duration].
2. Compute frame number from timeline.
3. Find active segments at time via segments_in_range.
4. For each element record, determine visibility: visible if created_at <= time.
5. For visible elements, apply default visual state (fully visible, no transform).
6. Find active narration text from overlapping Narration segments.
7. Collect theme as CSS properties.

Animation application is deferred to a future ticket (T-004-02 or similar) since
`play()` doesn't currently associate techniques with specific elements.

### 9. Serde Strategy

All FrameState types derive `Serialize, Deserialize`. Use `#[serde(rename_all = "camelCase")]`
for JavaScript-friendly JSON output.

TechniqueOutput in moron-techniques needs `Serialize, Deserialize` added.
It already has serde as a dependency.

---

## What Was Rejected

- **ECS-style component storage:** Over-engineered for current needs. Vec is sufficient.
- **Storing Technique objects on elements:** Techniques are ephemeral (consumed by play()).
  Visual state is computed at query time from timeline position, not stored.
- **Segment-element coupling:** Would require breaking changes to Segment enum.
  Element lifecycle is orthogonal to timeline segments for now.
- **Lazy computation / caching:** Premature optimization. Compute fresh each frame.
