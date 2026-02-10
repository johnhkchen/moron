//! Audio utilities: format conversion, normalization, and mixing.

/// Default sample rate for video production (48 kHz, broadcast standard).
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;

/// Errors that can occur when combining audio clips.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    /// Attempted to combine clips with different sample rates.
    #[error("sample rate mismatch: expected {expected}, got {got}")]
    SampleRateMismatch { expected: u32, got: u32 },

    /// Attempted to combine clips with different channel counts.
    #[error("channel count mismatch: expected {expected}, got {got}")]
    ChannelCountMismatch { expected: u16, got: u16 },
}

/// Raw audio clip produced by a TTS backend.
#[derive(Debug, Clone)]
pub struct AudioClip {
    /// Interleaved PCM sample data (f32, normalized to [-1.0, 1.0]).
    pub data: Vec<f32>,
    /// Duration in seconds.
    pub duration: f64,
    /// Sample rate in Hz (e.g. 22050, 44100).
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
}

/// Convert an f32 sample ([-1.0, 1.0]) to a 16-bit signed integer.
///
/// Values outside [-1.0, 1.0] are clamped before scaling.
fn f32_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * 32767.0) as i16
}

impl AudioClip {
    /// Create a silent audio clip of the given duration and sample rate (mono).
    pub fn silence(duration: f64, sample_rate: u32) -> Self {
        let num_samples = (duration * sample_rate as f64) as usize;
        Self {
            data: vec![0.0; num_samples],
            duration,
            sample_rate,
            channels: 1,
        }
    }

    /// Return the duration of this clip in seconds.
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Append another clip's samples to this clip.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::SampleRateMismatch`] if `other` has a different sample rate,
    /// or [`AudioError::ChannelCountMismatch`] if `other` has a different channel count.
    pub fn append(&mut self, other: &AudioClip) -> Result<(), AudioError> {
        if self.sample_rate != other.sample_rate {
            return Err(AudioError::SampleRateMismatch {
                expected: self.sample_rate,
                got: other.sample_rate,
            });
        }
        if self.channels != other.channels {
            return Err(AudioError::ChannelCountMismatch {
                expected: self.channels,
                got: other.channels,
            });
        }
        self.data.extend_from_slice(&other.data);
        self.duration =
            self.data.len() as f64 / (self.sample_rate as f64 * self.channels as f64);
        Ok(())
    }

    /// Concatenate a sequence of clips into a single clip.
    ///
    /// If `clips` is empty, returns a zero-duration clip with the given
    /// `sample_rate` and `channels`.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError`] if any clip has a different `sample_rate` or
    /// `channels` than the specified values.
    pub fn concat(clips: &[AudioClip], sample_rate: u32, channels: u16) -> Result<AudioClip, AudioError> {
        let mut result = AudioClip {
            data: Vec::new(),
            duration: 0.0,
            sample_rate,
            channels,
        };
        for clip in clips {
            result.append(clip)?;
        }
        Ok(result)
    }

    /// Encode this clip as WAV bytes (16-bit signed PCM, RIFF/WAVE container).
    ///
    /// Returns a `Vec<u8>` containing a complete WAV file: 44-byte header
    /// followed by interleaved 16-bit signed integer PCM samples in
    /// little-endian byte order.
    pub fn to_wav_bytes(&self) -> Vec<u8> {
        let bits_per_sample: u16 = 16;
        let bytes_per_sample = bits_per_sample / 8;
        let block_align = self.channels * bytes_per_sample;
        let byte_rate = self.sample_rate * u32::from(block_align);
        let data_size = self.data.len() as u32 * u32::from(bytes_per_sample);
        let chunk_size = 36 + data_size;

        let mut buf = Vec::with_capacity(44 + data_size as usize);

        // RIFF header
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&chunk_size.to_le_bytes());
        buf.extend_from_slice(b"WAVE");

