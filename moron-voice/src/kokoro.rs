//! Kokoro backend: primary local TTS engine integration.

use crate::audio::AudioClip;
use crate::backend::VoiceBackend;

/// Kokoro TTS backend — primary local engine.
pub struct KokoroBackend;

impl VoiceBackend for KokoroBackend {
    fn synthesize(&self, _text: &str) -> Result<AudioClip, anyhow::Error> {
        todo!("KokoroBackend::synthesize not yet implemented")
    }

    fn name(&self) -> &str {
        "kokoro"
    }
}
