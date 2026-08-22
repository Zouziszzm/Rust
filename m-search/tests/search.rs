use std::fs;
use std::path::PathBuf;

use m_search::{search, SearchConfig};

fn make_config(
    pattern: &str,
    roots: Vec<PathBuf>,
    name_only: bool,
    content_only: bool,
) -> SearchConfig {
    SearchConfig::from_pattern(pattern, roots, name_only, content_only, false, false, Some(2), None)
        .expect("valid config")
}

#[test]
fn finds_files_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    fs::write(root.join("alpha.rs"), "fn alpha() {}").unwrap();
    fs::write(root.join("beta.txt"), "text").unwrap();

    let config = make_config("*.rs", vec![root.clone()], true, false);
    let results = search(&config).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, root.join("alpha.rs"));
    assert!(results[0].filename_match);
}

#[test]
fn finds_content_matches() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    fs::write(root.join("main.rs"), "fn main() {}\nlet x = 1;\n").unwrap();

    let config = make_config("fn main", vec![root.clone()], false, true);
    let results = search(&config).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content_matches.len(), 1);
    assert_eq!(results[0].content_matches[0].line_number, 1);
}

#[test]
fn searches_both_name_and_content_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    fs::write(root.join("todo.txt"), "nothing here").unwrap();
    fs::write(root.join("notes.md"), "TODO: finish this").unwrap();

    let config = make_config("TODO", vec![root.clone()], false, false);
    let results = search(&config).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, root.join("notes.md"));
}
