# T-002-05 Structure: Wire Types Into Facade

## Files Modified

### moron-core/src/facade.rs

**Remove** (lines 8-21):
- Placeholder `trait Technique {}`
- Placeholder `struct Theme;`
- Placeholder `struct Voice;`
- Associated TODO comments

**Add** imports at top:
```rust
use moron_themes::Theme;
use moron_voice::Voice;
```

Note: `moron_techniques::Technique` is used inline as `impl moron_techniques::Technique` in the play() signature rather than imported, to avoid trait name collision in the module namespace.

**Modify** M struct:
```rust
pub struct M {
    next_element_id: u64,
    current_theme: Theme,
    current_voice: Voice,
}
```

**Modify** M::new():
- Initialize `current_theme: Theme::default()`
- Initialize `current_voice: Voice::kokoro()`

**Modify** M methods:
- `theme(&mut self, theme: Theme)` — store in `self.current_theme`
- `voice(&mut self, voice: Voice)` — store in `self.current_voice`
- `play(&mut self, _technique: impl moron_techniques::Technique)` — keep `todo!()`

**Add** getter methods:
- `current_theme(&self) -> &Theme`
- `current_voice(&self) -> &Voice`

**Modify** Default impl:
- Delegate to `Self::new()` (unchanged, but now includes theme/voice)

**Modify** tests:
- `default_m` test: update to also check theme/voice defaults
- Add new test: `theme_and_voice_setters`
- Add new test: `play_accepts_real_techniques` (compile-time check, no call)

### moron-core/src/lib.rs

**Modify** root re-exports:
```rust
pub use facade::{Direction, Element, M, Scene};
pub use moron_techniques::{Ease, Technique};
pub use moron_themes::Theme;
pub use moron_voice::Voice;
```

**Add** prelude module:
```rust
pub mod prelude {
    pub use moron_techniques::{Ease, Technique, TechniqueExt};
    pub use moron_themes::Theme;
    pub use moron_voice::Voice;
    pub use crate::facade::{Direction, Element, M, Scene};
}
```

## Files NOT Modified

- moron-core/Cargo.toml — dependencies already present
- moron-cli/src/main.rs — CLI demo deferred (optional per ticket)
- moron-techniques/* — no changes needed
- moron-themes/* — no changes needed
- moron-voice/* — no changes needed

## Module Boundary

After this change, `moron-core` is the integration point:
- It imports concrete types from sibling crates
- The `M` facade owns instances of `Theme` and `Voice`
- The `prelude` module is the recommended import for scene authors
- The root re-exports maintain `moron_core::Theme` etc. for non-prelude usage

## Public API Surface (prelude)

| Symbol | Source | Kind |
|--------|--------|------|
| Scene | facade | trait |
| M | facade | struct |
| Element | facade | struct |
| Direction | facade | enum |
| Theme | moron-themes | struct |
| Voice | moron-voice | struct |
| Technique | moron-techniques | trait |
| TechniqueExt | moron-techniques | trait |
| Ease | moron-techniques | enum |
