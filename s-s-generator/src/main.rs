use anyhow::Result;
use clap::Parser;

mod cli;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build {
            input,
            output,
            template,
            file,
            assets,
            clean,
        } => {
            if clean && output.exists() {
                std::fs::remove_dir_all(&output)?;
            }

            let assets_dir = assets.unwrap_or_else(|| input.clone());

            if let Some(path) = file {
                let raw = ssg::read_file(&path)?;
                println!("read {} bytes from {}", raw.len(), path.display());
            } else {
                let count = ssg::build_site(&input, &output, &template, &assets_dir)?;
                println!("built {count} page(s) into {}", output.display());
            }
        }
    }

    Ok(())
}
