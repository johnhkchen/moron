//! High-level facade: simplified API surface for common rendering workflows.
//!
//! The `M` struct is the single entry point for scene authors. It hides all internal
//! machinery (Bevy ECS, renderer, timeline, FFmpeg, TTS) behind a clean,
//! sequential API. Scenes implement the `Scene` trait and receive `&mut M`.

use moron_themes::Theme;
use moron_voice::Voice;

use crate::frame::ElementKind;
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

/// Internal record of an element's metadata, used for frame state computation.
#[derive(Debug, Clone)]
pub(crate) struct ElementRecord {
    /// Element identifier (matches the Element handle).
    pub id: u64,
    /// Structural type of this element.
    pub kind: ElementKind,
    /// Primary text content.
    pub content: String,
    /// List items (non-empty only for Steps elements).
    pub items: Vec<String>,
    /// Timeline position (in seconds) when this element was created.
    pub created_at: f64,
    /// Number of timeline segments that existed when this element was created.
    /// Used to recompute `created_at` after narration durations are resolved.
    pub segments_at_creation: usize,
    /// Timeline position (in seconds) when this element was cleared from the screen.
    /// `None` means the element stays visible until the end of the timeline.
    pub ended_at: Option<f64>,
    /// Number of timeline segments that existed when this element was ended.
    /// Used to recompute `ended_at` after narration durations are resolved.
    pub segments_at_end: Option<usize>,
}

/// Internal record of an animation, binding a technique to its target elements.
///
/// Stored on `M` and queried by `compute_frame_state()` to apply animation
/// transforms to elements during rendering.
pub(crate) struct AnimationRecord {
    /// The boxed technique object — called via `apply(progress)` at frame time.
    pub technique: Box<dyn moron_techniques::Technique>,
    /// Element IDs this animation applies to.
    pub target_ids: Vec<u64>,
    /// Index into `timeline.segments()` for this animation's segment.
    /// Used to compute absolute time window (survives duration resolution).
    pub segment_index: usize,
}

// ---------------------------------------------------------------------------
// ResolveDurationError
// ---------------------------------------------------------------------------

/// Error returned when narration duration resolution fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveDurationError {
    /// The number of provided durations does not match the number of narration segments.
    LengthMismatch {
        /// Number of narration segments in the timeline.
        expected: usize,
        /// Number of durations provided by the caller.
        provided: usize,
    },
}

impl std::fmt::Display for ResolveDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LengthMismatch { expected, provided } => {
                write!(
                    f,
                    "narration duration count mismatch: expected {expected}, got {provided}"
                )
            }
        }
    }
}

impl std::error::Error for ResolveDurationError {}

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
    /// Registry of all minted elements with their metadata.
    elements: Vec<ElementRecord>,
    /// Registry of all animation records (technique + target + segment index).
    animations: Vec<AnimationRecord>,
}

