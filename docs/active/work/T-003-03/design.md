# T-003-03 Design: Implement Pacing Primitives

## Approach

Add `timeline: Timeline` field to M. Implement pacing methods to push Silence segments. Also wire play() and narrate() to record Animation and Narration segments.

### Constants

```rust
pub const BEAT_DURATION: f64 = 0.3;
pub const BREATH_DURATION: f64 = 0.8;
pub const DEFAULT_NARRATION_WPM: f64 = 150.0;
```

### Pacing Methods

- `beat()`: adds Silence { duration: BEAT_DURATION }
- `breath()`: adds Silence { duration: BREATH_DURATION }
- `wait(d)`: adds Silence { duration: d }

### play() Implementation

```rust
pub fn play(&mut self, technique: impl Technique) {
    self.timeline.add_segment(Segment::Animation {
        name: technique.name().to_string(),
        duration: technique.duration(),
    });
}
```

### narrate() Implementation

Estimate duration from word count: `words * 60 / WPM`. Stores text and estimated duration.

```rust
pub fn narrate(&mut self, text: &str) {
    let words = text.split_whitespace().count() as f64;
    let duration = words * 60.0 / DEFAULT_NARRATION_WPM;
    self.timeline.add_segment(Segment::Narration {
        text: text.to_string(),
        duration,
    });
}
```

### M Timeline Access

Add `pub fn timeline(&self) -> &Timeline` getter so tests and downstream consumers can inspect the recording.
