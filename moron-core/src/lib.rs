//! moron-core: Scene graph, timeline, camera, Bevy integration, Chromium bridge, FFmpeg pipeline.
//!
//! This is the heart of moron — the Director layer that orchestrates the rendering pipeline.

pub mod timeline;
pub mod renderer;
pub mod chromium;
pub mod ffmpeg;
pub mod facade;
pub mod camera;

// Re-export facade types at crate root for convenience.
pub use facade::{Direction, Element, M, Scene, Technique, Theme, Voice};
