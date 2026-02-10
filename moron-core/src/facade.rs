//! High-level facade: simplified API surface for common rendering workflows.
//!
//! The `M` struct is the single entry point for scene authors. It hides all internal
//! machinery (Bevy ECS, renderer, timeline, FFmpeg, TTS) behind a clean,
//! sequential API. Scenes implement the `Scene` trait and receive `&mut M`.

use moron_themes::Theme;
use moron_voice::Voice;

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
    /// Active theme configuration.
    current_theme: Theme,
    /// Active voice/TTS configuration.
    current_voice: Voice,
}

impl M {
    /// Create a new facade instance with default theme and voice.
    pub fn new() -> Self {
        Self {
            next_element_id: 0,
            current_theme: Theme::default(),
            current_voice: Voice::kokoro(),
        }
    }

    /// Get the current theme.
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    /// Get the current voice configuration.
    pub fn current_voice(&self) -> &Voice {
        &self.current_voice
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
    pub fn play(&mut self, _technique: impl moron_techniques::Technique) {
        todo!()
    }

    // -- Configuration -----------------------------------------------------

    /// Set the active theme.
    pub fn theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }

    /// Set the active TTS voice.
    pub fn voice(&mut self, voice: Voice) {
        self.current_voice = voice;
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
        assert_eq!(m.current_theme().name, "moron-dark");
        assert!(matches!(
            m.current_voice().backend_type,
            moron_voice::VoiceBackendType::Kokoro
        ));
    }

    #[test]
    fn theme_and_voice_setters() {
        let mut m = M::new();

        // Change theme
        let mut custom_theme = Theme::default();
        custom_theme.name = "custom".to_string();
        m.theme(custom_theme);
        assert_eq!(m.current_theme().name, "custom");

        // Change voice
        let piper = Voice::piper();
        m.voice(piper);
        assert!(matches!(
            m.current_voice().backend_type,
            moron_voice::VoiceBackendType::Piper
        ));
    }

    #[test]
    fn play_accepts_real_techniques() {
        // Compile-time check: M.play() accepts concrete technique types.
        fn _assert_compiles(m: &mut M) {
            use moron_techniques::{FadeIn, FadeUp, Slide, Scale, Stagger, CountUp, TechniqueExt, Ease};
            m.play(FadeIn::default());
            m.play(FadeUp::default());
            m.play(Slide::default());
            m.play(Scale::default());
            m.play(CountUp::default());
            m.play(Stagger::new(FadeUp::default()));
            m.play(FadeUp::default().with_ease(Ease::OutBack));
        }
    }
}
