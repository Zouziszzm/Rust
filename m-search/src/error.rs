use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("invalid filename glob pattern: {0}")]
    InvalidGlob(#[from] globset::Error),

    #[error("invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),

    #[error("path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("walk error: {0}")]
    Walk(#[from] ignore::Error),
}
