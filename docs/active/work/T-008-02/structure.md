# T-008-02 Structure: clippy-pedantic-techniques

## Files Modified

All changes are in `moron-techniques/src/`. No files created or deleted.

### technique.rs
- Add `#[must_use]` to `pub fn ease()`
- Change `0.984375` to `0.984_375` in `ease_out_bounce()`

### reveals.rs
- Backtick CamelCase names in module doc comment (line 1)
- Change `fn name(&self) -> &str` to `-> &'static str` in FadeIn and FadeUp impls

### motion.rs
- Backtick CamelCase names in module doc comment (line 1)
- Change `fn name(&self) -> &str` to `-> &'static str` in Slide and Scale impls

### staging.rs
- Backtick CamelCase names in module doc comment (line 1)
- Add `#[must_use]` to `with_delay()` and `with_count()`
- Change `fn name(&self) -> &str` to `-> &'static str` in Stagger impl
- Add `#[allow(clippy::cast_precision_loss)]` on two lines with `as f64`

### emphasis.rs
- Backtick CamelCase names in module doc comment (line 1)

### camera.rs
- Backtick CamelCase names in module doc comment (line 1)

### transitions.rs
- Backtick CamelCase names in module doc comment (line 1)

### data.rs
- Backtick CamelCase names in module doc comment (line 1)
- Add `#[must_use]` to `pub fn current_value()`
- Change `fn name(&self) -> &str` to `-> &'static str` in CountUp impl