        // fmt sub-chunk
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (PCM)
        buf.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 = PCM)
        buf.extend_from_slice(&self.channels.to_le_bytes());
        buf.extend_from_slice(&self.sample_rate.to_le_bytes());
        buf.extend_from_slice(&byte_rate.to_le_bytes());
        buf.extend_from_slice(&block_align.to_le_bytes());
        buf.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data sub-chunk
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&data_size.to_le_bytes());

        // PCM samples: convert f32 -> i16, write little-endian
        for &sample in &self.data {
            buf.extend_from_slice(&f32_to_i16(sample).to_le_bytes());
        }

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Existing tests -----------------------------------------------------

    #[test]
    fn silence_has_correct_duration() {
        let clip = AudioClip::silence(2.5, 22050);
        assert!((clip.duration() - 2.5).abs() < f64::EPSILON);
        assert_eq!(clip.sample_rate, 22050);
        assert_eq!(clip.channels, 1);
        assert_eq!(clip.data.len(), (2.5 * 22050.0) as usize);
    }

    #[test]
    fn silence_data_is_all_zeros() {
        let clip = AudioClip::silence(1.0, 44100);
        assert!(clip.data.iter().all(|&s| s == 0.0));
    }

    // -- f32_to_i16 tests ---------------------------------------------------

    #[test]
    fn test_f32_to_i16_conversion() {
        assert_eq!(f32_to_i16(0.0), 0);
        assert_eq!(f32_to_i16(1.0), 32767);
        assert_eq!(f32_to_i16(-1.0), -32767);
        // Values beyond range should be clamped
        assert_eq!(f32_to_i16(2.0), 32767);
        assert_eq!(f32_to_i16(-2.0), -32767);
        // Mid-range value
        assert_eq!(f32_to_i16(0.5), 16383);
    }

    // -- append tests -------------------------------------------------------

    #[test]
    fn test_append_same_rate() {
        let mut a = AudioClip::silence(1.0, 48000);
        let b = AudioClip::silence(1.0, 48000);
        assert_eq!(a.data.len(), 48000);

        a.append(&b).unwrap();
        assert_eq!(a.data.len(), 96000);
        assert!((a.duration() - 2.0).abs() < 1e-10);
        assert_eq!(a.sample_rate, 48000);
        assert_eq!(a.channels, 1);
    }

    #[test]
    fn test_append_returns_error_on_sample_rate_mismatch() {
        let mut a = AudioClip::silence(1.0, 48000);
        let b = AudioClip::silence(1.0, 44100);
        let err = a.append(&b).unwrap_err();
        assert!(matches!(err, AudioError::SampleRateMismatch { expected: 48000, got: 44100 }));
    }

    #[test]
    fn test_append_returns_error_on_channel_mismatch() {
        let mut a = AudioClip::silence(1.0, 48000);
        let mut b = AudioClip::silence(1.0, 48000);
        b.channels = 2;
        b.data.extend_from_slice(&vec![0.0; 48000]);
        let err = a.append(&b).unwrap_err();
        assert!(matches!(err, AudioError::ChannelCountMismatch { expected: 1, got: 2 }));
    }

    // -- concat tests -------------------------------------------------------

    #[test]
    fn test_concat_empty() {
        let result = AudioClip::concat(&[], 48000, 1).unwrap();
        assert_eq!(result.data.len(), 0);
        assert!((result.duration()).abs() < f64::EPSILON);
        assert_eq!(result.sample_rate, 48000);
        assert_eq!(result.channels, 1);
    }

    #[test]
    fn test_concat_single() {
        let clip = AudioClip::silence(2.0, 48000);
        let result = AudioClip::concat(&[clip], 48000, 1).unwrap();
        assert_eq!(result.data.len(), 96000);
        assert!((result.duration() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_concat_multiple() {
        let a = AudioClip::silence(1.0, 48000);
        let b = AudioClip::silence(0.5, 48000);
        let c = AudioClip::silence(2.0, 48000);
        let result = AudioClip::concat(&[a, b, c], 48000, 1).unwrap();

        let expected_samples = 48000 + 24000 + 96000;
        assert_eq!(result.data.len(), expected_samples);
        assert!((result.duration() - 3.5).abs() < 1e-10);
    }

    // -- to_wav_bytes tests -------------------------------------------------

    #[test]
    fn test_to_wav_bytes_header() {
        let clip = AudioClip::silence(1.0, 48000); // mono, 48000 samples
        let wav = clip.to_wav_bytes();

        // Total size: 44 header + 48000 samples * 2 bytes each = 96044
        assert_eq!(wav.len(), 44 + 48000 * 2);

        // RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        let chunk_size = u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]);
        assert_eq!(chunk_size, 36 + 48000 * 2);
        assert_eq!(&wav[8..12], b"WAVE");

        // fmt sub-chunk
        assert_eq!(&wav[12..16], b"fmt ");
        let subchunk1_size = u32::from_le_bytes([wav[16], wav[17], wav[18], wav[19]]);
        assert_eq!(subchunk1_size, 16);
        let audio_format = u16::from_le_bytes([wav[20], wav[21]]);
        assert_eq!(audio_format, 1); // PCM
        let num_channels = u16::from_le_bytes([wav[22], wav[23]]);
        assert_eq!(num_channels, 1); // mono
        let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(sample_rate, 48000);
        let byte_rate = u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]);
        assert_eq!(byte_rate, 48000 * 1 * 2); // sample_rate * channels * bytes_per_sample
        let block_align = u16::from_le_bytes([wav[32], wav[33]]);
        assert_eq!(block_align, 2); // channels * bytes_per_sample
        let bits_per_sample = u16::from_le_bytes([wav[34], wav[35]]);
        assert_eq!(bits_per_sample, 16);

        // data sub-chunk
        assert_eq!(&wav[36..40], b"data");
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 48000 * 2);
    }

    #[test]
    fn test_to_wav_bytes_silence_is_zeros() {
        let clip = AudioClip::silence(0.1, 48000); // 4800 samples
        let wav = clip.to_wav_bytes();

        // All PCM data bytes after the 44-byte header should be zero
        let pcm_data = &wav[44..];
        assert!(pcm_data.iter().all(|&b| b == 0));
        assert_eq!(pcm_data.len(), 4800 * 2);
    }

    #[test]
    fn test_to_wav_bytes_with_signal() {
        // Create a clip with known non-zero values
        let mut clip = AudioClip::silence(0.0, 48000);
        clip.data = vec![0.5, -0.5, 1.0, -1.0];
        clip.duration = clip.data.len() as f64 / clip.sample_rate as f64;

        let wav = clip.to_wav_bytes();
        let pcm_data = &wav[44..];

        // 4 samples * 2 bytes = 8 bytes of PCM data
        assert_eq!(pcm_data.len(), 8);

        // Check each sample
        let s0 = i16::from_le_bytes([pcm_data[0], pcm_data[1]]);
        let s1 = i16::from_le_bytes([pcm_data[2], pcm_data[3]]);
        let s2 = i16::from_le_bytes([pcm_data[4], pcm_data[5]]);
        let s3 = i16::from_le_bytes([pcm_data[6], pcm_data[7]]);

        assert_eq!(s0, 16383); // 0.5 * 32767 = 16383.5 -> 16383
        assert_eq!(s1, -16383); // -0.5 * 32767 = -16383.5 -> -16383
        assert_eq!(s2, 32767); // 1.0 * 32767
        assert_eq!(s3, -32767); // -1.0 * 32767
    }

    // -- DEFAULT_SAMPLE_RATE test -------------------------------------------

    #[test]
    fn test_default_sample_rate() {
        assert_eq!(DEFAULT_SAMPLE_RATE, 48000);
    }
}
