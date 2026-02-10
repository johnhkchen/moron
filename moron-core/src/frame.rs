//! Frame state computation: the Rust-to-React JSON contract.
//!
//! [`FrameState`] captures the complete visual state at a given point in the timeline.
//! Rust computes it, serializes to JSON, and React renders it.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::facade::M;
use crate::timeline::Segment;

// ---------------------------------------------------------------------------
// ElementKind — what type of visual element
// ---------------------------------------------------------------------------

/// The structural type of a visual element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ElementKind {
    /// A title card.
    Title,
    /// Arbitrary display text.
    Show,
    /// A named section heading.
    Section,
    /// A metric with a directional indicator.
    Metric {
        /// Direction string: "up", "down", or "neutral".
        direction: String,
    },
    /// A list of items revealed with staggered timing.
    Steps {
        /// Number of items in the list.
        count: usize,
    },
}

// ---------------------------------------------------------------------------
// ElementState — per-element visual snapshot
// ---------------------------------------------------------------------------

/// The visual state of a single element at a point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementState {
    /// Unique element identifier.
    pub id: u64,
    /// Structural type of this element.
    pub kind: ElementKind,
    /// Primary text content.
    pub content: String,
    /// List items (non-empty only for Steps elements).
    pub items: Vec<String>,
    /// Whether this element is currently visible.
    pub visible: bool,
    /// Opacity (0.0 = transparent, 1.0 = fully opaque).
    pub opacity: f64,
    /// Horizontal translation in pixels.
    pub translate_x: f64,
    /// Vertical translation in pixels.
    pub translate_y: f64,
    /// Scale factor (1.0 = normal size).
    pub scale: f64,
    /// Rotation in degrees.
    pub rotation: f64,
}

// ---------------------------------------------------------------------------
// ThemeState — theme as CSS custom properties
// ---------------------------------------------------------------------------

/// Theme snapshot serialized as CSS custom property pairs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeState {
    /// Theme name.
    pub name: String,
    /// CSS custom properties (`--moron-*` key-value pairs).
    pub css_properties: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// FrameState — the complete visual state at a timestamp
// ---------------------------------------------------------------------------

/// Complete visual state at a given point in the timeline.
///
/// This is the data contract between Rust and React. Rust computes a `FrameState`
/// for each frame, serializes it to JSON, and React renders it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameState {
    /// Current time in seconds.
    pub time: f64,
    /// Current frame number (0-indexed).
    pub frame: u32,
    /// Total duration of the timeline in seconds.
    pub total_duration: f64,
    /// Frames per second.
    pub fps: u32,
    /// Visual state of all elements (both visible and hidden).
    pub elements: Vec<ElementState>,
    /// Text of the currently active narration, if any.
    pub active_narration: Option<String>,
    /// Current theme as CSS custom properties.
    pub theme: ThemeState,
}

// ---------------------------------------------------------------------------
// Computation
// ---------------------------------------------------------------------------

/// Compute the complete visual state at the given timestamp.
///
/// Walks the scene's element records and timeline to determine:
/// - Which elements are visible (created_at <= time)
/// - Default visual properties for visible elements
/// - Active narration text from overlapping narration segments
/// - Current theme as CSS properties
pub fn compute_frame_state(m: &M, time: f64) -> FrameState {
    let total_duration = m.timeline().total_duration();
    let fps = m.timeline().fps();
    let clamped_time = time.clamp(0.0, total_duration);
    let frame = m.timeline().frame_at(clamped_time);

    // Build element states from M's element records.
    let elements: Vec<ElementState> = m
        .elements()
        .iter()
        .map(|rec| {
            let visible = rec.created_at <= clamped_time;
            ElementState {
                id: rec.id,
                kind: rec.kind.clone(),
                content: rec.content.clone(),
                items: rec.items.clone(),
                visible,
                // Default visual state for visible elements.
                // Animation techniques will modify these in future tickets.
                opacity: if visible { 1.0 } else { 0.0 },
                translate_x: 0.0,
                translate_y: 0.0,
                scale: if visible { 1.0 } else { 0.0 },
                rotation: 0.0,
            }
        })
        .collect();

    // Find active narration: any Narration segment overlapping the current time.
    // Use a tiny epsilon window around the current time for point-in-time query.
    let epsilon = 1.0 / fps as f64 / 2.0;
    let active_narration = find_active_narration(m, clamped_time, epsilon);

    // Build theme state from current theme.
    let theme_ref = m.current_theme();
    let css_props: HashMap<String, String> = theme_ref
        .to_css_properties()
        .into_iter()
        .collect();
    let theme = ThemeState {
        name: theme_ref.name.clone(),
        css_properties: css_props,
    };

    FrameState {
        time: clamped_time,
        frame,
        total_duration,
        fps,
        elements,
        active_narration,
        theme,
    }
}

