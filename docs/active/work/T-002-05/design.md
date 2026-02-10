# T-002-05 Design: Wire Types Into Facade

## Decision: Direct Import with Stored State

### Approach

Replace the three placeholder types in facade.rs with direct imports from their crates. Store Theme and Voice as owned fields on M. The play() method takes `impl moron_techniques::Technique` but its body remains `todo!()` until T-003-02.

### Alternatives Considered

**A) Type aliases in facade.rs** — `type Theme = moron_themes::Theme;` etc.
- Pro: Minimal change, facade controls its own namespace
- Con: Adds indirection for no benefit. The types are public and stable.
- Rejected: Unnecessary abstraction layer.

**B) Trait objects for Theme/Voice** — `Box<dyn ThemeLike>`, `Box<dyn VoiceLike>`
- Pro: Maximum decoupling
- Con: These are concrete config structs, not polymorphic. No dispatch needed.
- Rejected: Over-engineering. Theme and Voice are data, not behavior.

**C) Direct import with stored state** (chosen)
- Pro: Simple, idiomatic, no wrappers. `M.theme` is the actual Theme struct.
- Pro: Matches how scene authors will use it: `m.theme(Theme::default())`
- Con: M now depends on concrete types from sibling crates (already the case via Cargo.toml)

### Design Details

#### M struct fields

```rust
pub struct M {
    next_element_id: u64,
    current_theme: Theme,    // moron_themes::Theme
    current_voice: Voice,    // moron_voice::Voice
}
```

Fields are private. Accessed via:
- `m.theme(new_theme)` — setter, replaces current theme
- `m.voice(new_voice)` — setter, replaces current voice
- `m.current_theme()` — getter (new), returns `&Theme`
- `m.current_voice()` — getter (new), returns `&Voice`

Getters are needed so downstream (timeline, renderer) can read config without owning M.

#### M::new() defaults

```rust
pub fn new() -> Self {
    Self {
        next_element_id: 0,
        current_theme: Theme::default(),      // moron-dark
        current_voice: Voice::kokoro(),        // Kokoro at 1.0x speed
    }
}
```

#### play() signature

```rust
pub fn play(&mut self, _technique: impl moron_techniques::Technique) {
    todo!()
}
```

The `impl Technique` bound uses the real trait, which requires `name()` and `duration()`. Body stays `todo!()` — technique dispatch involves timeline recording which is T-003-02.

#### Prelude module

New file: no. Inline module in lib.rs:

```rust
pub mod prelude {
    pub use moron_techniques::{Ease, Technique, TechniqueExt};
    pub use moron_themes::Theme;
    pub use moron_voice::Voice;
    pub use crate::facade::{Direction, Element, M, Scene};
}
```

Re-exports everything a scene author needs in one `use moron_core::prelude::*;`.

Includes `TechniqueExt` so `.with_ease()` is available without extra imports.

#### lib.rs root re-exports

Remove the placeholder re-exports of Theme/Voice/Technique from root. Keep Direction, Element, M, Scene at root level. Add the prelude module. Scene authors should use `prelude::*`; internal code can import specific items.

Updated lib.rs re-exports:
```rust
pub use facade::{Direction, Element, M, Scene};
pub use moron_techniques::{Ease, Technique};
pub use moron_themes::Theme;
pub use moron_voice::Voice;
```

This maintains backward compat: `moron_core::Theme` still resolves.

### Testing Strategy

1. Update existing tests in facade.rs to work with real types
2. Add integration test: construct M with defaults, set theme/voice, call play() would panic (todo), but we can test everything up to that
3. Test: construct M, verify default theme name is "moron-dark"
4. Test: construct M, verify default voice is Kokoro
5. Test: set custom theme, read it back via getter
6. Test: set custom voice, read it back via getter
7. Test: Scene trait still works with new M

### What This Does NOT Do

- Implement technique dispatch (T-003-02)
- Implement narration pipeline (requires timeline from T-003-01)
- Implement pacing methods (T-003-03)
- Add CLI demonstration (optional per ticket, deferring)
