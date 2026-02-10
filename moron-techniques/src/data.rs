//! Data-driven techniques: BarChartRace, CountUp, Annotate, FlowDiagram, etc.

use crate::technique::{Technique, TechniqueOutput};

/// Animates a number counting up from a start value to an end value.
///
/// The interpolated value is exposed via `current_value()`. The `apply()`
/// method maps the value to opacity as a normalized 0-1 progress indicator.
#[derive(Debug, Clone)]
pub struct CountUp {
    pub duration: f64,
    pub from: f64,
    pub to: f64,
}

impl Default for CountUp {
    fn default() -> Self {
        Self {
            duration: 1.0,
            from: 0.0,
            to: 100.0,
        }
    }
}

impl CountUp {
    /// Get the interpolated value at the given progress (0.0 to 1.0).
    pub fn current_value(&self, progress: f64) -> f64 {
        let p = progress.clamp(0.0, 1.0);
        self.from + (self.to - self.from) * p
    }
}

impl Technique for CountUp {
    fn name(&self) -> &str {
        "CountUp"
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn apply(&self, progress: f64) -> TechniqueOutput {
        let p = progress.clamp(0.0, 1.0);
        TechniqueOutput {
            opacity: p,
            ..TechniqueOutput::default()
        }
    }
}
