//! moron-voice: TTS abstraction with pluggable backends.
//!
//! Supports Kokoro (primary), Piper (fallback), API providers, and pre-recorded audio.

pub mod alignment;
pub mod audio;
pub mod backend;
pub mod kokoro;
pub mod piper;

pub use audio::{AudioClip, DEFAULT_SAMPLE_RATE};
pub use backend::{Voice, VoiceBackend, VoiceBackendType};
pub use kokoro::KokoroBackend;
#[cfg(feature = "kokoro")]
pub use kokoro::{KokoroConfig, KokoroError, KokoroVoice, KOKORO_SAMPLE_RATE};
pub use piper::PiperBackend;
