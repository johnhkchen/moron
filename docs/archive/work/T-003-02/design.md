# T-003-02 Design: Implement Core Techniques

## Approach: TechniqueOutput + apply() on Trait

### TechniqueOutput

```rust
struct TechniqueOutput {
    opacity: f64,        // 0.0 = transparent, 1.0 = fully opaque
    translate_x: f64,    // pixels
    translate_y: f64,    // pixels
    scale: f64,          // 1.0 = normal
    rotation: f64,       // degrees
}
```

Default: identity (opacity=1.0, translate=0.0, scale=1.0, rotation=0.0).

### Technique Trait Addition

```rust
fn apply(&self, progress: f64) -> TechniqueOutput;
```

progress is 0.0 at start, 1.0 at end. Caller is responsible for normalization.

### Easing Module

New function in technique.rs: `pub fn ease(curve: Ease, t: f64) -> f64`

Keeps easing math centralized. Used by WithEase wrapper and available for direct use.

### Technique Implementations

- **FadeIn**: opacity = progress, rest identity
- **FadeUp**: opacity = progress, translate_y = distance * (1.0 - progress)
- **Slide**: translate_x = offset_x * (1.0 - progress), translate_y = offset_y * (1.0 - progress)
- **Scale**: scale = from + (to - from) * progress
- **CountUp**: value represented via opacity as (from + (to-from)*progress) / to (normalized 0-1). CountUp is really about the value, not visual transform — but we need apply() to satisfy the trait. Use opacity as the primary channel.
- **Stagger**: apply() returns inner.apply(progress) for the reference item. Add `apply_item(index, progress) -> TechniqueOutput` as a method on Stagger itself.

### WithEase

```rust
fn apply(&self, progress: f64) -> TechniqueOutput {
    let eased = ease(self.ease, progress);
    self.inner.apply(eased)
}
```

### Rejected Alternatives

- **Separate easing module**: Unnecessary file for ~50 lines of math.
- **TechniqueOutput as trait object**: Over-engineered. Struct is simple and concrete.
- **Generic output type**: `type Output` associated type. Too complex for a uniform rendering pipeline.
