//! Reveal techniques: `TypeWriter`, `SweepIn`, `PixelDissolve`, `MaskWipe`, etc.

use crate::technique::{Technique, TechniqueOutput};

/// Fades an element in from transparent to fully opaque.
#[derive(Debug, Clone)]
pub struct FadeIn {
    pub duration: f64,
}

impl Default for FadeIn {
    fn default() -> Self {
        Self { duration: 0.5 }
    }
}

impl Technique for FadeIn {
    fn name(&self) -> &'static str {
        "FadeIn"
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        TechniqueOutput {
            opacity: progress.clamp(0.0, 1.0),
            ..TechniqueOutput::default()
        }
    }
}

/// Fades an element in while translating it upward.
#[derive(Debug, Clone)]
pub struct FadeUp {
    pub duration: f64,
    pub distance: f64,
}

impl Default for FadeUp {
    fn default() -> Self {
        Self {
            duration: 0.6,
            distance: 30.0,
        }
    }
}

impl Technique for FadeUp {
    fn name(&self) -> &'static str {
        "FadeUp"
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        let p = progress.clamp(0.0, 1.0);
        TechniqueOutput {
            opacity: p,
            translate_y: self.distance * (1.0 - p),
            ..TechniqueOutput::default()
        }
    }
}
