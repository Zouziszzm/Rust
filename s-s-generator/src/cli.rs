use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ssg", about = "A small static site generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Build HTML from markdown content
    Build {
        /// Input content directory
        #[arg(long, default_value = "content")]
        input: PathBuf,

        /// Output directory for generated HTML
        #[arg(long, default_value = "dist")]
        output: PathBuf,

        /// HTML layout template with {{placeholders}}
        #[arg(long, default_value = "templates/default.html")]
        template: PathBuf,

        /// Optional: read a single file instead of walking the input directory
        #[arg(long)]
        file: Option<PathBuf>,

        /// Static assets directory to copy into output (defaults to input)
        #[arg(long)]
        assets: Option<PathBuf>,

        /// Remove output directory before building
        #[arg(long)]
        clean: bool,
    },
}
