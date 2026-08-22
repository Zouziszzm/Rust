use std::path::Path;
use std::sync::Arc;

use crossbeam_channel::{bounded, Sender};
use ignore::WalkBuilder;
use ignore::WalkState;

use crate::config::{ContentPattern, SearchConfig};
use crate::error::SearchError;
use crate::matcher::{content, filename};
use crate::SearchResult;

pub fn search(config: &SearchConfig) -> Result<Vec<SearchResult>, SearchError> {
    for root in &config.roots {
        if !root.exists() {
            return Err(SearchError::PathNotFound(root.clone()));
        }
    }

    let content_regex = config.content.as_ref().map(|pattern| match pattern {
        ContentPattern::Regex(regex) => Arc::new(regex.clone()),
    });

    let (tx, rx) = bounded::<SearchResult>(256);

    for root in &config.roots {
        walk_root(root, config, content_regex.clone(), tx.clone())?;
    }

    drop(tx);

    let mut results: Vec<SearchResult> = rx.iter().collect();
    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

fn walk_root(
    root: &Path,
    config: &SearchConfig,
    content_regex: Option<Arc<regex::Regex>>,
    tx: Sender<SearchResult>,
) -> Result<(), SearchError> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!config.hidden)
        .follow_links(false)
        .threads(config.threads.unwrap_or_else(num_cpus));

    if let Some(depth) = config.max_depth {
        builder.max_depth(Some(depth));
    }

    let filename_pattern = config.filename.clone();
    let has_filename = filename_pattern.is_some();
    let has_content = content_regex.is_some();

    builder.build_parallel().run(|| {
        let tx = tx.clone();
        let filename_pattern = filename_pattern.clone();
        let content_regex = content_regex.clone();

        Box::new(move |entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                return WalkState::Continue;
            }

            let path = entry.path();
            let filename_match = filename_pattern
                .as_ref()
                .map(|pattern| filename::matches(path, pattern))
                .unwrap_or(false);

            let content_matches = content_regex
                .as_ref()
                .and_then(|regex| content::search_file(path, regex))
                .unwrap_or_default();

            let content_match = !content_matches.is_empty();

            let should_emit = match (has_filename, has_content) {
                (true, true) => filename_match || content_match,
                (true, false) => filename_match,
                (false, true) => content_match,
                (false, false) => false,
            };

            if should_emit {
                let _ = tx.send(SearchResult {
                    path: path.to_path_buf(),
                    filename_match,
                    content_matches,
                });
            }

            WalkState::Continue
        })
    });

    Ok(())
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
