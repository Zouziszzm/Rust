use std::path::PathBuf;

use regex::Regex;

use crate::error::SearchError;

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub roots: Vec<PathBuf>,
    pub filename: Option<FilenamePattern>,
    pub content: Option<ContentPattern>,
    pub case_insensitive: bool,
    pub hidden: bool,
    pub threads: Option<usize>,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum FilenamePattern {
    Glob(globset::GlobMatcher),
    Regex(Regex),
}

#[derive(Debug, Clone)]
pub enum ContentPattern {
    Regex(Regex),
}

impl SearchConfig {
    pub fn from_pattern(
        pattern: &str,
        roots: Vec<PathBuf>,
        name_only: bool,
        content_only: bool,
        case_insensitive: bool,
        hidden: bool,
        threads: Option<usize>,
        max_depth: Option<usize>,
    ) -> Result<Self, SearchError> {
        let filename = if content_only {
            None
        } else {
            Some(FilenamePattern::Glob(build_glob(pattern, case_insensitive)?))
        };

        let content = if name_only {
            None
        } else {
            Some(ContentPattern::Regex(build_content_regex(pattern, case_insensitive)?))
        };

        Ok(Self {
            roots,
            filename,
            content,
            case_insensitive,
            hidden,
            threads,
            max_depth,
        })
    }
}

fn build_regex(pattern: &str, case_insensitive: bool) -> Result<Regex, SearchError> {
    let mut builder = regex::RegexBuilder::new(pattern);
    builder.case_insensitive(case_insensitive);
    Ok(builder.build()?)
}

fn build_content_regex(pattern: &str, case_insensitive: bool) -> Result<Regex, SearchError> {
    match build_regex(pattern, case_insensitive) {
        Ok(regex) => Ok(regex),
        Err(_) => build_regex(&regex::escape(pattern), case_insensitive),
    }
}

fn build_glob(pattern: &str, case_insensitive: bool) -> Result<globset::GlobMatcher, SearchError> {
    let mut builder = globset::GlobBuilder::new(pattern);
    builder.case_insensitive(case_insensitive);
    Ok(builder.build()?.compile_matcher())
}
