//! High-level facade: simplified API surface for common rendering workflows.
//!
//! The `M` struct is the single entry point for scene authors. It hides all internal
//! machinery (Bevy ECS, renderer, timeline, FFmpeg, TTS) behind a clean,
//! sequential API. Scenes implement the `Scene` trait and receive `&mut M`.

use moron_themes::Theme;
use moron_voice::Voice;

use crate::timeline::{Segment, Timeline};

/// Default duration for `m.beat()` — a short rhythmic pause.
pub const BEAT_DURATION: f64 = 0.3;

/// Default duration for `m.breath()` — a slightly longer pause.
pub const BREATH_DURATION: f64 = 0.8;

/// Default words-per-minute for narration duration estimation.
pub const DEFAULT_NARRATION_WPM: f64 = 150.0;

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
    /// The timeline recording all segments produced during scene building.
    timeline: Timeline,
}

impl M {
    /// Create a new facade instance with default theme and voice.
    pub fn new() -> Self {
        Self {
            next_element_id: 0,
            current_theme: Theme::default(),
            current_voice: Voice::kokoro(),
            timeline: Timeline::default(),
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

    /// Get the recorded timeline.
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    // -- Content -----------------------------------------------------------

    /// Queue TTS narration for the given text.
    ///
    /// Duration is estimated from word count at [`DEFAULT_NARRATION_WPM`].
    pub fn narrate(&mut self, text: &str) {
        let words = text.split_whitespace().count().max(1) as f64;
        let duration = words * 60.0 / DEFAULT_NARRATION_WPM;
        self.timeline.add_segment(Segment::Narration {
            text: text.to_string(),
            duration,
        });
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

    /// Insert a short rhythmic pause ([`BEAT_DURATION`] seconds).
    pub fn beat(&mut self) {
        self.timeline
            .add_segment(Segment::Silence { duration: BEAT_DURATION });
    }

    /// Insert a slightly longer breathing pause ([`BREATH_DURATION`] seconds).
    pub fn breath(&mut self) {
        self.timeline
            .add_segment(Segment::Silence { duration: BREATH_DURATION });
    }

    /// Wait for an explicit duration in seconds.
    pub fn wait(&mut self, duration: f64) {
        self.timeline
            .add_segment(Segment::Silence { duration });
    }

    // -- Techniques --------------------------------------------------------

    /// Execute a composable animation technique.
    ///
    /// Records an [`Animation`](Segment::Animation) segment on the timeline.
    pub fn play(&mut self, technique: impl moron_techniques::Technique) {
        self.timeline.add_segment(Segment::Animation {
            name: technique.name().to_string(),
            duration: technique.duration(),
        });
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
    fn play_records_animation_segments() {
        use moron_techniques::{FadeIn, FadeUp, Technique};
        let mut m = M::new();
        m.play(FadeIn::default());
        m.play(FadeUp::default());

        assert_eq!(m.timeline().segments().len(), 2);
        let total = m.timeline().total_duration();
        let expected = FadeIn::default().duration() + FadeUp::default().duration();
        assert!((total - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn beat_adds_silence() {
        let mut m = M::new();
        m.beat();
        assert_eq!(m.timeline().segments().len(), 1);
        assert!((m.timeline().total_duration() - BEAT_DURATION).abs() < f64::EPSILON);
    }

    #[test]
    fn breath_adds_silence() {
        let mut m = M::new();
        m.breath();
        assert_eq!(m.timeline().segments().len(), 1);
        assert!((m.timeline().total_duration() - BREATH_DURATION).abs() < f64::EPSILON);
    }

    #[test]
    fn wait_adds_custom_silence() {
        let mut m = M::new();
        m.wait(2.5);
        assert_eq!(m.timeline().segments().len(), 1);
        assert!((m.timeline().total_duration() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn narrate_records_narration() {
        let mut m = M::new();
        m.narrate("Hello world");
        assert_eq!(m.timeline().segments().len(), 1);
        // "Hello world" = 2 words, 2 * 60 / 150 = 0.8s
        assert!((m.timeline().total_duration() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn timeline_tracks_cumulative_duration() {
        use moron_techniques::FadeIn;
        let mut m = M::new();
        m.narrate("Hello world");             // 0.8s
        m.beat();                              // 0.3s
        m.play(FadeIn { duration: 0.5 });     // 0.5s
        m.breath();                            // 0.8s
        m.wait(1.0);                           // 1.0s

        assert_eq!(m.timeline().segments().len(), 5);
        let expected = 0.8 + 0.3 + 0.5 + 0.8 + 1.0;
        assert!((m.timeline().total_duration() - expected).abs() < f64::EPSILON);
    }
}
