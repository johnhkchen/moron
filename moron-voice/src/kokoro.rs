//! Kokoro backend: primary local TTS engine integration.
//!
//! Uses the `kokoro-tts` crate for offline speech synthesis powered by the
//! Kokoro-82M ONNX model. The model files (~87 MB) must be downloaded
//! separately from HuggingFace:
//!
//! - Model: <https://huggingface.co/hexgrad/Kokoro-82M>
//! - ONNX: <https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX>
//!
//! # Example
//!
//! ```no_run
//! use moron_voice::{KokoroBackend, KokoroConfig, KokoroVoice, VoiceBackend};
//!
//! let config = KokoroConfig::new("models/kokoro.onnx", "models/voices.bin")
//!     .with_voice(KokoroVoice::AfHeart)
//!     .with_speed(1.0);
//! let backend = KokoroBackend::new(config).unwrap();
//! let clip = backend.synthesize("Hello, world!").unwrap();
//! assert_eq!(clip.sample_rate, 24000);
//! ```

use std::path::PathBuf;

use crate::audio::AudioClip;
use crate::backend::VoiceBackend;

/// Sample rate produced by the Kokoro model (24 kHz).
pub const KOKORO_SAMPLE_RATE: u32 = 24000;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors specific to the Kokoro TTS backend.
#[derive(Debug, thiserror::Error)]
pub enum KokoroError {
    /// The ONNX model file was not found at the configured path.
    #[error("kokoro model not found: {0}")]
    ModelNotFound(PathBuf),

    /// The voices data file was not found at the configured path.
    #[error("kokoro voices file not found: {0}")]
    VoicesNotFound(PathBuf),

    /// The ONNX model failed to load (runtime error).
    #[error("kokoro model load failed: {0}")]
    ModelLoadFailed(String),

    /// Speech synthesis failed at runtime.
    #[error("kokoro synthesis failed: {0}")]
    SynthesisFailed(String),

    /// The input text was empty.
    #[error("cannot synthesize empty text")]
    EmptyText,

    /// Failed to create the async runtime for the sync bridge.
    #[error("failed to create tokio runtime: {0}")]
    RuntimeCreationFailed(String),
}

// ---------------------------------------------------------------------------
// Voice selection
// ---------------------------------------------------------------------------

/// Curated set of Kokoro voice styles.
///
/// These map to the voice variants available in the Kokoro-82M model.
/// Not all 157 upstream voices are exposed — this is a practical subset
/// covering the most useful English voices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KokoroVoice {
    /// American female — warm, friendly (default).
    #[default]
    AfHeart,
    /// American female — bright, clear.
    AfSky,
    /// American female — calm, professional.
    AfBella,
    /// American female — smooth narrator.
    AfNova,
    /// American female — natural conversational.
    AfSarah,
    /// American male — neutral, clear.
    AmAdam,
    /// American male — energetic.
    AmPuck,
    /// American male — deep, authoritative.
    AmEric,
    /// American male — warm narrator.
    AmMichael,
    /// British female — elegant.
    BfEmma,
    /// British male — clear narrator.
    BmGeorge,
    /// British male — warm storyteller.
    BmLewis,
}

