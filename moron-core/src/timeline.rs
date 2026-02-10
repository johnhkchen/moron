//! Timeline management: ordered segments, duration tracking, frame mapping.
//!
//! The `Timeline` is the backbone of video sequencing. It stores an ordered list
//! of [`Segment`]s and provides methods to query total duration, map time to
//! frame numbers, and find segments within a time range.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Segment
// ---------------------------------------------------------------------------

/// A single segment on the timeline.
#[derive(Debug, Clone)]
pub enum Segment {
    /// Narrated text synthesized via TTS.
    Narration { text: String, duration: f64 },
    /// A visual animation technique applied to elements.
    Animation { name: String, duration: f64 },
    /// A period of silence (no audio, visuals hold).
    Silence { duration: f64 },
    /// A pre-recorded audio/video clip.
    Clip { path: PathBuf, duration: f64 },
}

impl Segment {
    /// Duration of this segment in seconds.
    pub fn duration(&self) -> f64 {
        match self {
            Segment::Narration { duration, .. }
            | Segment::Animation { duration, .. }
            | Segment::Silence { duration, .. }
            | Segment::Clip { duration, .. } => *duration,
        }
    }
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

/// Default frames per second for timelines.
pub const DEFAULT_FPS: u32 = 30;

/// An ordered sequence of segments representing the full video timeline.
#[derive(Debug, Clone)]
pub struct Timeline {
    segments: Vec<Segment>,
    fps: u32,
}

impl Timeline {
    /// Create an empty timeline with the given FPS.
    pub fn new(fps: u32) -> Self {
        Self {
            segments: Vec::new(),
            fps,
        }
    }

    /// Append a segment to the end of the timeline.
    pub fn add_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// Get the ordered list of segments.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Get the configured FPS.
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Total duration of the timeline in seconds.
    pub fn total_duration(&self) -> f64 {
        self.segments.iter().map(|s| s.duration()).sum()
    }

    /// Total number of frames at the timeline's FPS.
    pub fn total_frames(&self) -> u32 {
        let dur = self.total_duration();
        if dur <= 0.0 {
            return 0;
        }
        (dur * self.fps as f64).ceil() as u32
    }

    /// Map a time (in seconds) to a frame number at the given FPS.
    ///
    /// Clamps to valid range: negative time returns 0, time beyond
    /// total duration returns the last valid frame (or 0 for empty timelines).
    pub fn frame_at(&self, time: f64) -> u32 {
        let total = self.total_frames();
        if total == 0 {
            return 0;
        }
        if time <= 0.0 {
            return 0;
        }
        let frame = (time * self.fps as f64).floor() as u32;
        frame.min(total - 1)
    }

    /// Update the duration of the segment at the given index.
    ///
    /// Returns `true` if the index was valid and the duration was updated,
    /// `false` if the index is out of bounds.
    pub fn update_segment_duration(&mut self, index: usize, duration: f64) -> bool {
        match self.segments.get_mut(index) {
            Some(seg) => {
                match seg {
                    Segment::Narration { duration: d, .. }
                    | Segment::Animation { duration: d, .. }
                    | Segment::Silence { duration: d, .. }
                    | Segment::Clip { duration: d, .. } => *d = duration,
                }
                true
            }
            None => false,
        }
    }

    /// Return the indices of all `Narration` segments, in order.
    pub fn narration_indices(&self) -> Vec<usize> {
        self.segments
            .iter()
            .enumerate()
            .filter_map(|(i, seg)| match seg {
                Segment::Narration { .. } => Some(i),
                _ => None,
            })
            .collect()
    }

