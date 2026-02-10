//! moron-voice: TTS abstraction with pluggable backends.
//!
//! Supports Kokoro (primary), Piper (fallback), API providers, and pre-recorded audio.

pub mod backend;
pub mod kokoro;
pub mod piper;
pub mod alignment;
pub mod audio;