#[cfg(feature = "kokoro")]
impl KokoroVoice {
    /// Convert to the upstream `kokoro_tts::Voice` enum variant.
    fn to_kokoro_tts_voice(self) -> kokoro_tts::Voice {
        match self {
            Self::AfHeart => kokoro_tts::Voice::AfHeart(1.0),
            Self::AfSky => kokoro_tts::Voice::AfSky(1.0),
            Self::AfBella => kokoro_tts::Voice::AfBella(1.0),
            Self::AfNova => kokoro_tts::Voice::AfNova(1.0),
            Self::AfSarah => kokoro_tts::Voice::AfSarah(1.0),
            Self::AmAdam => kokoro_tts::Voice::AmAdam(1.0),
            Self::AmPuck => kokoro_tts::Voice::AmPuck(1.0),
            Self::AmEric => kokoro_tts::Voice::AmEric(1.0),
            Self::AmMichael => kokoro_tts::Voice::AmMichael(1.0),
            Self::BfEmma => kokoro_tts::Voice::BfEmma(1.0),
            Self::BmGeorge => kokoro_tts::Voice::BmGeorge(1.0),
            Self::BmLewis => kokoro_tts::Voice::BmLewis(1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for the Kokoro TTS backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KokoroConfig {
    /// Path to the Kokoro ONNX model file.
    pub model_path: PathBuf,
    /// Path to the voices data file (voices.bin).
    pub voices_path: PathBuf,
    /// Which voice style to use.
    pub voice: KokoroVoice,
    /// Speech speed multiplier (1.0 = normal).
    pub speed: f32,
}

impl KokoroConfig {
    /// Create a new config with the given model and voices paths.
    ///
    /// Uses `KokoroVoice::AfHeart` and speed 1.0 by default.
    pub fn new(model_path: impl Into<PathBuf>, voices_path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: model_path.into(),
            voices_path: voices_path.into(),
            voice: KokoroVoice::default(),
            speed: 1.0,
        }
    }

    /// Set the voice style.
    pub fn with_voice(mut self, voice: KokoroVoice) -> Self {
        self.voice = voice;
        self
    }

    /// Set the speech speed multiplier.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Validate that the configured paths exist on disk.
    pub fn validate(&self) -> Result<(), KokoroError> {
        if !self.model_path.exists() {
            return Err(KokoroError::ModelNotFound(self.model_path.clone()));
        }
        if !self.voices_path.exists() {
            return Err(KokoroError::VoicesNotFound(self.voices_path.clone()));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Backend (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "kokoro")]
mod inner {
    use std::sync::Mutex;

    use super::*;

    /// Kokoro TTS backend — primary local engine.
    ///
    /// Wraps the `kokoro-tts` crate with lazy model loading. The ONNX model
    /// is loaded on the first call to [`synthesize()`] (or fails fast if the
    /// model files are missing).
    pub struct KokoroBackend {
        config: KokoroConfig,
        engine: Mutex<Option<kokoro_tts::KokoroTts>>,
        runtime: tokio::runtime::Runtime,
    }

    impl KokoroBackend {
        /// Create a new Kokoro backend with the given configuration.
        ///
        /// This does **not** load the model. The model is loaded lazily on the
        /// first call to [`synthesize()`]. Call [`KokoroConfig::validate()`]
        /// to check paths eagerly.
        pub fn new(config: KokoroConfig) -> Result<Self, KokoroError> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| KokoroError::RuntimeCreationFailed(e.to_string()))?;

            Ok(Self {
                config,
                engine: Mutex::new(None),
                runtime,
            })
        }

        /// Ensure the model is loaded, initializing it if needed.
        fn ensure_loaded(&self) -> Result<(), KokoroError> {
            let mut guard = self.engine.lock().map_err(|e| {
                KokoroError::ModelLoadFailed(format!("engine lock poisoned: {}", e))
            })?;

            if guard.is_some() {
                return Ok(());
            }

            // Validate paths before attempting load.
            self.config.validate()?;

            let engine = self.runtime.block_on(async {
                kokoro_tts::KokoroTts::new(
                    &self.config.model_path,
                    &self.config.voices_path,
                )
                .await
                .map_err(|e| KokoroError::ModelLoadFailed(e.to_string()))
            })?;

            *guard = Some(engine);
            Ok(())
        }
    }

    impl VoiceBackend for KokoroBackend {
        fn synthesize(&self, text: &str) -> Result<AudioClip, anyhow::Error> {
            if text.trim().is_empty() {
                return Err(KokoroError::EmptyText.into());
            }

            self.ensure_loaded()?;

            let guard = self.engine.lock().map_err(|e| {
                KokoroError::SynthesisFailed(format!("engine lock poisoned: {}", e))
            })?;
            let engine = guard.as_ref().expect("engine must be loaded after ensure_loaded");
            let voice = self.config.voice.to_kokoro_tts_voice();

            let (samples, duration) = self.runtime.block_on(async {
                engine.synth(text, voice).await
            })
            .map_err(|e| KokoroError::SynthesisFailed(e.to_string()))?;

            Ok(AudioClip {
                data: samples,
                duration: duration.as_secs_f64(),
                sample_rate: KOKORO_SAMPLE_RATE,
                channels: 1,
            })
        }

        fn name(&self) -> &str {
            "kokoro"
        }
    }
}

#[cfg(feature = "kokoro")]
pub use inner::KokoroBackend;

// ---------------------------------------------------------------------------
// Stub when feature is disabled
// ---------------------------------------------------------------------------

#[cfg(not(feature = "kokoro"))]
mod inner {
    use super::*;

    /// Stub Kokoro backend when the `kokoro` feature is disabled.
    pub struct KokoroBackend;

    impl VoiceBackend for KokoroBackend {
        fn synthesize(&self, _text: &str) -> Result<AudioClip, anyhow::Error> {
            anyhow::bail!(
                "kokoro TTS is not available: compile with the `kokoro` feature enabled"
            )
        }

        fn name(&self) -> &str {
            "kokoro (disabled)"
        }
    }
}

#[cfg(not(feature = "kokoro"))]
pub use inner::KokoroBackend;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Config tests -------------------------------------------------------

    #[test]
    fn config_new_sets_defaults() {
        let config = KokoroConfig::new("/tmp/model.onnx", "/tmp/voices.bin");
        assert_eq!(config.model_path, PathBuf::from("/tmp/model.onnx"));
        assert_eq!(config.voices_path, PathBuf::from("/tmp/voices.bin"));
        assert_eq!(config.voice, KokoroVoice::AfHeart);
        assert!((config.speed - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn config_builder_methods() {
        let config = KokoroConfig::new("/tmp/model.onnx", "/tmp/voices.bin")
            .with_voice(KokoroVoice::AmAdam)
            .with_speed(1.5);
        assert_eq!(config.voice, KokoroVoice::AmAdam);
        assert!((config.speed - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn config_validate_missing_model() {
        let config = KokoroConfig::new(
            "/nonexistent/path/model.onnx",
            "/nonexistent/path/voices.bin",
        );
        let err = config.validate().unwrap_err();
        assert!(matches!(err, KokoroError::ModelNotFound(_)));
        assert!(err.to_string().contains("/nonexistent/path/model.onnx"));
    }

    #[test]
    fn config_validate_missing_voices() {
        // Create a temp file to act as model so we get past the first check.
        let dir = std::env::temp_dir().join("moron_test_kokoro_validate");
        let _ = std::fs::create_dir_all(&dir);
        let model_path = dir.join("model.onnx");
        std::fs::write(&model_path, b"fake").unwrap();

        let config = KokoroConfig::new(&model_path, "/nonexistent/voices.bin");
        let err = config.validate().unwrap_err();
        assert!(matches!(err, KokoroError::VoicesNotFound(_)));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- Voice tests --------------------------------------------------------

    #[test]
    fn voice_default_is_af_heart() {
        assert_eq!(KokoroVoice::default(), KokoroVoice::AfHeart);
    }

    #[test]
    fn voice_enum_has_expected_variants() {
        // Ensure all curated voices exist and are distinct.
        let voices = [
            KokoroVoice::AfHeart,
            KokoroVoice::AfSky,
            KokoroVoice::AfBella,
            KokoroVoice::AfNova,
            KokoroVoice::AfSarah,
            KokoroVoice::AmAdam,
            KokoroVoice::AmPuck,
            KokoroVoice::AmEric,
            KokoroVoice::AmMichael,
            KokoroVoice::BfEmma,
            KokoroVoice::BmGeorge,
            KokoroVoice::BmLewis,
        ];
        // All should be distinct
        for (i, a) in voices.iter().enumerate() {
            for (j, b) in voices.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "voices at index {} and {} should differ", i, j);
                }
            }
        }
    }

    #[test]
    fn voice_clone_and_copy() {
        let v = KokoroVoice::AfSky;
        let v2 = v; // Copy
        let v3 = v.clone(); // Clone
        assert_eq!(v, v2);
        assert_eq!(v, v3);
    }

    // -- Backend tests (no model required) ----------------------------------

    #[cfg(feature = "kokoro")]
    mod backend_tests {
        use super::*;

        #[test]
        fn backend_name() {
            let config = KokoroConfig::new("/tmp/model.onnx", "/tmp/voices.bin");
            let backend = KokoroBackend::new(config).unwrap();
            assert_eq!(backend.name(), "kokoro");
        }

        #[test]
        fn synthesize_missing_model_returns_error() {
            let config = KokoroConfig::new(
                "/nonexistent/kokoro.onnx",
                "/nonexistent/voices.bin",
            );
            let backend = KokoroBackend::new(config).unwrap();
            let result = backend.synthesize("hello");
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(
                err_msg.contains("not found"),
                "error should mention 'not found', got: {}",
                err_msg
            );
        }

        #[test]
        fn synthesize_empty_text_returns_error() {
            let config = KokoroConfig::new("/tmp/model.onnx", "/tmp/voices.bin");
            let backend = KokoroBackend::new(config).unwrap();
            let result = backend.synthesize("");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("empty text"));
        }

        #[test]
        fn synthesize_whitespace_only_returns_error() {
            let config = KokoroConfig::new("/tmp/model.onnx", "/tmp/voices.bin");
            let backend = KokoroBackend::new(config).unwrap();
            let result = backend.synthesize("   \n\t  ");
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("empty text"));
        }
    }

    // -- Integration tests (require model files) ----------------------------

    #[cfg(feature = "kokoro")]
    mod integration_tests {
        use super::*;

        /// Set `KOKORO_MODEL_PATH` and `KOKORO_VOICES_PATH` env vars to run.
        fn model_config() -> Option<KokoroConfig> {
            let model = std::env::var("KOKORO_MODEL_PATH").ok()?;
            let voices = std::env::var("KOKORO_VOICES_PATH").ok()?;
            Some(KokoroConfig::new(model, voices))
        }

        #[test]
        #[ignore = "requires Kokoro model files (set KOKORO_MODEL_PATH and KOKORO_VOICES_PATH)"]
        fn synthesize_produces_audio() {
            let config = model_config().expect("model env vars not set");
            let backend = KokoroBackend::new(config).unwrap();
            let clip = backend.synthesize("Hello, world!").unwrap();

            assert!(!clip.data.is_empty(), "audio data should not be empty");
            assert_eq!(clip.sample_rate, KOKORO_SAMPLE_RATE);
            assert_eq!(clip.channels, 1);
            assert!(clip.duration > 0.0, "duration should be positive");
        }

        #[test]
        #[ignore = "requires Kokoro model files (set KOKORO_MODEL_PATH and KOKORO_VOICES_PATH)"]
        fn synthesize_duration_scales_with_text() {
            let config = model_config().expect("model env vars not set");
            let backend = KokoroBackend::new(config).unwrap();

            let short = backend.synthesize("Hi").unwrap();
            let long = backend
                .synthesize("This is a much longer sentence with many more words to speak.")
                .unwrap();

            assert!(
                long.duration > short.duration,
                "longer text should produce longer audio: short={:.2}s, long={:.2}s",
                short.duration,
                long.duration
            );
        }

        #[test]
        #[ignore = "requires Kokoro model files (set KOKORO_MODEL_PATH and KOKORO_VOICES_PATH)"]
        fn synthesize_different_voices() {
            let config = KokoroConfig::new(
                std::env::var("KOKORO_MODEL_PATH").unwrap(),
                std::env::var("KOKORO_VOICES_PATH").unwrap(),
            )
            .with_voice(KokoroVoice::AmAdam);

            let backend = KokoroBackend::new(config).unwrap();
            let clip = backend.synthesize("Testing voice selection.").unwrap();

            assert!(!clip.data.is_empty());
            assert_eq!(clip.sample_rate, KOKORO_SAMPLE_RATE);
        }
    }
}
