# T-008-02 Design: clippy-pedantic-techniques

## Approach

All 34 warnings have clear, mechanical fixes. One decision point exists.

### Decision: `cast_precision_loss` in staging.rs

**Option A:** Allow the lint on specific lines with `#[allow(clippy::cast_precision_loss)]`.
**Option B:** Use `as f64` with a wrapping comment explaining why it's safe.
**Option C:** Convert to use `f64::from()` where possible.

`f64::from(u32)` exists but `f64::from(usize)` does not (platform-dependent width). We'd need a cast through `u32` or similar, which adds complexity for no safety gain. Element counts will never exceed 2^52.

**Decision:** Option A — allow the lint on the two specific lines. This is the idiomatic approach for known-safe usize-to-f64 casts.

### Decision: `unnecessary_literal_bound` on trait impls

Change only the impl signatures to `-> &'static str`, not the trait definition. This is what clippy suggests, and it works because `&'static str` is a subtype of `&'a str`. The trait stays general (future impls might compute names dynamically), while concrete impls document that they return static strings.

### All Other Warnings

Mechanical fixes:
- Add `#[must_use]` attributes where clippy requests them
- Add digit separators to `0.984375` → `0.984_375`
- Add backticks around CamelCase names in doc comments