    /// Find all segments that overlap the time range `[start, end)`.
    ///
    /// Returns `(segment_start_time, &Segment)` pairs for every segment
    /// whose time span intersects the query range.
    pub fn segments_in_range(&self, start: f64, end: f64) -> Vec<(f64, &Segment)> {
        let mut result = Vec::new();
        let mut cursor = 0.0_f64;

        for seg in &self.segments {
            let seg_end = cursor + seg.duration();

            // Segment [cursor, seg_end) overlaps [start, end) if:
            // cursor < end AND seg_end > start
            if cursor < end && seg_end > start {
                result.push((cursor, seg));
            }

            cursor = seg_end;

            // Early exit: past the query range
            if cursor >= end {
                break;
            }
        }

        result
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new(DEFAULT_FPS)
    }
}

// ---------------------------------------------------------------------------
// TimelineBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for constructing [`Timeline`] instances.
#[derive(Debug)]
pub struct TimelineBuilder {
    segments: Vec<Segment>,
    fps: u32,
}

impl TimelineBuilder {
    /// Start building a new timeline.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            fps: DEFAULT_FPS,
        }
    }

    /// Set the frames per second.
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Add a narration segment.
    pub fn narration(mut self, text: &str, duration: f64) -> Self {
        self.segments.push(Segment::Narration {
            text: text.to_string(),
            duration,
        });
        self
    }

    /// Add an animation segment.
    pub fn animation(mut self, name: &str, duration: f64) -> Self {
        self.segments.push(Segment::Animation {
            name: name.to_string(),
            duration,
        });
        self
    }

    /// Add a silence segment.
    pub fn silence(mut self, duration: f64) -> Self {
        self.segments.push(Segment::Silence { duration });
        self
    }

    /// Add a clip segment.
    pub fn clip(mut self, path: impl Into<PathBuf>, duration: f64) -> Self {
        self.segments.push(Segment::Clip {
            path: path.into(),
            duration,
        });
        self
    }

    /// Consume the builder and produce a [`Timeline`].
    pub fn build(self) -> Timeline {
        Timeline {
            segments: self.segments,
            fps: self.fps,
        }
    }
}

impl Default for TimelineBuilder {
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
    fn empty_timeline_has_zero_duration() {
        let tl = Timeline::default();
        assert_eq!(tl.total_duration(), 0.0);
        assert_eq!(tl.total_frames(), 0);
        assert_eq!(tl.frame_at(0.0), 0);
        assert_eq!(tl.frame_at(5.0), 0);
    }

    #[test]
    fn add_segments_and_check_duration() {
        let mut tl = Timeline::new(30);
        tl.add_segment(Segment::Narration {
            text: "Hello".into(),
            duration: 2.0,
        });
        tl.add_segment(Segment::Silence { duration: 0.5 });
        tl.add_segment(Segment::Animation {
            name: "FadeIn".into(),
            duration: 1.0,
        });

        assert!((tl.total_duration() - 3.5).abs() < f64::EPSILON);
        assert_eq!(tl.segments().len(), 3);
    }

    #[test]
    fn frame_at_basic_mapping() {
        let tl = TimelineBuilder::new()
            .fps(30)
            .narration("Test", 2.0)
            .silence(1.0)
            .build();

        // 3.0s at 30fps = 90 frames (0..89)
        assert_eq!(tl.frame_at(0.0), 0);
        assert_eq!(tl.frame_at(1.0), 30);
        assert_eq!(tl.frame_at(2.0), 60);
        assert_eq!(tl.frame_at(2.5), 75);
    }

    #[test]
    fn frame_at_60fps() {
        let tl = TimelineBuilder::new()
            .fps(60)
            .animation("FadeUp", 1.0)
            .build();

        assert_eq!(tl.frame_at(0.0), 0);
        assert_eq!(tl.frame_at(0.5), 30);
        assert_eq!(tl.frame_at(1.0), 59); // clamped to last frame
    }

    #[test]
    fn frame_at_clamps_out_of_range() {
        let tl = TimelineBuilder::new()
            .fps(30)
            .silence(1.0)
            .build();

        // 1.0s at 30fps = 30 frames (0..29)
        assert_eq!(tl.frame_at(-1.0), 0);
        assert_eq!(tl.frame_at(100.0), 29);
    }

