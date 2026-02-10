# T-002-05 Plan: Wire Types Into Facade

## Steps

### Step 1: Remove placeholders and add imports in facade.rs

- Delete the placeholder `Technique` trait, `Theme` struct, `Voice` struct (lines 8-21)
- Add `use moron_themes::Theme;` and `use moron_voice::Voice;`
- Verify: `cargo check -p moron-core` will fail (M methods reference removed types — expected, fixed in step 2)

### Step 2: Update M struct and constructor

- Add `current_theme: Theme` and `current_voice: Voice` fields to M
- Update `M::new()` to initialize with `Theme::default()` and `Voice::kokoro()`
- Update `theme()` method: `self.current_theme = theme;`
- Update `voice()` method: `self.current_voice = voice;`
- Update `play()` signature: `impl moron_techniques::Technique`
- Add getters: `current_theme(&self) -> &Theme`, `current_voice(&self) -> &Voice`
- Verify: `cargo check -p moron-core`

### Step 3: Update lib.rs re-exports and add prelude

- Replace facade placeholder re-exports with real crate types
- Add `pub mod prelude { ... }` with Scene, M, Element, Direction, Theme, Voice, Technique, TechniqueExt, Ease
- Verify: `cargo check -p moron-core`

### Step 4: Update and add tests

- Update `default_m` test to verify theme name and voice backend
- Add `theme_and_voice_setters` test
- Add `play_accepts_real_techniques` compile-check test
- Add integration test: construct M via prelude, exercise full API
- Verify: `cargo test -p moron-core`

### Step 5: Full workspace verification

- `cargo check` — full workspace
- `cargo test` — all crates
- `cargo clippy` — no warnings

### Step 6: Commit and update ticket phase

- Commit with message: `feat(core): wire Theme, Voice, Technique into M facade (T-002-05)`
- Update ticket frontmatter: phase → implement → done
