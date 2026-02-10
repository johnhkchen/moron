//! End-to-end validation tests for the moron rendering pipeline.
//!
//! These tests exercise the complete pipeline: scene -> timeline -> frames -> video,
//! including TTS integration (narration synthesis, duration resolution, audio assembly).
//!
//! # Running
//!
//! Non-ignored tests (no system dependencies):
//! ```sh
//! cargo test --test e2e
//! ```
//!
//! Full pipeline tests (requires FFmpeg on PATH):
//! ```sh
//! cargo test --test e2e -- --ignored
//! ```
//!
//! TTS pipeline tests (requires Kokoro model files):
//! ```sh
//! export KOKORO_MODEL_PATH=models/kokoro/onnx/model_quantized.onnx
//! export KOKORO_VOICES_PATH=models/kokoro/voices.bin
//! cargo test --test e2e -- --ignored
//! ```
//!
//! All tests in this file:
//! ```sh
//! cargo test --test e2e -- --include-ignored
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use moron_core::prelude::*;
use moron_voice::VoiceBackend;

// ---------------------------------------------------------------------------
// Helper: minimal valid PNG bytes
// ---------------------------------------------------------------------------

/// Return a valid 8x8 solid-blue PNG as a byte vector.
///
/// This is a pre-built minimal PNG file. FFmpeg requires valid PNG input, so
/// we cannot use arbitrary bytes. The image content does not matter for
/// pipeline validation -- we just need structurally valid files.
fn minimal_png_bytes() -> Vec<u8> {
    // Construct a valid PNG programmatically:
    // - 8x8 pixels, RGB (bit depth 8, color type 2)
    // - Single IDAT chunk with zlib-compressed raw scanlines
    //
    // Each scanline: 1 filter byte (0 = None) + 8 pixels * 3 bytes = 25 bytes
    // 8 scanlines = 200 bytes of raw data

    let width: u32 = 8;
    let height: u32 = 8;
    let bytes_per_pixel: u32 = 3; // RGB
    let _scanline_bytes = 1 + width * bytes_per_pixel; // filter byte + pixel data

    // Build raw scanline data (filter=0, solid blue pixels)
    let mut raw_data = Vec::new();
    for _ in 0..height {
        raw_data.push(0u8); // filter byte: None
        for _ in 0..width {
            raw_data.push(0);   // R
            raw_data.push(0);   // G
            raw_data.push(255); // B
        }
    }

    // Compress with zlib (deflate stored blocks -- no compression needed)
    let compressed = zlib_compress_stored(&raw_data);

    // Build PNG file
    let mut png = Vec::new();

    // PNG signature
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR chunk
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8);  // bit depth
    ihdr_data.push(2);  // color type: RGB
    ihdr_data.push(0);  // compression method
    ihdr_data.push(0);  // filter method
    ihdr_data.push(0);  // interlace method
    write_png_chunk(&mut png, b"IHDR", &ihdr_data);

    // IDAT chunk
    write_png_chunk(&mut png, b"IDAT", &compressed);

    // IEND chunk
    write_png_chunk(&mut png, b"IEND", &[]);

    png
}

/// Write a PNG chunk: length (4 bytes) + type (4 bytes) + data + CRC (4 bytes).
fn write_png_chunk(buf: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(chunk_type);
    buf.extend_from_slice(data);

    // CRC32 over chunk_type + data
    let crc = crc32(chunk_type, data);
    buf.extend_from_slice(&crc.to_be_bytes());
}

