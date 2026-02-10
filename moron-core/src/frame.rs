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
// ItemState — per-item visual snapshot (for Steps elements)
// ---------------------------------------------------------------------------

/// The visual state of a single item within a Steps element.
///
/// Carries per-item text alongside individual transforms, enabling Stagger
/// animations to animate each bullet independently.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemState {
    /// Item text content.
    pub text: String,
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

impl ItemState {
    /// Create an ItemState with default (fully visible) transforms.
    fn new(text: String) -> Self {
        Self {
            text,
            opacity: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }
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
    /// List items with per-item visual state (non-empty only for Steps elements).
    pub items: Vec<ItemState>,
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
    /// Vertical layout position: 0.0 = top of frame, 0.5 = center, 1.0 = bottom.
    /// Computed automatically based on element kind and co-visible elements.
    pub layout_y: f64,
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
    let mut elements: Vec<ElementState> = m
        .elements()
        .iter()
        .map(|rec| {
            let visible = rec.created_at <= clamped_time
                && rec.ended_at.is_none_or(|end| clamped_time < end);
            ElementState {
                id: rec.id,
                kind: rec.kind.clone(),
                content: rec.content.clone(),
                items: rec.items.iter().map(|t| ItemState::new(t.clone())).collect(),
                visible,
                // Default visual state — overwritten by apply_animations() below.
                opacity: if visible { 1.0 } else { 0.0 },
                translate_x: 0.0,
                translate_y: 0.0,
                scale: if visible { 1.0 } else { 0.0 },
                rotation: 0.0,
                layout_y: 0.5,
            }
        })
        .collect();

    // Apply animation technique outputs to elements.
    apply_animations(m, clamped_time, &mut elements);

    // Assign vertical layout positions based on visible element composition.
    assign_layout_positions(&mut elements);

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

/// Apply animation technique outputs to elements based on the current time.
///
/// For each animation record, computes the animation's progress within its
/// timeline segment and applies the technique's visual output to target elements.
fn apply_animations(m: &M, time: f64, elements: &mut [ElementState]) {
    // Build ID → index lookup for O(1) element access.
    let id_to_index: HashMap<u64, usize> = elements
        .iter()
        .enumerate()
        .map(|(i, e)| (e.id, i))
        .collect();

    let segments = m.timeline().segments();

    for record in m.animations() {
        // Compute the animation segment's absolute time window.
        let seg_start: f64 = segments[..record.segment_index]
            .iter()
            .map(|s| s.duration())
            .sum();
        let seg_duration = segments[record.segment_index].duration();
        let seg_end = seg_start + seg_duration;

        // Compute progress: 0.0 before start, 1.0 after end, linear during.
        let progress = if time < seg_start {
            0.0
        } else if time >= seg_end || seg_duration <= 0.0 {
            1.0
        } else {
            (time - seg_start) / seg_duration
        };

        // Apply to each target element (only if visible).
        for &target_id in &record.target_ids {
            if let Some(&idx) = id_to_index.get(&target_id)
                && elements[idx].visible
            {
                let item_count = elements[idx].items.len();
                if item_count > 0 {
                    // Steps element: apply per-item transforms via apply_items().
                    let outputs = record.technique.apply_items(item_count, progress);
                    for (i, item_output) in outputs.into_iter().enumerate() {
                        if i < elements[idx].items.len() {
                            elements[idx].items[i].opacity = item_output.opacity;
                            elements[idx].items[i].translate_x = item_output.translate_x;
                            elements[idx].items[i].translate_y = item_output.translate_y;
                            elements[idx].items[i].scale = item_output.scale;
                            elements[idx].items[i].rotation = item_output.rotation;
                        }
                    }
                    // Keep element-level transforms at defaults so the wrapper
                    // doesn't double-apply what's already on each item.
                } else {
                    // Non-Steps element: apply element-level transforms.
                    let output = record.technique.apply(progress);
                    elements[idx].opacity = output.opacity;
                    elements[idx].translate_x = output.translate_x;
                    elements[idx].translate_y = output.translate_y;
                    elements[idx].scale = output.scale;
                    elements[idx].rotation = output.rotation;
                }
            }
        }
    }
}

/// Returns true for element kinds that act as headers (Title, Section).
fn is_header(kind: &ElementKind) -> bool {
    matches!(kind, ElementKind::Title | ElementKind::Section)
}

/// Assign vertical layout positions to elements based on kind and co-visibility.
///
/// Headers (Title, Section) sort before bodies (Show, Steps, Metric).
/// Within each group, creation order is preserved. Positions are assigned:
/// - 1 visible element: centered at 0.5
/// - 2 visible elements: 0.3 and 0.65
/// - 3+ visible elements: evenly spaced from 0.2 to 0.8
fn assign_layout_positions(elements: &mut [ElementState]) {
    // Collect indices of visible elements, partitioned into headers-first order.
    let mut header_indices: Vec<usize> = Vec::new();
    let mut body_indices: Vec<usize> = Vec::new();

    for (i, el) in elements.iter().enumerate() {
        if el.visible {
            if is_header(&el.kind) {
                header_indices.push(i);
            } else {
                body_indices.push(i);
            }
        }
    }

    // Merge: headers first, then bodies (preserving creation order within each group).
    let mut sorted_indices: Vec<usize> = Vec::with_capacity(header_indices.len() + body_indices.len());
    sorted_indices.extend(header_indices);
    sorted_indices.extend(body_indices);

    let count = sorted_indices.len();
    if count == 0 {
        return;
    }

    // Compute layout_y positions based on visible element count.
    let positions: Vec<f64> = match count {
        1 => vec![0.5],
        2 => vec![0.3, 0.65],
        n => {
            // Evenly distribute from 0.2 to 0.8.
            (0..n)
                .map(|i| {
                    if n == 1 {
                        0.5
                    } else {
                        0.2 + (0.6 * i as f64 / (n - 1) as f64)
                    }
                })
                .collect()
        }
    };

    for (slot, &elem_idx) in sorted_indices.iter().enumerate() {
        elements[elem_idx].layout_y = positions[slot];
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
        let item_texts: Vec<&str> = fs.elements[0].items.iter().map(|i| i.text.as_str()).collect();
        assert_eq!(item_texts, vec!["one", "two", "three"]);
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

    #[test]
    fn two_themes_produce_different_frame_state() {
        use moron_themes::Theme;

        // Dark theme (default)
        let m_dark = M::new();
        let fs_dark = compute_frame_state(&m_dark, 0.0);

        // Light theme
        let mut m_light = M::new();
        m_light.theme(Theme::light());
        let fs_light = compute_frame_state(&m_light, 0.0);

        // Theme names must differ.
        assert_eq!(fs_dark.theme.name, "moron-dark");
        assert_eq!(fs_light.theme.name, "moron-light");
        assert_ne!(fs_dark.theme.name, fs_light.theme.name);

        // Both must have all 56 CSS properties.
        assert_eq!(fs_dark.theme.css_properties.len(), 56);
        assert_eq!(fs_light.theme.css_properties.len(), 56);

        // At least the background and foreground colors must differ.
        assert_ne!(
            fs_dark.theme.css_properties.get("--moron-bg-primary"),
            fs_light.theme.css_properties.get("--moron-bg-primary"),
            "bg-primary must differ between dark and light themes"
        );
        assert_ne!(
            fs_dark.theme.css_properties.get("--moron-fg-primary"),
            fs_light.theme.css_properties.get("--moron-fg-primary"),
            "fg-primary must differ between dark and light themes"
        );

        // The full ThemeState structs must differ.
        assert_ne!(fs_dark.theme, fs_light.theme);

        // JSON output must differ.
        let json_dark = serde_json::to_string(&fs_dark.theme).unwrap();
        let json_light = serde_json::to_string(&fs_light.theme).unwrap();
        assert_ne!(json_dark, json_light, "Theme JSON must differ between dark and light");
    }

    // -- Layout tests ------------------------------------------------------

    #[test]
    fn layout_single_element_centered() {
        let mut m = M::new();
        m.title("Solo Title");

        let fs = compute_frame_state(&m, 0.0);
        assert!((fs.elements[0].layout_y - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn layout_title_plus_show() {
        let mut m = M::new();
        m.title("Heading");
        m.show("Body text");

        let fs = compute_frame_state(&m, 0.0);
        let title = fs.elements.iter().find(|e| e.content == "Heading").unwrap();
        let show = fs.elements.iter().find(|e| e.content == "Body text").unwrap();

        // Title (header) should be above Show (body).
        assert!((title.layout_y - 0.3).abs() < f64::EPSILON, "title layout_y: {}", title.layout_y);
        assert!((show.layout_y - 0.65).abs() < f64::EPSILON, "show layout_y: {}", show.layout_y);
    }

    #[test]
    fn layout_section_plus_steps() {
        let mut m = M::new();
        m.section("Features");
        m.steps(&["fast", "offline", "simple"]);

        let fs = compute_frame_state(&m, 0.0);
        let section = fs.elements.iter().find(|e| e.content == "Features").unwrap();
        let steps = fs.elements.iter().find(|e| e.kind == ElementKind::Steps { count: 3 }).unwrap();

        assert!((section.layout_y - 0.3).abs() < f64::EPSILON);
        assert!((steps.layout_y - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn layout_three_elements() {
        let mut m = M::new();
        m.section("Intro");
        m.show("Detail");
        m.steps(&["a", "b"]);

        let fs = compute_frame_state(&m, 0.0);
        let section = fs.elements.iter().find(|e| e.content == "Intro").unwrap();
        let show = fs.elements.iter().find(|e| e.content == "Detail").unwrap();
        let steps = fs.elements.iter().find(|e| e.kind == ElementKind::Steps { count: 2 }).unwrap();

        // Header first, then two bodies. 3 elements: 0.2, 0.5, 0.8.
        assert!((section.layout_y - 0.2).abs() < f64::EPSILON, "section: {}", section.layout_y);
        assert!((show.layout_y - 0.5).abs() < f64::EPSILON, "show: {}", show.layout_y);
        assert!((steps.layout_y - 0.8).abs() < f64::EPSILON, "steps: {}", steps.layout_y);
    }

    #[test]
    fn layout_after_clear_recenters() {
        let mut m = M::new();
        m.title("Slide 1");
        m.show("Detail");
        m.wait(1.0);
        m.clear();
        m.title("Slide 2 Solo");

        // At t=1.5 only "Slide 2 Solo" is visible.
        let fs = compute_frame_state(&m, 1.5);
        let visible: Vec<_> = fs.elements.iter().filter(|e| e.visible).collect();
        assert_eq!(visible.len(), 1);
        assert!((visible[0].layout_y - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn layout_hidden_elements_ignored() {
        let mut m = M::new();
        m.title("Early");
        m.wait(1.0);
        m.show("Late"); // created at t=1.0

        // At t=0.5 only "Early" is visible.
        let fs = compute_frame_state(&m, 0.5);
        let early = fs.elements.iter().find(|e| e.content == "Early").unwrap();
        assert!(early.visible);
        assert!((early.layout_y - 0.5).abs() < f64::EPSILON, "solo visible should center");
    }

    #[test]
    fn layout_two_body_elements() {
        let mut m = M::new();
        m.show("First");
        m.show("Second");

        let fs = compute_frame_state(&m, 0.0);
        let first = &fs.elements[0];
        let second = &fs.elements[1];

        // Two bodies: 0.3, 0.65.
        assert!((first.layout_y - 0.3).abs() < f64::EPSILON);
        assert!((second.layout_y - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn layout_y_serializes_as_camel_case() {
        let mut m = M::new();
        m.title("Test");

        let fs = compute_frame_state(&m, 0.0);
        let value: serde_json::Value = serde_json::to_value(&fs).unwrap();
        let elem = &value["elements"][0];
        assert!(elem.get("layoutY").is_some(), "layout_y should serialize as layoutY");
    }

    #[test]
    fn layout_empty_scene() {
        let m = M::new();
        let fs = compute_frame_state(&m, 0.0);
        assert!(fs.elements.is_empty());
    }

    // -- Animation execution tests -----------------------------------------

    #[test]
    fn animation_fade_in_progress() {
        use moron_techniques::FadeIn;

        let mut m = M::new();
        m.title("Hello");                     // element 0, created at t=0
        m.play(FadeIn { duration: 1.0 });     // animation segment [0.0, 1.0)

        // At start of animation: progress=0.0, opacity=0.0
        let fs = compute_frame_state(&m, 0.0);
        assert!(fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 0.0).abs() < f64::EPSILON);

        // At midpoint: progress=0.5, opacity=0.5
        let fs = compute_frame_state(&m, 0.5);
        assert!((fs.elements[0].opacity - 0.5).abs() < f64::EPSILON);

        // At end of animation: progress=1.0, opacity=1.0
        let fs = compute_frame_state(&m, 1.0);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_before_start_shows_initial_state() {
        use moron_techniques::FadeIn;

        let mut m = M::new();
        m.title("Hello");
        m.wait(1.0);                          // silence [0.0, 1.0)
        m.play(FadeIn { duration: 0.5 });     // animation [1.0, 1.5)

        // At t=0.5, animation hasn't started → apply(0.0) → opacity=0.0
        let fs = compute_frame_state(&m, 0.5);
        assert!(fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_after_end_shows_final_state() {
        use moron_techniques::FadeIn;

        let mut m = M::new();
        m.title("Hello");
        m.play(FadeIn { duration: 0.5 });     // animation [0.0, 0.5)
        m.wait(1.0);                           // silence [0.5, 1.5)

        // At t=1.0, animation completed → apply(1.0) → opacity=1.0
        let fs = compute_frame_state(&m, 1.0);
        assert!(fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
        assert!((fs.elements[0].scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn no_animation_retains_defaults() {
        let mut m = M::new();
        m.title("Hello");
        m.wait(1.0);

        let fs = compute_frame_state(&m, 0.5);
        assert!(fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
        assert!((fs.elements[0].scale - 1.0).abs() < f64::EPSILON);
        assert!((fs.elements[0].translate_x - 0.0).abs() < f64::EPSILON);
        assert!((fs.elements[0].translate_y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_fade_up_produces_translation() {
        use moron_techniques::FadeUp;

        let mut m = M::new();
        m.title("Hello");
        m.play(FadeUp { duration: 1.0, distance: 30.0 });

        // At midpoint: opacity=0.5, translate_y=15.0
        let fs = compute_frame_state(&m, 0.5);
        assert!((fs.elements[0].opacity - 0.5).abs() < f64::EPSILON);
        assert!((fs.elements[0].translate_y - 15.0).abs() < f64::EPSILON);

        // At end: opacity=1.0, translate_y=0.0
        let fs = compute_frame_state(&m, 1.0);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
        assert!((fs.elements[0].translate_y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_not_applied_to_invisible_element() {
        use moron_techniques::FadeIn;

        let mut m = M::new();
        m.title("Hello");
        m.clear();
        m.play(FadeIn { duration: 0.5 });
        m.wait(1.0);

        // Element cleared before animation — stays at invisible defaults
        let fs = compute_frame_state(&m, 0.25);
        assert!(!fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 0.0).abs() < f64::EPSILON);
        assert!((fs.elements[0].scale - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animation_with_preceding_segments() {
        use moron_techniques::FadeIn;

        let mut m = M::new();
        m.narrate("Hello world");             // 0.8s narration [0.0, 0.8)
        m.title("Title");                     // element 0, created at t=0.8
        m.play(FadeIn { duration: 1.0 });     // animation [0.8, 1.8)
        m.wait(1.0);                          // silence [1.8, 2.8)

        // At t=0.8 (animation start): progress=0.0, opacity=0.0
        let fs = compute_frame_state(&m, 0.8);
        assert!(fs.elements[0].visible);
        assert!((fs.elements[0].opacity - 0.0).abs() < f64::EPSILON);

        // At t=1.3 (midpoint): progress=0.5, opacity=0.5
        let fs = compute_frame_state(&m, 1.3);
        assert!((fs.elements[0].opacity - 0.5).abs() < f64::EPSILON);

        // At t=2.0 (after animation): progress=1.0, opacity=1.0
        let fs = compute_frame_state(&m, 2.0);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON);
    }

    // -- Per-item animation tests (Stagger + Steps) -------------------------

    #[test]
    fn stagger_steps_per_item_opacity() {
        use moron_techniques::{FadeIn, Stagger};

        let mut m = M::new();
        m.steps(&["a", "b", "c"]);
        // Stagger with FadeIn: inner=0.5s, delay=0.1, count doesn't matter
        // (apply_items receives count=3 from element's items).
        // Total stagger duration for 3 items = 0.5 + 0.1*(3-1) = 0.7s
        m.play(Stagger::new(FadeIn { duration: 0.5 }).with_delay(0.1));

        // At start (t=0, progress=0): all items at apply(0) → opacity=0
        let fs = compute_frame_state(&m, 0.0);
        assert_eq!(fs.elements[0].items.len(), 3);
        for item in &fs.elements[0].items {
            assert!((item.opacity - 0.0).abs() < f64::EPSILON, "item opacity at start: {}", item.opacity);
        }
    }

    #[test]
    fn stagger_steps_first_item_animates_before_last() {
        use moron_techniques::{FadeIn, Stagger};

        let mut m = M::new();
        m.steps(&["a", "b", "c"]);
        // Total duration for 3 items: 0.5 + 0.1*2 = 0.7s
        m.play(Stagger::new(FadeIn { duration: 0.5 }).with_delay(0.1));

        // At t=0.35 (progress=0.5): first item well into animation, last item barely started
        let fs = compute_frame_state(&m, 0.35);
        let first = &fs.elements[0].items[0];
        let last = &fs.elements[0].items[2];
        assert!(first.opacity > last.opacity, "first item ({}) should be more opaque than last ({})", first.opacity, last.opacity);
    }

    #[test]
    fn stagger_steps_end_state() {
        use moron_techniques::{FadeIn, Stagger};

        let mut m = M::new();
        m.steps(&["a", "b", "c"]);
        // Duration: 0.5 + 0.1*2 = 0.7s
        let stagger = Stagger::new(FadeIn { duration: 0.5 }).with_delay(0.1);
        m.play(stagger);
        m.wait(1.0);

        // At t=1.0 (well past animation end): all items fully opaque
        let fs = compute_frame_state(&m, 1.0);
        for item in &fs.elements[0].items {
            assert!((item.opacity - 1.0).abs() < f64::EPSILON, "item opacity at end: {}", item.opacity);
        }
    }

    #[test]
    fn stagger_steps_element_level_stays_default() {
        use moron_techniques::{FadeIn, Stagger};

        let mut m = M::new();
        m.steps(&["a", "b"]);
        m.play(Stagger::new(FadeIn { duration: 0.5 }).with_delay(0.1));

        // At midpoint: element-level opacity should remain 1.0
        // (per-item transforms handle the animation, not element-level)
        let fs = compute_frame_state(&m, 0.25);
        assert!((fs.elements[0].opacity - 1.0).abs() < f64::EPSILON,
            "element-level opacity should stay 1.0, got: {}", fs.elements[0].opacity);
    }

    #[test]
    fn non_stagger_steps_all_items_same() {
        use moron_techniques::FadeIn;

        let mut m = M::new();
        m.steps(&["a", "b", "c"]);
        m.play(FadeIn { duration: 1.0 });

        // At midpoint: FadeIn's apply_items default gives all items same opacity
        let fs = compute_frame_state(&m, 0.5);
        for item in &fs.elements[0].items {
            assert!((item.opacity - 0.5).abs() < f64::EPSILON,
                "non-stagger items should all have same opacity: {}", item.opacity);
        }
    }

    #[test]
    fn item_state_default_transforms() {
        let mut m = M::new();
        m.steps(&["x", "y"]);

        // No animation: items should have default transforms
        let fs = compute_frame_state(&m, 0.0);
        for item in &fs.elements[0].items {
            assert!((item.opacity - 1.0).abs() < f64::EPSILON);
            assert!((item.scale - 1.0).abs() < f64::EPSILON);
            assert!((item.translate_x - 0.0).abs() < f64::EPSILON);
            assert!((item.translate_y - 0.0).abs() < f64::EPSILON);
            assert!((item.rotation - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn item_state_serializes_as_camel_case() {
        let mut m = M::new();
        m.steps(&["a"]);

        let fs = compute_frame_state(&m, 0.0);
        let value: serde_json::Value = serde_json::to_value(&fs).unwrap();
        let items = value["elements"][0]["items"].as_array().unwrap();
        let item = &items[0];
        assert!(item.get("text").is_some(), "should have 'text' field");
        assert!(item.get("translateX").is_some(), "should have 'translateX' field");
        assert!(item.get("translateY").is_some(), "should have 'translateY' field");
    }
}
