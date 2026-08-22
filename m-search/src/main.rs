use anyhow::Result;
use clap::Parser;

use m_search::cli::Args;
use m_search::output;
use m_search::search;

fn main() -> Result<()> {
    let args = Args::parse();
    let json = args.json;
    let config = args.into_config()?;
    let results = search(&config)?;
    output::print_results(&results, json)?;
    Ok(())
}