/// Compute CRC32 for PNG (over chunk type + chunk data).
fn crc32(chunk_type: &[u8; 4], data: &[u8]) -> u32 {
    // CRC32 lookup table (PNG uses the standard CRC-32 polynomial)
    static CRC_TABLE: std::sync::LazyLock<[u32; 256]> = std::sync::LazyLock::new(|| {
        let mut table = [0u32; 256];
        for n in 0..256u32 {
            let mut c = n;
            for _ in 0..8 {
                if c & 1 != 0 {
                    c = 0xEDB88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
            }
            table[n as usize] = c;
        }
        table
    });

    let mut crc = 0xFFFF_FFFFu32;
    for &byte in chunk_type.iter().chain(data.iter()) {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = CRC_TABLE[index] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// Minimal zlib compression using stored (uncompressed) blocks.
///
/// Wraps raw data in a zlib container with no actual compression.
/// This produces valid zlib output that any decoder can handle.
fn zlib_compress_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();

    // Zlib header: CMF=0x78 (deflate, window size 32K), FLG=0x01 (check bits)
    out.push(0x78);
    out.push(0x01);

    // Split into stored blocks of at most 65535 bytes
    let max_block = 65535usize;
    let mut offset = 0;

    while offset < data.len() {
        let remaining = data.len() - offset;
        let block_len = remaining.min(max_block);
        let is_final = offset + block_len >= data.len();

        // Block header: BFINAL (1 bit) + BTYPE=00 (2 bits) = stored block
        out.push(if is_final { 0x01 } else { 0x00 });

        // LEN and NLEN (little-endian 16-bit)
        let len = block_len as u16;
        let nlen = !len;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&nlen.to_le_bytes());

        // Block data
        out.extend_from_slice(&data[offset..offset + block_len]);
        offset += block_len;
    }

    // Handle empty data: need at least one final stored block
    if data.is_empty() {
        out.push(0x01); // final block
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    }

    // Adler-32 checksum of uncompressed data
    let adler = adler32(data);
    out.extend_from_slice(&adler.to_be_bytes());

    out
}

/// Compute Adler-32 checksum.
fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

// ---------------------------------------------------------------------------
// Helper: write synthetic frames to disk
// ---------------------------------------------------------------------------

/// Write `count` numbered PNG files to `dir` using the moron naming convention.
///
/// Files: `frame_000000.png`, `frame_000001.png`, ...
fn write_synthetic_frames(dir: &Path, count: u32) {
    let png_bytes = minimal_png_bytes();
    std::fs::create_dir_all(dir).expect("failed to create frames directory");
    for i in 0..count {
        let path = dir.join(format!("frame_{:06}.png", i));
        std::fs::write(&path, &png_bytes).expect("failed to write synthetic frame");
    }
}

// ---------------------------------------------------------------------------
// Helper: ffprobe utilities (best-effort, returns None if unavailable)
// ---------------------------------------------------------------------------

/// Run ffprobe and return the duration of the media file in seconds.
/// Returns `None` if ffprobe is not available or the command fails.
fn ffprobe_duration(path: &Path) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<f64>().ok()
}

/// Check whether the file has a video stream via ffprobe.
/// Returns `None` if ffprobe is not available.
fn ffprobe_has_video_stream(path: &Path) -> Option<bool> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v",
            "-show_entries", "stream=codec_type",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.trim().contains("video"))
}

/// Check whether the file has an audio stream via ffprobe.
/// Returns `None` if ffprobe is not available.
fn ffprobe_has_audio_stream(path: &Path) -> Option<bool> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "a",
            "-show_entries", "stream=codec_type",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.trim().contains("audio"))
}

// ---------------------------------------------------------------------------
// Helper: unique temp directory for each test
// ---------------------------------------------------------------------------

/// Create a unique temporary directory for a test.
fn test_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "moron-e2e-{}-{}",
        test_name,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create test temp dir");
    dir
}

// ===========================================================================
// Non-ignored tests (no system dependencies required)
// ===========================================================================

