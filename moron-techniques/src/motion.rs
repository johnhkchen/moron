//! Motion techniques: `SlideIn`, `ArcPath`, Orbit, `SpringPop`, Parallax, etc.

use crate::technique::{Technique, TechniqueOutput};

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
    fn name(&self) -> &'static str {
        "Slide"
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        let p = progress.clamp(0.0, 1.0);
        TechniqueOutput {
            translate_x: self.offset_x * (1.0 - p),
            translate_y: self.offset_y * (1.0 - p),
            ..TechniqueOutput::default()
        }
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
    fn name(&self) -> &'static str {
        "Scale"
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        let p = progress.clamp(0.0, 1.0);
        TechniqueOutput {
            scale: self.from + (self.to - self.from) * p,
            ..TechniqueOutput::default()
        }
    }
}
