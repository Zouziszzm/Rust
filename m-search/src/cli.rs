use std::path::PathBuf;

use clap::Parser;

use crate::SearchConfig;

#[derive(Debug, Parser)]
#[command(
    name = "m-search",
    about = "Multi-threaded file name and content search"
)]
pub struct Args {
    pub pattern: String,

    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    #[arg(short = 'n', long = "name-only", conflicts_with = "content_only")]
    pub name_only: bool,

    #[arg(short = 'c', long = "content-only", conflicts_with = "name_only")]
    pub content_only: bool,

    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    #[arg(short = 'j', long = "threads")]
    pub threads: Option<usize>,

    #[arg(long = "hidden")]
    pub hidden: bool,

    #[arg(long = "max-depth")]
    pub max_depth: Option<usize>,

    #[arg(long = "json")]
    pub json: bool,
}

impl Args {
    pub fn into_config(self) -> Result<SearchConfig, crate::SearchError> {
        SearchConfig::from_pattern(
            &self.pattern,
            self.paths,
            self.name_only,
            self.content_only,
            self.ignore_case,
            self.hidden,
            self.threads,
            self.max_depth,
        )
    }
}