    #[test]
    fn segments_in_range_overlap() {
        let tl = TimelineBuilder::new()
            .fps(30)
            .narration("Hello", 2.0)    // [0.0, 2.0)
            .silence(0.5)                // [2.0, 2.5)
            .animation("FadeIn", 1.0)   // [2.5, 3.5)
            .build();

        // Query [1.5, 2.5) should hit narration and silence
        let hits = tl.segments_in_range(1.5, 2.5);
        assert_eq!(hits.len(), 2);
        assert!((hits[0].0 - 0.0).abs() < f64::EPSILON); // narration starts at 0
        assert!((hits[1].0 - 2.0).abs() < f64::EPSILON); // silence starts at 2

        // Query [0.0, 0.1) should hit only narration
        let hits = tl.segments_in_range(0.0, 0.1);
        assert_eq!(hits.len(), 1);

        // Query [10.0, 20.0) should hit nothing
        let hits = tl.segments_in_range(10.0, 20.0);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn builder_produces_correct_timeline() {
        let tl = TimelineBuilder::new()
            .fps(60)
            .narration("Intro", 3.0)
            .silence(0.3)
            .animation("FadeUp", 0.5)
            .clip("/tmp/clip.mp4", 5.0)
            .build();

        assert_eq!(tl.fps(), 60);
        assert_eq!(tl.segments().len(), 4);
        assert!((tl.total_duration() - 8.8).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_default_fps() {
        let tl = TimelineBuilder::new().build();
        assert_eq!(tl.fps(), DEFAULT_FPS);
        assert_eq!(tl.fps(), 30);
    }

    #[test]
    fn segment_duration_method() {
        let narr = Segment::Narration {
            text: "Hi".into(),
            duration: 1.5,
        };
        let anim = Segment::Animation {
            name: "Slide".into(),
            duration: 0.8,
        };
        let sil = Segment::Silence { duration: 0.3 };
        let clip = Segment::Clip {
            path: "/tmp/a.wav".into(),
            duration: 4.2,
        };

        assert!((narr.duration() - 1.5).abs() < f64::EPSILON);
        assert!((anim.duration() - 0.8).abs() < f64::EPSILON);
        assert!((sil.duration() - 0.3).abs() < f64::EPSILON);
        assert!((clip.duration() - 4.2).abs() < f64::EPSILON);
    }

    #[test]
    fn total_frames_rounding() {
        // 1.0s at 30fps should give exactly 30 frames
        let tl = TimelineBuilder::new().fps(30).silence(1.0).build();
        assert_eq!(tl.total_frames(), 30);

        // 0.1s at 30fps = 3.0 frames -> ceil -> 3
        let tl = TimelineBuilder::new().fps(30).silence(0.1).build();
        assert_eq!(tl.total_frames(), 3);
    }

    #[test]
    fn update_segment_duration() {
        let mut tl = Timeline::new(30);
        tl.add_segment(Segment::Narration {
            text: "Hi".into(),
            duration: 1.0,
        });
        tl.add_segment(Segment::Silence { duration: 0.5 });
        tl.add_segment(Segment::Narration {
            text: "Bye".into(),
            duration: 1.0,
        });

        // Total starts at 2.5
        assert!((tl.total_duration() - 2.5).abs() < f64::EPSILON);

        // Update middle segment (silence) from 0.5 to 1.5
        assert!(tl.update_segment_duration(1, 1.5));
        assert!((tl.total_duration() - 3.5).abs() < f64::EPSILON);

        // Update first narration from 1.0 to 2.0
        assert!(tl.update_segment_duration(0, 2.0));
        assert!((tl.total_duration() - 4.5).abs() < f64::EPSILON);
    }

    #[test]
    fn update_segment_duration_out_of_bounds() {
        let mut tl = Timeline::new(30);
        tl.add_segment(Segment::Silence { duration: 1.0 });

        assert!(!tl.update_segment_duration(5, 2.0));
        // Timeline unchanged
        assert!((tl.total_duration() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn narration_indices_mixed() {
        let mut tl = Timeline::new(30);
        tl.add_segment(Segment::Narration {
            text: "A".into(),
            duration: 1.0,
        }); // index 0
        tl.add_segment(Segment::Silence { duration: 0.5 }); // index 1
        tl.add_segment(Segment::Narration {
            text: "B".into(),
            duration: 1.0,
        }); // index 2
        tl.add_segment(Segment::Animation {
            name: "FadeIn".into(),
            duration: 0.5,
        }); // index 3

        assert_eq!(tl.narration_indices(), vec![0, 2]);
    }

    #[test]
    fn narration_indices_empty() {
        let mut tl = Timeline::new(30);
        tl.add_segment(Segment::Silence { duration: 1.0 });
        tl.add_segment(Segment::Animation {
            name: "FadeIn".into(),
            duration: 0.5,
        });

        assert!(tl.narration_indices().is_empty());
    }
}
