//! moron-techniques: Composable animation techniques for motion graphics.
//!
//! Each technique is a Rust struct implementing the Technique trait.
//! ~30 techniques across categories: reveals, motion, morphing, staging, emphasis, camera, transitions, data.

pub mod technique;
pub mod reveals;
pub mod motion;
pub mod staging;
pub mod emphasis;
pub mod camera;
pub mod transitions;
pub mod data;
