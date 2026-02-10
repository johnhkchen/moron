//! TTS backend trait: common interface for all voice synthesis providers.

use std::path::PathBuf;

use crate::audio::AudioClip;

/// Common trait for all TTS synthesis backends.
pub trait VoiceBackend {
    /// Synthesize the given text into an audio clip.
    fn synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error>;

    /// Human-readable name of this backend (e.g. "kokoro", "piper").
    fn name(&self) -> &str;
}

/// Which backend implementation to use for voice synthesis.
#[derive(Debug, Clone)]
pub enum VoiceBackendType {
    /// Kokoro — primary local TTS engine.
    Kokoro,
    /// Piper — fallback local TTS engine.
    Piper,
    /// Remote API provider identified by name (e.g. "elevenlabs").
    ApiProvider(String),
    /// Pre-recorded audio file on disk.
    PreRecorded(PathBuf),
}

/// Configuration for a voice: which backend to use and synthesis parameters.
#[derive(Debug, Clone)]
pub struct Voice {
    /// The backend type to use for synthesis.
    pub backend_type: VoiceBackendType,
    /// Speech speed multiplier (1.0 = normal).
    pub speed: f64,
    /// Pitch shift multiplier (1.0 = normal).
    pub pitch: f64,
}

impl Voice {
    /// Create a default Kokoro voice configuration.
    pub fn kokoro() -> Self {
        Self {
            backend_type: VoiceBackendType::Kokoro,
            speed: 1.0,
            pitch: 1.0,
        }
    }

    /// Create a default Piper voice configuration.
    pub fn piper() -> Self {
        Self {
            backend_type: VoiceBackendType::Piper,
            speed: 1.0,
            pitch: 1.0,
        }
    }

    /// Create a voice configuration that plays a pre-recorded audio file.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self {
            backend_type: VoiceBackendType::PreRecorded(path.into()),
            speed: 1.0,
            pitch: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kokoro_defaults() {
        let v = Voice::kokoro();
        assert!(matches!(v.backend_type, VoiceBackendType::Kokoro));
        assert!((v.speed - 1.0).abs() < f64::EPSILON);
        assert!((v.pitch - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn piper_defaults() {
        let v = Voice::piper();
        assert!(matches!(v.backend_type, VoiceBackendType::Piper));
        assert!((v.speed - 1.0).abs() < f64::EPSILON);
        assert!((v.pitch - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn file_constructor() {
        let v = Voice::file("/tmp/hello.wav");
        match &v.backend_type {
            VoiceBackendType::PreRecorded(p) => {
                assert_eq!(p, &PathBuf::from("/tmp/hello.wav"));
            }
            other => panic!("expected PreRecorded, got {:?}", other),
        }
    }
}
