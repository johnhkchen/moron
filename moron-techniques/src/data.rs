//! Data-driven techniques: BarChartRace, CountUp, Annotate, FlowDiagram, etc.

use crate::technique::Technique;

/// Animates a number counting up from a start value to an end value.
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

impl Technique for CountUp {
    fn name(&self) -> &str {
        "CountUp"
    }

    fn duration(&self) -> f64 {
        self.duration
    }
}
