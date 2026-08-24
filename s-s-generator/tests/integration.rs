use std::fs;

use ssg::{build_site, find_markdown_files, parse_front_matter, read_file, render_markdown};
use tempfile::tempdir;

#[test]
fn integration_build_site() {
    let input = tempdir().unwrap();
    let output = tempdir().unwrap();

    let content_dir = input.path().join("content");
    let posts_dir = content_dir.join("posts");
    fs::create_dir_all(&posts_dir).unwrap();

    fs::write(
        content_dir.join("index.md"),
        "---\ntitle: Index\n---\n# Index page\n",
    )
    .unwrap();
    fs::write(
        posts_dir.join("hello.md"),
        "---\ntitle: Hello\n---\n# Hello post\n",
    )
    .unwrap();

    let template = input.path().join("template.html");
    fs::write(
        &template,
        "<html><title>{{title}}</title><body>{{content}}</body></html>",
    )
    .unwrap();

    let count = build_site(&content_dir, output.path(), &template, &content_dir).unwrap();
    assert_eq!(count, 2);

    let index_html = fs::read_to_string(output.path().join("index.html")).unwrap();
    assert!(index_html.contains("<title>Index</title>"));
    assert!(index_html.contains("<h1>Index page</h1>"));

    let post_html = fs::read_to_string(output.path().join("posts/hello.html")).unwrap();
    assert!(post_html.contains("<title>Hello</title>"));
    assert!(post_html.contains("<h1>Hello post</h1>"));
}

#[test]
fn integration_front_matter_and_markdown() {
    let raw = "---\ntitle: Test\n---\n# Heading\n";
    let parsed = parse_front_matter(raw).unwrap();
    let html = render_markdown(parsed.body);

    assert_eq!(parsed.front_matter.title.as_deref(), Some("Test"));
    assert!(html.contains("<h1>Heading</h1>"));
}

#[test]
fn integration_find_markdown_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "a").unwrap();
    fs::create_dir(dir.path().join("posts")).unwrap();
    fs::write(dir.path().join("posts/b.md"), "b").unwrap();
    fs::write(dir.path().join("notes.txt"), "skip").unwrap();

    let files = find_markdown_files(dir.path()).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn integration_read_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.md");
    fs::write(&path, "hello").unwrap();

    let content = read_file(&path).unwrap();
    assert_eq!(content, "hello");
}
