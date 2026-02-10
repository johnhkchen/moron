//! Reveal techniques: TypeWriter, SweepIn, PixelDissolve, MaskWipe, etc.

use crate::technique::Technique;

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
    fn name(&self) -> &str {
        "FadeIn"
    }

    fn duration(&self) -> f64 {
        self.duration
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
    fn name(&self) -> &str {
        "FadeUp"
    }

    fn duration(&self) -> f64 {
        self.duration
    }
}