impl M {
    /// Create a new facade instance with default theme and voice.
    pub fn new() -> Self {
        Self {
            next_element_id: 0,
            current_theme: Theme::default(),
            current_voice: Voice::kokoro(),
            timeline: Timeline::default(),
            elements: Vec::new(),
            animations: Vec::new(),
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
    pub fn show(&mut self, text: &str) -> Element {
        self.mint_element_with_meta(ElementKind::Show, text.to_string(), Vec::new())
    }

    // -- Structure ---------------------------------------------------------

    /// Display a title card.
    pub fn title(&mut self, text: &str) -> Element {
        self.mint_element_with_meta(ElementKind::Title, text.to_string(), Vec::new())
    }

    /// Begin a new named section.
    pub fn section(&mut self, text: &str) -> Element {
        self.mint_element_with_meta(ElementKind::Section, text.to_string(), Vec::new())
    }

    /// Display a metric with a directional indicator.
    pub fn metric(&mut self, label: &str, value: &str, direction: Direction) -> Element {
        let dir_str = match direction {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Neutral => "neutral",
        };
        self.mint_element_with_meta(
            ElementKind::Metric {
                direction: dir_str.to_string(),
            },
            format!("{label}: {value}"),
            Vec::new(),
        )
    }

    /// Reveal a list of items with staggered timing.
    pub fn steps(&mut self, items: &[&str]) -> Element {
        let items_vec: Vec<String> = items.iter().map(|s| s.to_string()).collect();
        self.mint_element_with_meta(
            ElementKind::Steps {
                count: items_vec.len(),
            },
            items.join(", "),
            items_vec,
        )
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

    // -- Scene management --------------------------------------------------

    /// Clear all visible elements from the screen.
    ///
    /// Marks every element that doesn't already have an `ended_at` with the
    /// current timeline position, so they become invisible from this point on.
    /// Call this between logical "slides" to get a clean screen.
    pub fn clear(&mut self) {
        let now = self.timeline.total_duration();
        let seg_count = self.timeline.segments().len();
        for rec in &mut self.elements {
            if rec.ended_at.is_none() {
                rec.ended_at = Some(now);
                rec.segments_at_end = Some(seg_count);
            }
        }
    }

    // -- Techniques --------------------------------------------------------

    /// Execute a composable animation technique.
    ///
    /// Records an [`Animation`](Segment::Animation) segment on the timeline and
    /// stores the technique object for frame-time evaluation. The animation
    /// targets the most recently created element.
    pub fn play(&mut self, technique: impl moron_techniques::Technique + 'static) {
        let segment_index = self.timeline.segments().len();
        let target_ids = self
            .elements
            .last()
            .map(|e| vec![e.id])
            .unwrap_or_default();
        self.timeline.add_segment(Segment::Animation {
            name: technique.name().to_string(),
            duration: technique.duration(),
        });
        self.animations.push(AnimationRecord {
            technique: Box::new(technique),
            target_ids,
            segment_index,
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

    // -- Accessors (crate-internal) ----------------------------------------

    /// Get the element metadata records (for frame state computation).
    pub(crate) fn elements(&self) -> &[ElementRecord] {
        &self.elements
    }

    /// Get the animation records (for frame state computation).
    pub(crate) fn animations(&self) -> &[AnimationRecord] {
        &self.animations
    }

    // -- Duration resolution -----------------------------------------------

    /// Return the number of narration segments in the timeline.
    ///
    /// This is the expected length of the `durations` slice passed to
    /// [`resolve_narration_durations`](Self::resolve_narration_durations).
    pub fn narration_count(&self) -> usize {
        self.timeline.narration_indices().len()
    }

    /// Replace WPM-estimated narration durations with actual TTS durations.
    ///
    /// `durations` must contain exactly one entry per narration segment, in
    /// timeline order. After updating segment durations, all element
    /// `created_at` timestamps are recomputed to reflect the new timeline.
    ///
    /// If this method is never called, the WPM estimates remain — existing
    /// behavior is fully preserved.
    pub fn resolve_narration_durations(
        &mut self,
        durations: &[f64],
    ) -> Result<(), ResolveDurationError> {
        let indices = self.timeline.narration_indices();
        if durations.len() != indices.len() {
            return Err(ResolveDurationError::LengthMismatch {
                expected: indices.len(),
                provided: durations.len(),
            });
        }

        // Update narration segment durations.
        for (&idx, &dur) in indices.iter().zip(durations.iter()) {
            self.timeline.update_segment_duration(idx, dur);
        }

        // Recompute element created_at and ended_at timestamps.
        for rec in &mut self.elements {
            rec.created_at = self.timeline.segments()[..rec.segments_at_creation]
                .iter()
                .map(|s| s.duration())
                .sum();
            if let Some(seg_count) = rec.segments_at_end {
                rec.ended_at = Some(
                    self.timeline.segments()[..seg_count]
                        .iter()
                        .map(|s| s.duration())
                        .sum(),
                );
            }
        }

        Ok(())
    }

    // -- Internal helpers --------------------------------------------------

    /// Allocate the next `Element` handle and record its metadata.
    fn mint_element_with_meta(
        &mut self,
        kind: ElementKind,
        content: String,
        items: Vec<String>,
    ) -> Element {
        let id = self.next_element_id;
        self.next_element_id += 1;

        let created_at = self.timeline.total_duration();
        let segments_at_creation = self.timeline.segments().len();

        self.elements.push(ElementRecord {
            id,
            kind,
            content,
            items,
            created_at,
            segments_at_creation,
            ended_at: None,
            segments_at_end: None,
        });

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

    // -- Duration resolution tests -----------------------------------------

    #[test]
    fn segments_at_creation_tracked() {
        let mut m = M::new();
        m.title("A");                       // 0 segments exist
        m.narrate("first");                 // adds segment 0
        m.show("B");                        // 1 segment exists
        m.narrate("second");                // adds segment 1
        m.section("C");                     // 2 segments exist

        assert_eq!(m.elements()[0].segments_at_creation, 0); // "A"
        assert_eq!(m.elements()[1].segments_at_creation, 1); // "B"
        assert_eq!(m.elements()[2].segments_at_creation, 2); // "C"
    }

    #[test]
    fn narration_count() {
        let mut m = M::new();
        assert_eq!(m.narration_count(), 0);

        m.narrate("one");
        assert_eq!(m.narration_count(), 1);

        m.beat();
        m.narrate("two");
        assert_eq!(m.narration_count(), 2);
    }

    #[test]
    fn resolve_narration_durations() {
        let mut m = M::new();
        // "hello" = 1 word -> 1*60/150 = 0.4s
        m.title("A");                       // created_at = 0.0, segments_at = 0
        m.narrate("hello");                 // segment 0: WPM est = 0.4
        m.show("B");                        // created_at = 0.4, segments_at = 1
        m.narrate("hello");                 // segment 1: WPM est = 0.4
        m.section("D");                     // created_at = 0.8, segments_at = 2

        // Verify WPM estimates
        assert!((m.elements()[0].created_at - 0.0).abs() < f64::EPSILON);
        assert!((m.elements()[1].created_at - 0.4).abs() < f64::EPSILON);
        assert!((m.elements()[2].created_at - 0.8).abs() < f64::EPSILON);
        assert!((m.timeline().total_duration() - 0.8).abs() < f64::EPSILON);

        // Resolve with actual TTS durations
        m.resolve_narration_durations(&[1.0, 2.0]).unwrap();

        // After resolution: segment 0 = 1.0, segment 1 = 2.0
        assert!((m.timeline().total_duration() - 3.0).abs() < f64::EPSILON);
        assert!((m.elements()[0].created_at - 0.0).abs() < f64::EPSILON); // before all segments
        assert!((m.elements()[1].created_at - 1.0).abs() < f64::EPSILON); // after segment 0
        assert!((m.elements()[2].created_at - 3.0).abs() < f64::EPSILON); // after segments 0+1
    }

    #[test]
    fn resolve_narration_durations_length_mismatch() {
        let mut m = M::new();
        m.narrate("A");
        m.narrate("B");

        let err = m.resolve_narration_durations(&[1.0, 2.0, 3.0]);
        assert_eq!(
            err,
            Err(ResolveDurationError::LengthMismatch {
                expected: 2,
                provided: 3,
            })
        );

        // Also test with too few
        let err = m.resolve_narration_durations(&[1.0]);
        assert_eq!(
            err,
            Err(ResolveDurationError::LengthMismatch {
                expected: 2,
                provided: 1,
            })
        );
    }

    #[test]
    fn resolve_preserves_non_narration_timing() {
        let mut m = M::new();
        m.narrate("word");                  // segment 0: WPM = 0.4
        m.wait(0.5);                        // segment 1: silence 0.5
        m.narrate("word");                  // segment 2: WPM = 0.4
        m.title("end");                     // created_at = 1.3, segments_at = 3

        m.resolve_narration_durations(&[1.0, 2.0]).unwrap();

        // Silence should be untouched
        assert!((m.timeline().segments()[1].duration() - 0.5).abs() < f64::EPSILON);
        // Total = 1.0 + 0.5 + 2.0 = 3.5
        assert!((m.timeline().total_duration() - 3.5).abs() < f64::EPSILON);
        // Element after all 3 segments: 1.0 + 0.5 + 2.0 = 3.5
        assert!((m.elements()[0].created_at - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn resolve_duration_error_display() {
        let err = ResolveDurationError::LengthMismatch {
            expected: 3,
            provided: 1,
        };
        let msg = format!("{err}");
        assert!(msg.contains("3"));
        assert!(msg.contains("1"));
        assert!(msg.contains("mismatch"));
    }

    #[test]
    fn resolve_with_zero_narrations_succeeds() {
        let mut m = M::new();
        m.wait(1.0);
        m.title("A");

        // Empty slice matches zero narrations
        m.resolve_narration_durations(&[]).unwrap();

        // Nothing changed
        assert!((m.timeline().total_duration() - 1.0).abs() < f64::EPSILON);
        assert!((m.elements()[0].created_at - 1.0).abs() < f64::EPSILON);
    }
}
