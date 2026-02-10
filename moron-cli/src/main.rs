//! moron-cli: Binary wrapper for the `moron build` command.

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use moron_core::{
    build_video, BuildConfig, BuildError, BuildProgress, DemoScene, M, Scene,
    WhatIsMoronScene,
};

#[derive(Parser)]
#[command(name = "moron", version, about = "Motion graphics renderer — offline-first, code-driven")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a project to video
    Build {
        /// Path to the project directory
        #[arg(default_value = ".")]
        path: String,

        /// Output file path
        #[arg(short, long, default_value = "output.mp4")]
        output: String,

        /// Path to the built React app's index.html
        #[arg(long)]
        html_path: Option<String>,

        /// Output video width in pixels
        #[arg(long, default_value = "1920")]
        width: u32,

        /// Output video height in pixels
        #[arg(long, default_value = "1080")]
        height: u32,

        /// Keep intermediate frame PNGs (do not clean up temp directory)
        #[arg(long)]
        keep_frames: bool,

        /// Scene to render: "what-is-moron" (default) or "demo"
        #[arg(long, default_value = "what-is-moron")]
        scene: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            path,
            output,
            html_path,
            width,
            height,
            keep_frames,
            scene,
        } => {
            run_build(path, output, html_path, width, height, keep_frames, scene).await?;
        }
    }

    Ok(())
}

/// Run the full build pipeline.
async fn run_build(
    path: String,
    output: String,
    html_path: Option<String>,
    width: u32,
    height: u32,
    keep_frames: bool,
    scene: String,
) -> anyhow::Result<()> {
    // Resolve the HTML path: CLI flag, or convention-based fallback.
    let resolved_html_path = resolve_html_path(&path, html_path.as_deref())?;

    // Build the selected scene.
    let mut m = M::new();
    match scene.as_str() {
        "demo" => DemoScene::build(&mut m),
        "what-is-moron" => WhatIsMoronScene::build(&mut m),
        other => anyhow::bail!("Unknown scene: {other}. Available: what-is-moron, demo"),
    };

    // Create progress callback.
    let progress: Arc<dyn Fn(BuildProgress) + Send + Sync> = Arc::new(|event| {
        match event {
            BuildProgress::SynthesizingTts { current, total } => {
                eprintln!(
                    "[0/4] Synthesizing TTS {}/{} ({:.0}%)",
                    current + 1,
                    total,
                    (current + 1) as f64 / total as f64 * 100.0,
                );
            }
            BuildProgress::SceneBuilt { total_duration, total_frames } => {
                eprintln!(
                    "[1/4] Scene built: {total_frames} frames, {total_duration:.1}s"
                );
            }
            BuildProgress::RenderingFrame { current, total } => {
                eprintln!(
                    "[2/4] Rendering frame {}/{} ({:.0}%)",
                    current + 1,
                    total,
                    (current + 1) as f64 / total as f64 * 100.0,
                );
            }
            BuildProgress::Encoding => {
                eprintln!("[3/4] Encoding video...");
            }
            BuildProgress::MuxingAudio => {
                eprintln!("[3/4] Muxing audio...");
            }
            BuildProgress::Complete { ref output_path, total_frames, duration } => {
                eprintln!(
                    "[4/4] Done: {} ({} frames, {:.1}s)",
                    output_path.display(),
                    total_frames,
                    duration,
                );
            }
        }
    });

    let config = BuildConfig {
        output_path: PathBuf::from(&output),
        html_path: resolved_html_path,
        width,
        height,
        keep_frames,
        progress: Some(progress),
        voice_backend: None,
    };

    match build_video(&mut m, config).await {
        Ok(result) => {
            println!(
                "Build complete: {} ({} frames, {:.1}s)",
                result.output_path.display(),
                result.total_frames,
                result.duration,
            );
            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!("{}", format_build_error(&e)))
        }
    }
}

/// Resolve the path to the React app's index.html.
///
/// Priority:
/// 1. Explicit `--html-path` CLI flag
/// 2. Convention: `{project_path}/packages/ui/dist/index.html`
///
/// Returns an error if neither exists.
fn resolve_html_path(project_path: &str, explicit: Option<&str>) -> anyhow::Result<PathBuf> {
    if let Some(explicit_path) = explicit {
        let p = PathBuf::from(explicit_path);
        if !p.exists() {
            anyhow::bail!(
                "HTML path does not exist: {}\n\
                 Provide a valid path to the built React app's index.html.",
                p.display()
            );
        }
        return Ok(p);
    }

    // Convention-based fallback.
    let convention = PathBuf::from(project_path)
        .join("packages/ui/dist/index.html");
    if convention.exists() {
        return Ok(convention);
    }

    anyhow::bail!(
        "Could not locate React app index.html.\n\
         Tried: {}\n\
         Use --html-path to specify the location explicitly.",
        convention.display()
    )
}

/// Format a BuildError into a user-friendly message.
fn format_build_error(err: &BuildError) -> String {
    match err {
        BuildError::Render(render_err) => {
            let msg = format!("{render_err}");
            if msg.contains("Chrome") || msg.contains("chrome") {
                format!(
                    "Error: Chrome/Chromium not found or failed to launch.\n\
                     Install Chrome or set CHROME_PATH environment variable.\n\
                     Details: {msg}"
                )
            } else {
                format!("Error: Rendering failed.\nDetails: {msg}")
            }
        }
        BuildError::Ffmpeg(ffmpeg_err) => {
            let msg = format!("{ffmpeg_err}");
            if msg.contains("not found") || msg.contains("NotFound") {
                "Error: FFmpeg not found.\n\
                 Install FFmpeg and ensure it is on your PATH.\n\
                 Download: https://ffmpeg.org/download.html"
                    .to_string()
            } else {
                format!("Error: Video encoding failed.\nDetails: {msg}")
            }
        }
        BuildError::Io(io_err) => {
            format!("Error: I/O failure.\nDetails: {io_err}")
        }
        BuildError::Config(msg) => {
            format!("Error: {msg}")
        }
        BuildError::Tts { segment, source } => {
            format!(
                "Error: TTS synthesis failed for narration segment {segment}.\n\
                 Details: {source}"
            )
        }
        BuildError::Audio(audio_err) => {
            format!("Error: Audio assembly failed.\nDetails: {audio_err}")
        }
    }
}
