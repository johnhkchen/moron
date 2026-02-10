//! Piper backend: fallback local TTS engine integration.

use crate::audio::AudioClip;
use crate::backend::VoiceBackend;

/// Piper TTS backend — fallback local engine.
pub struct PiperBackend;

impl VoiceBackend for PiperBackend {
    fn synthesize(&self, _text: &str) -> Result<AudioClip, anyhow::Error> {
        todo!("PiperBackend::synthesize not yet implemented")
    }

    fn name(&self) -> &str {
        "piper"
    }
}
