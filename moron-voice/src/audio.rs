//! Audio utilities: format conversion, normalization, and mixing.

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