/// Find the text of the active narration segment at the given time.
fn find_active_narration(m: &M, time: f64, epsilon: f64) -> Option<String> {
    let hits = m.timeline().segments_in_range(time, time + epsilon);
    for (_start, segment) in hits {
        if let Segment::Narration { text, .. } = segment {
            return Some(text.clone());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::{Direction, M};

    #[test]
    fn empty_scene_frame_state() {
        let m = M::new();
        let fs = compute_frame_state(&m, 0.0);

        assert_eq!(fs.time, 0.0);
        assert_eq!(fs.frame, 0);
        assert_eq!(fs.total_duration, 0.0);
        assert_eq!(fs.fps, 30);
        assert!(fs.elements.is_empty());
        assert!(fs.active_narration.is_none());
        assert_eq!(fs.theme.name, "moron-dark");
        assert!(!fs.theme.css_properties.is_empty());
    }

    #[test]
    fn elements_visible_at_creation_time() {
        let mut m = M::new();
        // Elements created at t=0 (before any segments advance the timeline)
        let _e1 = m.title("Hello");
        let _e2 = m.show("World");

        let fs = compute_frame_state(&m, 0.0);

        assert_eq!(fs.elements.len(), 2);
        assert!(fs.elements[0].visible);
        assert!(fs.elements[1].visible);
        assert_eq!(fs.elements[0].content, "Hello");
        assert_eq!(fs.elements[1].content, "World");
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn elements_not_visible_before_creation() {
        let mut m = M::new();
        // Push a 2-second narration first
        m.narrate("Some narration here");
        // Then create an element — it will be created at t=2.4 (after narration)
        let _e1 = m.title("After Narration");

        // At t=0 the element hasn't been created yet
        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.elements.len(), 1);
        assert!(!fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 0.0).abs() < f64::EPSILON);

        // At t=3.0 the element should be visible
        let fs = compute_frame_state(&m, 3.0);
        assert!(fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn active_narration_during_segment() {
        let mut m = M::new();
        m.narrate("Hello world"); // ~0.8s

        // At t=0.0, narration should be active
        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.active_narration, Some("Hello world".to_string()));

        // At t=0.4, narration should still be active
        let fs = compute_frame_state(&m, 0.4);
        assert_eq!(fs.active_narration, Some("Hello world".to_string()));
    }

    #[test]
    fn no_narration_during_silence() {
        let mut m = M::new();
        m.narrate("Hello world"); // ~0.8s
        m.wait(2.0); // silence from 0.8 to 2.8

        // At t=1.5, we're in the silence segment
        let fs = compute_frame_state(&m, 1.5);
        assert!(fs.active_narration.is_none());
    }

    #[test]
    fn frame_state_serializes_to_json() {
        let mut m = M::new();
        m.title("Test Title");
        m.narrate("Some text");

        let fs = compute_frame_state(&m, 0.0);
        let json = serde_json::to_string(&fs);
        assert!(json.is_ok(), "FrameState must serialize to JSON");

        let json_str = json.unwrap();
        assert!(!json_str.is_empty());
    }

    #[test]
    fn frame_state_json_has_camel_case_keys() {
        let mut m = M::new();
        m.title("Test");

        let fs = compute_frame_state(&m, 0.0);
        let value: serde_json::Value = serde_json::to_value(&fs).unwrap();

        // Top-level keys should be camelCase
        assert!(value.get("totalDuration").is_some());
        assert!(value.get("activeNarration").is_some());
        assert!(value.get("fps").is_some());

        // Element keys should be camelCase
        let elements = value.get("elements").unwrap().as_array().unwrap();
        let elem = &elements[0];
        assert!(elem.get("translateX").is_some());
        assert!(elem.get("translateY").is_some());
    }

    #[test]
    fn frame_state_json_round_trip() {
        let mut m = M::new();
        m.title("Round Trip");
        m.show("Content");

        let fs = compute_frame_state(&m, 0.0);
        let json = serde_json::to_string(&fs).unwrap();
        let deserialized: FrameState = serde_json::from_str(&json).unwrap();
        assert_eq!(fs, deserialized);
    }

    #[test]
    fn metric_element_preserves_direction() {
        let mut m = M::new();
        m.metric("Revenue", "$1M", Direction::Up);

        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.elements.len(), 1);
        assert_eq!(
            fs.elements[0].kind,
            ElementKind::Metric {
                direction: "up".to_string()
            }
        );
        assert_eq!(fs.elements[0].content, "Revenue: $1M");
    }

    #[test]
    fn steps_element_preserves_items() {
        let mut m = M::new();
        m.steps(&["one", "two", "three"]);

        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.elements.len(), 1);
        assert_eq!(
            fs.elements[0].kind,
            ElementKind::Steps { count: 3 }
        );
        assert_eq!(
            fs.elements[0].items,
            vec!["one".to_string(), "two".to_string(), "three".to_string()]
        );
    }

    #[test]
    fn frame_number_computed_correctly() {
        let mut m = M::new();
        m.wait(2.0); // 2 seconds at 30fps = 60 frames (0..59)

        let fs = compute_frame_state(&m, 1.0);
        assert_eq!(fs.frame, 30); // 1.0s * 30fps = frame 30

        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.frame, 0);
    }

    #[test]
    fn time_clamped_to_valid_range() {
        let mut m = M::new();
        m.wait(1.0);

        // Negative time clamped to 0
        let fs = compute_frame_state(&m, -5.0);
        assert_eq!(fs.time, 0.0);

        // Time beyond duration clamped
        let fs = compute_frame_state(&m, 100.0);
        assert!((fs.time - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn theme_state_contains_css_properties() {
        let m = M::new();
        let fs = compute_frame_state(&m, 0.0);

        assert_eq!(fs.theme.name, "moron-dark");
        assert!(fs.theme.css_properties.contains_key("--moron-bg-primary"));
        assert!(fs.theme.css_properties.contains_key("--moron-accent"));
        assert!(fs.theme.css_properties.len() >= 50);
    }

    #[test]
    fn section_element_kind() {
        let mut m = M::new();
        m.section("Part 1");

        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.elements[0].kind, ElementKind::Section);
        assert_eq!(fs.elements[0].content, "Part 1");
    }

    #[test]
    fn multiple_elements_with_timing() {
        let mut m = M::new();
        m.title("Intro"); // created at t=0
        m.wait(1.0);
        m.show("Detail"); // created at t=1.0
        m.wait(1.0);
        m.section("Part 2"); // created at t=2.0

        // At t=0.5: only Intro visible
        let fs = compute_frame_state(&m, 0.5);
        let visible: Vec<_> = fs.elements.iter().filter(|e| e.visible).collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].content, "Intro");

        // At t=1.5: Intro and Detail visible
        let fs = compute_frame_state(&m, 1.5);
        let visible: Vec<_> = fs.elements.iter().filter(|e| e.visible).collect();
        assert_eq!(visible.len(), 2);

        // At t=2.5: all three visible
        let fs = compute_frame_state(&m, 2.5);
        let visible: Vec<_> = fs.elements.iter().filter(|e| e.visible).collect();
        assert_eq!(visible.len(), 3);
    }
}