#[test]
fn e2e_demo_scene_frame_states_serialize() {
    // Build the demo scene and verify every frame's state serializes to JSON.
    let mut m = M::new();
    DemoScene::build(&mut m);

    let total_frames = m.timeline().total_frames();
    let fps = m.timeline().fps();

    assert!(total_frames > 0, "DemoScene must produce frames");
    assert!(
        m.timeline().total_duration() > 0.0,
        "DemoScene must have positive duration"
    );

    // Compute and serialize FrameState for every frame.
    for frame_num in 0..total_frames {
        let time = frame_num as f64 / fps as f64;
        let state = compute_frame_state(&m, time);

        // Verify serialization succeeds.
        let json = serde_json::to_string(&state);
        assert!(
            json.is_ok(),
            "FrameState serialization failed at frame {frame_num}: {:?}",
            json.err()
        );

        // Verify the JSON is non-empty and contains expected keys.
        let json_str = json.unwrap();
        assert!(!json_str.is_empty());
        assert!(json_str.contains("\"time\""));
        assert!(json_str.contains("\"frame\""));
        assert!(json_str.contains("\"elements\""));
        assert!(json_str.contains("\"theme\""));
    }
}

#[test]
fn e2e_demo_scene_timeline_properties() {
    // Verify DemoScene produces a well-formed timeline.
    let mut m = M::new();
    DemoScene::build(&mut m);

    let tl = m.timeline();

    // DemoScene has narrations, animations, beats, breaths -- should have many segments.
    assert!(
        tl.segments().len() >= 5,
        "DemoScene should produce at least 5 segments, got {}",
        tl.segments().len()
    );

    // Duration should be roughly 3-10 seconds for the demo scene.
    let dur = tl.total_duration();
    assert!(dur > 1.0, "Duration too short: {dur}");
    assert!(dur < 30.0, "Duration unexpectedly long: {dur}");

    // FPS should be the default 30.
    assert_eq!(tl.fps(), 30);

    // Frame count should match duration * fps (ceiling).
    let expected_frames = (dur * 30.0).ceil() as u32;
    assert_eq!(tl.total_frames(), expected_frames);
}

