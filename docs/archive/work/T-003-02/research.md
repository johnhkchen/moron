# T-003-02 Research: Implement Core Techniques

## Current State

The Technique trait in technique.rs has two methods: `name()` and `duration()`. Six concrete technique structs exist with these implementations but no visual output logic. The ticket requires adding `apply(progress: f64) -> TechniqueOutput` to the trait.

### Existing Technique Structs

| Technique | File | Fields | Behavior Needed |
|-----------|------|--------|-----------------|
| FadeIn | reveals.rs | duration | opacity 0→1 |
| FadeUp | reveals.rs | duration, distance(30.0) | opacity 0→1, translate_y from distance→0 |
| Slide | motion.rs | duration, offset_x(100.0), offset_y(0.0) | translate from offset→0 |
| Scale | motion.rs | duration, from(0.0), to(1.0) | scale from→to |
| CountUp | data.rs | duration, from(0.0), to(100.0) | value interpolation (represented as opacity for now) |
| Stagger&lt;T&gt; | staging.rs | inner, delay, count | distributes progress across items |

### Easing Curves Needed

Ease enum has 7 variants: Linear, EaseIn, EaseOut, EaseInOut, OutBack, OutBounce, Spring.

Standard formulas:
- Linear: `t`
- EaseIn: `t^2` (quadratic)
- EaseOut: `1 - (1-t)^2`
- EaseInOut: cubic-bezier style or piecewise quadratic
- OutBack: overshoots then settles (`t * t * ((s+1)*t - s)` with s=1.70158, applied as out variant)
- OutBounce: bouncing ball math
- Spring: damped spring oscillation

### WithEase Combinator

`WithEase<T>` wraps a technique with an ease. Its `apply()` should remap `progress` through the easing function, then delegate to `inner.apply(eased_progress)`.

### Stagger Complexity

Stagger applies the inner technique to N items with a delay. `apply()` returns the output for a single reference item (e.g., item 0). The renderer would call `apply_item(index, progress)` for per-item state. For now, Stagger.apply() can return the first item's state.

## Constraints

- Adding `apply()` to the Technique trait is a **breaking change** — all implementors must update.
- TechniqueOutput must be general enough for all techniques (opacity, translate, scale, rotation).
- Progress is always 0.0 to 1.0 (caller-normalized).
