//! moron-core: Scene graph, timeline, camera, Bevy integration, Chromium bridge, FFmpeg pipeline.
//!
//! This is the heart of moron — the Director layer that orchestrates the rendering pipeline.

pub mod timeline;
pub mod renderer;
pub mod chromium;
pub mod ffmpeg;
pub mod facade;
pub mod camera;
pub mod frame;

// Re-export key types at crate root for convenience.
pub use facade::{Direction, Element, M, Scene, BEAT_DURATION, BREATH_DURATION};
pub use frame::{compute_frame_state, ElementKind, ElementState, FrameState, ThemeState};
pub use moron_techniques::{Ease, Technique};
pub use moron_themes::Theme;
pub use moron_voice::Voice;
pub use renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
pub use ffmpeg::{detect_ffmpeg, encode as encode_video, EncodeConfig, FfmpegError};
pub use timeline::{Segment, Timeline, TimelineBuilder};

/// Prelude module: import everything a scene author needs in one line.
///
/// ```ignore
/// use moron_core::prelude::*;
/// ```
pub mod prelude {
    pub use moron_techniques::{Ease, Technique, TechniqueExt};
    pub use moron_themes::Theme;
    pub use moron_voice::Voice;
    pub use crate::facade::{Direction, Element, M, Scene};
    pub use crate::frame::{compute_frame_state, ElementKind, ElementState, FrameState, ThemeState};
    pub use crate::renderer::{render, RenderConfig, RenderError, RenderProgress, RenderResult};
    pub use crate::ffmpeg::{detect_ffmpeg, encode as encode_video, EncodeConfig, FfmpegError};
    pub use crate::timeline::{Segment, Timeline, TimelineBuilder};
}
