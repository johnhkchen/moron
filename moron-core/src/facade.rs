//! High-level facade: simplified API surface for common rendering workflows.
//!
//! The `M` struct is the single entry point for scene authors. It hides all internal
//! machinery (Bevy ECS, renderer, timeline, FFmpeg, TTS) behind a clean,
//! sequential API. Scenes implement the `Scene` trait and receive `&mut M`.

// ---------------------------------------------------------------------------
// Placeholder traits/structs for cross-crate types
// ---------------------------------------------------------------------------

/// Placeholder trait for composable animation techniques.
// TODO: Replace with types from moron-techniques in T-002-05
pub trait Technique {}

/// Placeholder struct for theme configuration.
// TODO: Replace with types from moron-themes in T-002-05
pub struct Theme;

/// Placeholder struct for voice/TTS configuration.
// TODO: Replace with types from moron-voice in T-002-05
pub struct Voice;

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Direction indicator for metric displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Neutral,
}

/// Opaque handle to a visual element on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element(pub(crate) u64);

// ---------------------------------------------------------------------------
// Scene trait
// ---------------------------------------------------------------------------

/// The trait that every scene file implements.
///
/// A scene receives an `&mut M` and uses its methods to describe what should
/// appear on screen, in what order, and with what timing.
pub trait Scene {
    /// Build the scene by issuing commands against the facade.
    fn build(m: &mut M);
}

// ---------------------------------------------------------------------------
// M — the facade
// ---------------------------------------------------------------------------

/// The facade struct that scene authors interact with.
///
/// All internal machinery is accessed exclusively through `M`'s methods.
pub struct M {
    /// Monotonically increasing counter used to mint `Element` handles.
    next_element_id: u64,
}

impl M {
    /// Create a new, empty facade instance.
    pub fn new() -> Self {
        Self {
            next_element_id: 0,
        }
    }

    // -- Content -----------------------------------------------------------

    /// Queue TTS narration for the given text.
    pub fn narrate(&mut self, _text: &str) {
        todo!()
    }

    /// Display text on screen in a context-aware manner.
    pub fn show(&mut self, _text: &str) -> Element {
        self.mint_element()
    }

    // -- Structure ---------------------------------------------------------

    /// Display a title card.
    pub fn title(&mut self, _text: &str) -> Element {
        self.mint_element()
    }

    /// Begin a new named section.
    pub fn section(&mut self, _text: &str) -> Element {
        self.mint_element()
    }

    /// Display a metric with a directional indicator.
    pub fn metric(&mut self, _label: &str, _value: &str, _direction: Direction) -> Element {
        self.mint_element()
    }

    /// Reveal a list of items with staggered timing.
    pub fn steps(&mut self, _items: &[&str]) -> Element {
        self.mint_element()
    }

    // -- Pacing ------------------------------------------------------------

    /// Insert a short rhythmic pause (roughly a "beat" in timing).
    pub fn beat(&mut self) {
        todo!()
    }

    /// Insert a slightly longer breathing pause.
    pub fn breath(&mut self) {
        todo!()
    }

    /// Wait for an explicit duration in seconds.
    pub fn wait(&mut self, _duration: f64) {
        todo!()
    }

    // -- Techniques --------------------------------------------------------

    /// Execute a composable animation technique.
    pub fn play(&mut self, _technique: impl Technique) {
        todo!()
    }

    // -- Configuration -----------------------------------------------------

    /// Set the active theme.
    pub fn theme(&mut self, _theme: Theme) {
        todo!()
    }

    /// Set the active TTS voice.
    pub fn voice(&mut self, _voice: Voice) {
        todo!()
    }

    // -- Internal helpers --------------------------------------------------

    /// Allocate the next `Element` handle.
    fn mint_element(&mut self) -> Element {
        let id = self.next_element_id;
        self.next_element_id += 1;
        Element(id)
    }
}

impl Default for M {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_m_and_create_elements() {
        let mut m = M::new();

        let e1 = m.title("Hello");
        let e2 = m.show("World");
        let e3 = m.section("Part 1");
        let e4 = m.metric("Revenue", "$1M", Direction::Up);
        let e5 = m.steps(&["one", "two", "three"]);

        // Each call should mint a unique element handle.
        let ids = [e1, e2, e3, e4, e5];
        for (i, a) in ids.iter().enumerate() {
            for (j, b) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "elements {i} and {j} must be unique");
                }
            }
        }
    }

    #[test]
    fn scene_trait_is_implementable() {
        struct DemoScene;

        impl Scene for DemoScene {
            fn build(m: &mut M) {
                m.title("Demo");
                m.show("content");
            }
        }

        let mut m = M::new();
        DemoScene::build(&mut m);
    }

    #[test]
    fn direction_enum_variants() {
        // Ensure all variants exist and are distinct.
        assert_ne!(Direction::Up, Direction::Down);
        assert_ne!(Direction::Up, Direction::Neutral);
        assert_ne!(Direction::Down, Direction::Neutral);
    }

    #[test]
    fn default_m() {
        let m = M::default();
        assert_eq!(m.next_element_id, 0);
    }
}
