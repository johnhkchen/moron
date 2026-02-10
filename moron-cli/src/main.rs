//! moron-cli: Binary wrapper for `moron build`, `moron preview`, and related commands.

use clap::{Parser, Subcommand};

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
    },
    /// Launch a live-preview window with hot reload
    Preview {
        /// Path to the project directory
        #[arg(default_value = ".")]
        path: String,
    },
    /// Scaffold a new moron project
    Init {
        /// Name of the new project
        name: Option<String>,
    },
    /// Browse the built-in technique gallery
    Gallery,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { path } => {
            println!("moron build: not yet implemented (path: {path})");
        }
        Commands::Preview { path } => {
            println!("moron preview: not yet implemented (path: {path})");
        }
        Commands::Init { name } => {
            let name = name.as_deref().unwrap_or("my-moron-project");
            println!("moron init: not yet implemented (name: {name})");
        }
        Commands::Gallery => {
            println!("moron gallery: not yet implemented");
        }
    }

    Ok(())
}
