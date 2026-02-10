//! Motion techniques: SlideIn, ArcPath, Orbit, SpringPop, Parallax, etc.

use crate::technique::Technique;

/// Slides an element from one position to another.
#[derive(Debug, Clone)]
pub struct Slide {
    pub duration: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl Default for Slide {
    fn default() -> Self {
        Self {
            duration: 0.5,
            offset_x: 100.0,
            offset_y: 0.0,
        }
    }
}

impl Technique for Slide {
    fn name(&self) -> &str {
        "Slide"
    }

    fn duration(&self) -> f64 {
        self.duration
    }
}

/// Scales an element from one size to another.
#[derive(Debug, Clone)]
pub struct Scale {
    pub duration: f64,
    pub from: f64,
    pub to: f64,
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            duration: 0.4,
            from: 0.0,
            to: 1.0,
        }
    }
}

impl Technique for Scale {
    fn name(&self) -> &str {
        "Scale"
    }

    fn duration(&self) -> f64 {
        self.duration
    }
}
