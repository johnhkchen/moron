# T-008-02 Plan: clippy-pedantic-techniques

## Steps

1. Fix technique.rs: `#[must_use]` on `ease()`, digit separator in `ease_out_bounce()`
2. Fix reveals.rs: backtick doc comment, `&'static str` on two `name()` impls
3. Fix motion.rs: backtick doc comment, `&'static str` on two `name()` impls
4. Fix staging.rs: backtick doc comment, `#[must_use]` on builders, `&'static str` on `name()`, `#[allow]` on casts
5. Fix emphasis.rs, camera.rs, transitions.rs: backtick doc comments
6. Fix data.rs: backtick doc comment, `#[must_use]` on `current_value()`, `&'static str` on `name()`
7. Run `cargo clippy -p moron-techniques -- -W clippy::pedantic` — verify 0 warnings
8. Run `cargo test -p moron-techniques` — verify all tests pass unchanged
9. Run `cargo clippy` — verify workspace still clean
10. Update ticket frontmatter to phase: done, status: done
