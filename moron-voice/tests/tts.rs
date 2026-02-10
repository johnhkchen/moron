//! TTS backend validation tests for moron-voice.
//!
//! These tests exercise the Kokoro TTS backend through the public
//! `VoiceBackend` trait. They require the Kokoro ONNX model files to be
//! present on disk.
//!
//! # Setup
//!
//! Download the model files from HuggingFace:
//!
//! ```sh
//! # Clone the ONNX model repo (or download individual files):
//! git lfs install
//! git clone https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX models/kokoro
//!
//! # Set environment variables:
//! export KOKORO_MODEL_PATH=models/kokoro/onnx/model_quantized.onnx
//! export KOKORO_VOICES_PATH=models/kokoro/voices.bin
//! ```
//!
//! # Running
//!
//! ```sh
//! # Skip these tests (default -- no model files):
//! cargo test -p moron-voice --test tts
//!
//! # Run with model files:
//! cargo test -p moron-voice --test tts -- --ignored
//! ```

#[cfg(feature = "kokoro")]
mod kokoro_tests {
    use moron_voice::{KokoroBackend, KokoroConfig, KOKORO_SAMPLE_RATE, VoiceBackend};

    /// Read Kokoro model paths from environment variables.
    /// Returns `None` if either variable is unset.
    fn kokoro_config() -> Option<KokoroConfig> {
        let model = std::env::var("KOKORO_MODEL_PATH").ok()?;
        let voices = std::env::var("KOKORO_VOICES_PATH").ok()?;
        Some(KokoroConfig::new(model, voices))
    }

    #[test]
    #[ignore = "requires Kokoro model files (set KOKORO_MODEL_PATH and KOKORO_VOICES_PATH)"]
    fn kokoro_synthesis_produces_valid_audio() {
        // Synthesize a short phrase and verify AudioClip properties.
        let config = kokoro_config().expect(
            "KOKORO_MODEL_PATH and KOKORO_VOICES_PATH must be set to run this test",
        );
        let backend = KokoroBackend::new(config).expect("failed to create KokoroBackend");

        let clip = backend
            .synthesize("Hello, this is a test of the Kokoro speech synthesis engine.")
            .expect("synthesis failed");

        // Basic properties.
        assert!(!clip.data.is_empty(), "audio data must not be empty");
        assert_eq!(clip.sample_rate, KOKORO_SAMPLE_RATE, "sample rate must be 24000 Hz");
        assert_eq!(clip.channels, 1, "output must be mono");
        assert!(clip.duration() > 0.0, "duration must be positive");

        // Duration should be reasonable for a ~12-word sentence (roughly 1-10 seconds).
        assert!(
            clip.duration() > 0.5,
            "duration too short: {:.2}s",
            clip.duration()
        );
        assert!(
            clip.duration() < 30.0,
            "duration unexpectedly long: {:.2}s",
            clip.duration()
        );

        // Sample count should match duration * sample_rate (within 1 sample).
        let expected_samples = (clip.duration() * clip.sample_rate as f64).round() as usize;
        let sample_diff = (clip.data.len() as i64 - expected_samples as i64).unsigned_abs() as usize;
        assert!(
            sample_diff <= 1,
            "sample count ({}) should match duration * sample_rate ({})",
            clip.data.len(),
            expected_samples
        );

        // Samples should be in valid range [-1.0, 1.0] (or at least close).
        let max_abs = clip
            .data
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);
        assert!(
            max_abs <= 1.5,
            "samples should be approximately in [-1.0, 1.0], max absolute value: {max_abs}"
        );

        // Audio should not be all zeros (should contain actual speech).
        let has_nonzero = clip.data.iter().any(|&s| s.abs() > 0.001);
        assert!(has_nonzero, "audio data should contain non-silence");
    }

    #[test]
    #[ignore = "requires Kokoro model files (set KOKORO_MODEL_PATH and KOKORO_VOICES_PATH)"]
    fn kokoro_wav_encoding_roundtrip() {
        // Synthesize text, encode to WAV, verify the WAV is well-formed.
        let config = kokoro_config().expect(
            "KOKORO_MODEL_PATH and KOKORO_VOICES_PATH must be set to run this test",
        );
        let backend = KokoroBackend::new(config).expect("failed to create KokoroBackend");

        let clip = backend
            .synthesize("Testing WAV encoding.")
            .expect("synthesis failed");

        let wav_bytes = clip.to_wav_bytes();

        // WAV must have at minimum a 44-byte header.
        assert!(
            wav_bytes.len() > 44,
            "WAV must be larger than the 44-byte header, got {} bytes",
            wav_bytes.len()
        );

        // RIFF header.
        assert_eq!(&wav_bytes[0..4], b"RIFF", "must start with RIFF");
        assert_eq!(&wav_bytes[8..12], b"WAVE", "must contain WAVE marker");

        // fmt sub-chunk.
        assert_eq!(&wav_bytes[12..16], b"fmt ", "must contain fmt chunk");
        let audio_format = u16::from_le_bytes([wav_bytes[20], wav_bytes[21]]);
        assert_eq!(audio_format, 1, "audio format must be PCM (1)");
        let num_channels = u16::from_le_bytes([wav_bytes[22], wav_bytes[23]]);
        assert_eq!(num_channels, 1, "must be mono");
        let sample_rate = u32::from_le_bytes([wav_bytes[24], wav_bytes[25], wav_bytes[26], wav_bytes[27]]);
        assert_eq!(sample_rate, KOKORO_SAMPLE_RATE, "sample rate must match Kokoro output");

        // data sub-chunk.
        assert_eq!(&wav_bytes[36..40], b"data", "must contain data chunk");
        let data_size = u32::from_le_bytes([wav_bytes[40], wav_bytes[41], wav_bytes[42], wav_bytes[43]]);
        assert!(data_size > 0, "data chunk must be non-empty");

        // PCM data should contain non-zero bytes (actual speech, not silence).
        let pcm_data = &wav_bytes[44..];
        let has_nonzero = pcm_data.iter().any(|&b| b != 0);
        assert!(has_nonzero, "WAV PCM data should contain non-silence");
    }
}