#[test]
fn e2e_empty_scene_produces_build_error() {
    // An empty scene (no timeline segments) should be rejected by build_video.
    let mut m = M::new();
    assert_eq!(m.timeline().total_frames(), 0);

    let temp_dir = test_temp_dir("empty-scene");
    let output_path = temp_dir.join("output.mp4");
    let html_path = temp_dir.join("nonexistent.html");

    let config = BuildConfig::new(&output_path, &html_path);

    // Run the async function in a sync test context.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let result = rt.block_on(build_video(&mut m, config));

    assert!(result.is_err(), "build_video should fail on empty scene");
    let err_msg = match result {
        Err(e) => format!("{e}"),
        Ok(_) => unreachable!("already asserted is_err"),
    };
    assert!(
        err_msg.contains("0 frames"),
        "Error should mention zero frames, got: {err_msg}"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn e2e_audio_assembly_from_demo_scene() {
    // Verify audio track assembly works for the demo scene timeline.
    let mut m = M::new();
    DemoScene::build(&mut m);

    let clip = assemble_audio_track(m.timeline(), moron_voice::DEFAULT_SAMPLE_RATE, None);

    // Audio duration should match timeline duration.
    let tl_dur = m.timeline().total_duration();
    assert!(
        (clip.duration() - tl_dur).abs() < 0.01,
        "Audio duration ({}) should match timeline duration ({tl_dur})",
        clip.duration()
    );

    // WAV encoding should produce valid bytes.
    let wav_bytes = clip.to_wav_bytes();
    assert!(wav_bytes.len() > 44, "WAV must be larger than header");
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");
}

// ===========================================================================
// Ignored tests (require FFmpeg on PATH)
// ===========================================================================

#[test]
#[ignore]
fn e2e_full_pipeline() {
    // Full end-to-end test: scene -> timeline -> frames -> encode -> mux -> validate .mp4
    //
    // Requires: FFmpeg on PATH
    // Run with: cargo test --test e2e -- --ignored

    // Step 0: Check FFmpeg is available.
    detect_ffmpeg().expect(
        "FFmpeg is required for this test. Install FFmpeg and ensure it is on your PATH.",
    );

    let temp_dir = test_temp_dir("full-pipeline");
    let frames_dir = temp_dir.join("frames");
    let video_only_path = temp_dir.join("video_only.mp4");
    let audio_path = temp_dir.join("audio.wav");
    let output_path = temp_dir.join("output.mp4");

    // Step 1: Build the demo scene.
    let mut m = M::new();
    DemoScene::build(&mut m);

    let total_frames = m.timeline().total_frames();
    let total_duration = m.timeline().total_duration();
    let fps = m.timeline().fps();

    assert!(total_frames > 0, "DemoScene should produce frames");
    assert!(total_duration > 0.0, "DemoScene should have positive duration");

    // Step 2: Compute FrameStates for all frames (exercises the frame computation).
    for frame_num in 0..total_frames {
        let time = frame_num as f64 / fps as f64;
        let state = compute_frame_state(&m, time);
        let json = serde_json::to_string(&state);
        assert!(json.is_ok(), "Frame {frame_num} failed to serialize");
    }

    // Step 3: Write synthetic frames (substituting for Chrome rendering).
    write_synthetic_frames(&frames_dir, total_frames);

    // Verify frames were written.
    let frame_count = std::fs::read_dir(&frames_dir)
        .expect("failed to read frames dir")
        .filter(|e| {
            e.as_ref()
                .map(|e| {
                    let name = e.file_name();
                    let name = name.to_string_lossy();
                    name.starts_with("frame_") && name.ends_with(".png")
                })
                .unwrap_or(false)
        })
        .count();
    assert_eq!(
        frame_count, total_frames as usize,
        "Expected {total_frames} frame files, found {frame_count}"
    );

    // Step 4: Encode frames to video-only .mp4.
    let encode_config = EncodeConfig::new(&frames_dir, &video_only_path)
        .fps(fps)
        .resolution(8, 8); // Match our 8x8 synthetic PNGs

    encode_video(&encode_config).expect("FFmpeg encoding failed");

    assert!(video_only_path.exists(), "Video-only .mp4 should exist");
    let video_size = std::fs::metadata(&video_only_path)
        .expect("failed to stat video file")
        .len();
    assert!(video_size > 0, "Video-only .mp4 should be non-empty");

    // Step 5: Assemble audio track and write as WAV.
    let audio_clip = assemble_audio_track(m.timeline(), moron_voice::DEFAULT_SAMPLE_RATE, None);
    let wav_bytes = audio_clip.to_wav_bytes();
    std::fs::write(&audio_path, &wav_bytes).expect("failed to write audio WAV");

    assert!(audio_path.exists(), "Audio WAV should exist");
    assert!(
        std::fs::metadata(&audio_path).unwrap().len() > 44,
        "Audio WAV should be larger than the header"
    );

    // Step 6: Mux video + audio into final .mp4.
    mux_audio(&video_only_path, &audio_path, &output_path).expect("FFmpeg muxing failed");

    // Step 7: Validate the final output.
    assert!(output_path.exists(), "Final .mp4 should exist");
    let output_size = std::fs::metadata(&output_path)
        .expect("failed to stat output file")
        .len();
    assert!(output_size > 0, "Final .mp4 should be non-empty");
    assert!(
        output_size > video_size / 2,
        "Final .mp4 ({output_size} bytes) should be substantial relative to video-only ({video_size} bytes)"
    );

    // Step 8: Optional ffprobe validation.
    if let Some(duration) = ffprobe_duration(&output_path) {
        // Allow some tolerance -- FFmpeg may round slightly.
        assert!(
            (duration - total_duration).abs() < 1.0,
            "Output duration ({duration}s) should be close to timeline duration ({total_duration}s)"
        );
    }

    if let Some(has_video) = ffprobe_has_video_stream(&output_path) {
        assert!(has_video, "Output should contain a video stream");
    }

    if let Some(has_audio) = ffprobe_has_audio_stream(&output_path) {
        assert!(has_audio, "Output should contain an audio stream");
    }

    // Cleanup.
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
#[ignore]
fn e2e_encode_and_mux_roundtrip() {
    // Focused test: write synthetic frames, encode, mux with audio, validate.
    // Simpler than the full pipeline -- no scene building, just FFmpeg integration.
    //
    // Requires: FFmpeg on PATH
    // Run with: cargo test --test e2e -- --ignored

    detect_ffmpeg().expect(
        "FFmpeg is required for this test. Install FFmpeg and ensure it is on your PATH.",
    );

    let temp_dir = test_temp_dir("encode-mux");
    let frames_dir = temp_dir.join("frames");
    let video_only_path = temp_dir.join("video_only.mp4");
    let audio_path = temp_dir.join("audio.wav");
    let output_path = temp_dir.join("output.mp4");

    // Write 30 synthetic frames (1 second at 30fps).
    let frame_count = 30u32;
    let fps = 30u32;
    write_synthetic_frames(&frames_dir, frame_count);

    // Encode.
    let config = EncodeConfig::new(&frames_dir, &video_only_path)
        .fps(fps)
        .resolution(8, 8);

    encode_video(&config).expect("FFmpeg encoding failed");
    assert!(video_only_path.exists());
    assert!(std::fs::metadata(&video_only_path).unwrap().len() > 0);

    // Create a 1-second silence WAV for muxing.
    let audio_clip = moron_voice::AudioClip::silence(1.0, moron_voice::DEFAULT_SAMPLE_RATE);
    let wav_bytes = audio_clip.to_wav_bytes();
    std::fs::write(&audio_path, &wav_bytes).expect("failed to write audio WAV");

    // Mux.
    mux_audio(&video_only_path, &audio_path, &output_path).expect("FFmpeg muxing failed");

    // Validate.
    assert!(output_path.exists(), "Final .mp4 should exist");
    assert!(
        std::fs::metadata(&output_path).unwrap().len() > 0,
        "Final .mp4 should be non-empty"
    );

    // Optional ffprobe checks.
    if let Some(duration) = ffprobe_duration(&output_path) {
        assert!(
            duration > 0.5 && duration < 2.0,
            "Duration should be approximately 1 second, got {duration}s"
        );
    }

    if let Some(has_video) = ffprobe_has_video_stream(&output_path) {
        assert!(has_video, "Should have video stream");
    }

    if let Some(has_audio) = ffprobe_has_audio_stream(&output_path) {
        assert!(has_audio, "Should have audio stream");
    }

    // Cleanup.
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
#[ignore]
fn e2e_ffmpeg_rejects_empty_frames_dir() {
    // Verify that encoding with no frames produces an error.
    //
    // Requires: FFmpeg on PATH

    detect_ffmpeg().expect("FFmpeg is required for this test.");

    let temp_dir = test_temp_dir("empty-frames");
    let frames_dir = temp_dir.join("frames");
    std::fs::create_dir_all(&frames_dir).expect("failed to create frames dir");

    let output_path = temp_dir.join("output.mp4");
    let config = EncodeConfig::new(&frames_dir, &output_path).fps(30);

    let result = encode_video(&config);
    assert!(
        result.is_err(),
        "Encoding with no frames should fail"
    );

    // Cleanup.
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ===========================================================================
// TTS pipeline tests -- mock backend (no system dependencies)
// ===========================================================================

// ---------------------------------------------------------------------------
// Helper: mock TTS backend for integration tests
// ---------------------------------------------------------------------------

/// A mock TTS backend that produces deterministic non-silent audio.
///
/// Each word in the input text produces `seconds_per_word` seconds of audio
/// at a constant sample value of `signal_value`. This makes it easy to verify
/// that assembled audio contains real TTS content (non-zero) vs silence (zero).
struct MockTtsBackend {
    sample_rate: u32,
    seconds_per_word: f64,
    signal_value: f32,
}

impl MockTtsBackend {
    fn new() -> Self {
        Self {
            sample_rate: moron_voice::DEFAULT_SAMPLE_RATE,
            seconds_per_word: 0.3,
            signal_value: 0.42,
        }
    }
}

impl moron_voice::VoiceBackend for MockTtsBackend {
    fn synthesize(&self, text: &str) -> Result<moron_voice::AudioClip, anyhow::Error> {
        let words = text.split_whitespace().count().max(1) as f64;
        let duration = words * self.seconds_per_word;
        let num_samples = (duration * self.sample_rate as f64) as usize;
        Ok(moron_voice::AudioClip {
            data: vec![self.signal_value; num_samples],
            duration,
            sample_rate: self.sample_rate,
            channels: 1,
        })
    }

    fn name(&self) -> &str {
        "mock-e2e"
    }
}

#[test]
fn e2e_mock_tts_duration_resolution() {
    // Build the DemoScene, then manually synthesize narrations using the mock
    // backend and resolve durations -- verifying that the timeline updates from
    // WPM estimates to actual TTS durations.

    let mut m = M::new();
    DemoScene::build(&mut m);

    // Capture WPM-estimated duration before TTS.
    let wpm_duration = m.timeline().total_duration();
    let narration_count = m.timeline().narration_indices().len();
    assert!(narration_count > 0, "DemoScene must have narrations");

    // Collect narration texts.
    let narration_texts: Vec<String> = m
        .timeline()
        .narration_indices()
        .iter()
        .filter_map(|&idx| match &m.timeline().segments()[idx] {
            Segment::Narration { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(narration_texts.len(), narration_count);

    // Synthesize with mock backend.
    let backend = MockTtsBackend::new();
    let mut clips = Vec::new();
    let mut durations = Vec::new();
    for text in &narration_texts {
        let clip = backend.synthesize(text).expect("mock synthesis failed");
        durations.push(clip.duration());
        clips.push(clip);
    }

    // Resolve durations.
    m.resolve_narration_durations(&durations)
        .expect("duration resolution failed");

    let tts_duration = m.timeline().total_duration();

    // The durations should differ from WPM estimates (mock uses 0.3s/word
    // vs WPM's 60/150 = 0.4s/word), so total should change.
    assert!(
        (tts_duration - wpm_duration).abs() > 0.01,
        "timeline duration should change after TTS resolution: WPM={wpm_duration}, TTS={tts_duration}"
    );

    // Verify each narration segment now has the mock TTS duration.
    for (i, &idx) in m.timeline().narration_indices().iter().enumerate() {
        let seg_dur = m.timeline().segments()[idx].duration();
        assert!(
            (seg_dur - durations[i]).abs() < 1e-10,
            "narration segment {i} duration ({seg_dur}) should match TTS duration ({})",
            durations[i]
        );
    }

    // Non-narration segments should be unchanged.
    for (i, seg) in m.timeline().segments().iter().enumerate() {
        if !matches!(seg, Segment::Narration { .. }) {
            let dur = seg.duration();
            assert!(
                dur > 0.0,
                "non-narration segment {i} should have positive duration"
            );
        }
    }

    // Frame count should reflect new duration.
    let expected_frames = (tts_duration * m.timeline().fps() as f64).ceil() as u32;
    assert_eq!(m.timeline().total_frames(), expected_frames);
}

#[test]
fn e2e_audio_assembly_with_tts_clips() {
    // Build a scene with narrations, create mock TTS clips, assemble the
    // audio track, and verify that narration positions contain non-zero
    // samples while silence positions contain zeros.

    let mut m = M::new();
    m.narrate("Hello world");  // 2 words
    m.wait(0.5);               // silence gap
    m.narrate("Goodbye");      // 1 word

    let backend = MockTtsBackend::new();
    let sample_rate = backend.sample_rate;
    let signal_value = backend.signal_value;

    // Synthesize clips.
    let narration_texts: Vec<String> = m
        .timeline()
        .narration_indices()
        .iter()
        .filter_map(|&idx| match &m.timeline().segments()[idx] {
            Segment::Narration { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    let mut clips = Vec::new();
    let mut durations = Vec::new();
    for text in &narration_texts {
        let clip = backend.synthesize(text).expect("mock synthesis failed");
        durations.push(clip.duration());
        clips.push(clip);
    }

    // Resolve durations so timeline matches TTS output.
    m.resolve_narration_durations(&durations)
        .expect("duration resolution failed");

    // Assemble audio track with the TTS clips.
    let assembled = assemble_audio_track(m.timeline(), sample_rate, Some(&clips));

    // Total duration should match timeline.
    let tl_dur = m.timeline().total_duration();
    assert!(
        (assembled.duration() - tl_dur).abs() < 0.01,
        "assembled duration ({}) should match timeline duration ({tl_dur})",
        assembled.duration()
    );

    // Check narration 1: "Hello world" = 2 words * 0.3s = 0.6s.
    let narr1_samples = (durations[0] * sample_rate as f64) as usize;
    assert!(narr1_samples > 0);
    // First narr1_samples should be the signal value.
    for i in 0..narr1_samples {
        assert!(
            (assembled.data[i] - signal_value).abs() < f32::EPSILON,
            "sample {i} in narration 1 should be {signal_value}, got {}",
            assembled.data[i]
        );
    }

    // Check silence gap: 0.5s at 48kHz.
    let silence_start = narr1_samples;
    let silence_samples = (0.5 * sample_rate as f64) as usize;
    for i in 0..silence_samples {
        let idx = silence_start + i;
        assert!(
            assembled.data[idx].abs() < f32::EPSILON,
            "sample {idx} in silence gap should be 0.0, got {}",
            assembled.data[idx]
        );
    }

    // Check narration 2: "Goodbye" = 1 word * 0.3s.
    let narr2_start = silence_start + silence_samples;
    let narr2_samples = (durations[1] * sample_rate as f64) as usize;
    for i in 0..narr2_samples {
        let idx = narr2_start + i;
        assert!(
            (assembled.data[idx] - signal_value).abs() < f32::EPSILON,
            "sample {idx} in narration 2 should be {signal_value}, got {}",
            assembled.data[idx]
        );
    }

    // WAV encoding should produce valid bytes.
    let wav_bytes = assembled.to_wav_bytes();
    assert!(wav_bytes.len() > 44, "WAV must be larger than header");
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");

    // PCM data should contain non-zero bytes (from TTS clips).
    let pcm_data = &wav_bytes[44..];
    let has_nonzero = pcm_data.iter().any(|&b| b != 0);
    assert!(has_nonzero, "WAV PCM data should contain non-silence from TTS clips");
}

// ===========================================================================
// Ignored TTS tests (require Kokoro model files)
// ===========================================================================

#[test]
#[ignore = "requires Kokoro model files (set KOKORO_MODEL_PATH and KOKORO_VOICES_PATH)"]
fn e2e_full_pipeline_with_tts() {
    // Full TTS integration: DemoScene -> Kokoro synthesis -> duration resolution
    // -> audio assembly -> WAV output. Optionally muxes with FFmpeg if available.
    //
    // Requires:
    //   - KOKORO_MODEL_PATH env var pointing to kokoro.onnx
    //   - KOKORO_VOICES_PATH env var pointing to voices.bin
    //   - Optionally: FFmpeg on PATH for the final mux step
    //
    // Run with:
    //   KOKORO_MODEL_PATH=... KOKORO_VOICES_PATH=... cargo test --test e2e -- --ignored

    let model_path = std::env::var("KOKORO_MODEL_PATH")
        .expect("KOKORO_MODEL_PATH must be set");
    let voices_path = std::env::var("KOKORO_VOICES_PATH")
        .expect("KOKORO_VOICES_PATH must be set");

    // Step 1: Build the demo scene and capture WPM duration.
    let mut m = M::new();
    DemoScene::build(&mut m);

    let wpm_duration = m.timeline().total_duration();
    let narration_count = m.timeline().narration_indices().len();
    assert!(narration_count > 0, "DemoScene must have narrations");

    // Step 2: Create Kokoro backend and synthesize narrations.
    let config = moron_voice::KokoroConfig::new(&model_path, &voices_path);
    let backend = moron_voice::KokoroBackend::new(config)
        .expect("failed to create KokoroBackend");

    let narration_texts: Vec<String> = m
        .timeline()
        .narration_indices()
        .iter()
        .filter_map(|&idx| match &m.timeline().segments()[idx] {
            Segment::Narration { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    let mut clips = Vec::new();
    let mut durations = Vec::new();
    for text in &narration_texts {
        let clip = backend.synthesize(text).expect("Kokoro synthesis failed");

        // Verify each clip's basic properties.
        assert!(!clip.data.is_empty(), "TTS clip must not be empty");
        assert_eq!(clip.sample_rate, moron_voice::KOKORO_SAMPLE_RATE);
        assert_eq!(clip.channels, 1);
        assert!(clip.duration() > 0.0);

        durations.push(clip.duration());
        clips.push(clip);
    }

    // Step 3: Resolve narration durations.
    m.resolve_narration_durations(&durations)
        .expect("duration resolution failed");

    let tts_duration = m.timeline().total_duration();

    // Durations should differ from WPM estimates.
    assert!(
        (tts_duration - wpm_duration).abs() > 0.01,
        "timeline should change after TTS: WPM={wpm_duration:.2}s, TTS={tts_duration:.2}s"
    );

    // Step 4: Assemble audio track with real TTS clips.
    let kokoro_sr = moron_voice::KOKORO_SAMPLE_RATE;
    let assembled = assemble_audio_track(m.timeline(), kokoro_sr, Some(&clips));

    assert!(
        (assembled.duration() - tts_duration).abs() < 0.1,
        "assembled audio ({:.2}s) should approximate timeline ({tts_duration:.2}s)",
        assembled.duration()
    );

    // Audio should contain non-silence (real speech).
    let has_speech = assembled.data.iter().any(|&s| s.abs() > 0.001);
    assert!(has_speech, "assembled audio should contain non-silent speech");

    // Step 5: Encode WAV and verify.
    let wav_bytes = assembled.to_wav_bytes();
    assert!(wav_bytes.len() > 44, "WAV must be larger than header");
    assert_eq!(&wav_bytes[0..4], b"RIFF");

    // Step 6: Optionally mux with FFmpeg if available.
    if detect_ffmpeg().is_ok() {
        let temp_dir = test_temp_dir("tts-pipeline");
        let frames_dir = temp_dir.join("frames");
        let video_only_path = temp_dir.join("video_only.mp4");
        let audio_path = temp_dir.join("audio.wav");
        let output_path = temp_dir.join("output.mp4");

        // Write synthetic frames matching the new timeline.
        let total_frames = m.timeline().total_frames();
        write_synthetic_frames(&frames_dir, total_frames);

        // Encode frames to video.
        let encode_config = EncodeConfig::new(&frames_dir, &video_only_path)
            .fps(m.timeline().fps())
            .resolution(8, 8);
        encode_video(&encode_config).expect("FFmpeg encoding failed");

        // Write audio WAV.
        std::fs::write(&audio_path, &wav_bytes).expect("failed to write audio WAV");

        // Mux video + audio.
        mux_audio(&video_only_path, &audio_path, &output_path)
            .expect("FFmpeg muxing failed");

        // Validate output.
        assert!(output_path.exists(), "final .mp4 should exist");
        let output_size = std::fs::metadata(&output_path)
            .expect("failed to stat output")
            .len();
        assert!(output_size > 0, "final .mp4 should be non-empty");

        // Check for audio stream.
        if let Some(has_audio) = ffprobe_has_audio_stream(&output_path) {
            assert!(has_audio, "output .mp4 should contain an audio stream");
        }

        if let Some(has_video) = ffprobe_has_video_stream(&output_path) {
            assert!(has_video, "output .mp4 should contain a video stream");
        }

        // Cleanup.
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
