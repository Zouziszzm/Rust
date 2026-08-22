use std::path::PathBuf;

use serde::Serialize;

pub mod cli;
pub mod config;
pub mod engine;
pub mod error;
pub mod matcher;
pub mod output;

pub use config::SearchConfig;
pub use engine::search;
pub use error::SearchError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchResult {
    pub path: PathBuf,
    pub filename_match: bool,
    pub content_matches: Vec<LineMatch>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LineMatch {
    pub line_number: usize,
    pub line: String,
}
