# T-002-05 Research: Wire Types Into Facade

## Objective

Integrate types from moron-themes, moron-voice, and moron-techniques into moron-core's M facade. Create a prelude module for convenient re-exports.

## Current State

### M facade (moron-core/src/facade.rs)

The M struct currently holds only `next_element_id: u64`. It defines **placeholder** types:

- `pub trait Technique {}` — local placeholder, no methods
- `pub struct Theme;` — unit struct placeholder
- `pub struct Voice;` — unit struct placeholder

These placeholders are re-exported from `moron-core/src/lib.rs`:
```rust
pub use facade::{Direction, Element, M, Scene, Technique, Theme, Voice};
```

Methods that use these placeholders:
- `play(&mut self, _technique: impl Technique)` — takes local placeholder trait
- `theme(&mut self, _theme: Theme)` — takes local placeholder struct
- `voice(&mut self, _voice: Voice)` — takes local placeholder struct

All three methods have `todo!()` bodies.

### moron-techniques types

- `Technique` trait: `name() -> &str`, `duration() -> f64`
- `TechniqueExt` trait (blanket impl): `with_ease(self, ease: Ease) -> WithEase<Self>`
- `Ease` enum: Linear, EaseIn, EaseOut, EaseInOut, OutBack, OutBounce, Spring
- Concrete techniques: FadeIn, FadeUp, Slide, Scale, Stagger, CountUp

### moron-themes types

- `Theme` struct: name, colors, typography, spacing, timing, shadows
- Implements Default (moron-dark theme), Serialize, Deserialize, PartialEq, Clone
- `to_css_properties() -> Vec<(String, String)>`

### moron-voice types

- `Voice` struct: backend_type, speed, pitch
- Constructors: `Voice::kokoro()`, `Voice::piper()`, `Voice::file(path)`
- `VoiceBackend` trait: `synthesize(text) -> Result<AudioClip>`, `name() -> &str`
- `VoiceBackendType` enum: Kokoro, Piper, ApiProvider(String), PreRecorded(PathBuf)

### Cargo.toml dependencies

moron-core already depends on moron-techniques, moron-voice, moron-themes as workspace deps. No new dependencies needed.

## Key Observations

1. **Name collision**: The facade defines local `Technique`, `Theme`, `Voice` that shadow the real crate types. These must be removed and replaced with `use` imports from the real crates.

2. **Trait vs struct**: The local `Technique` is a trait (empty marker). The real `moron_techniques::Technique` is also a trait but with `name()` and `duration()` methods. The `play()` signature `impl Technique` is compatible — just swap the trait import.

3. **M needs state**: M should store the current Theme and Voice so that downstream consumers (timeline, renderer) can access configuration. Both types implement Clone.

4. **Default construction**: `M::new()` should use `Theme::default()` and `Voice::kokoro()` for sensible defaults.

5. **Prelude module**: The ticket requires `moron_core::prelude` re-exporting: Scene, M, Theme, Voice, Technique, Ease, Direction. This is a new module in moron-core/src/lib.rs.

6. **lib.rs re-exports**: Current root re-exports (`pub use facade::{...Theme, Voice, Technique...}`) must switch to the real types from their crates instead of the facade placeholders.

7. **Existing tests**: Four tests in facade.rs. `construct_m_and_create_elements` and `scene_trait_is_implementable` don't touch theme/voice/technique. `default_m` checks `next_element_id == 0`. These should continue to pass with minor updates.

## Files Involved

| File | Action | Notes |
|------|--------|-------|
| moron-core/src/facade.rs | Modify | Remove placeholders, add imports, add theme/voice fields to M |
| moron-core/src/lib.rs | Modify | Add prelude module, update re-exports |
| moron-core/Cargo.toml | None | Dependencies already present |

## Constraints

- `cargo check` must pass across the full workspace
- No breaking changes to Scene trait or Element/Direction types
- M::new() must remain a simple constructor (no Result, no async)
- play() body can remain `todo!()` — technique dispatch is T-003-02's concern
- theme()/voice() should store the values, not just discard them
