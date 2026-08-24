use std::path::Path;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::frontmatter::parse_front_matter;
use crate::markdown::render_markdown;
use crate::template::apply_template;
use crate::walker::{find_markdown_files, read_file};

pub fn build_page(md_path: &Path, template_path: &Path, out_path: &Path) -> Result<()> {
    let raw = read_file(md_path)?;
    let template = read_file(template_path)?;
    let parsed = parse_front_matter(&raw)?;
    let html_body = render_markdown(parsed.body);
    let page = apply_template(&template, &parsed.front_matter, &html_body);

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    std::fs::write(out_path, page).with_context(|| format!("failed to write {}", out_path.display()))?;
    Ok(())
}

pub fn build_site(
    input_dir: &Path,
    output_dir: &Path,
    template_path: &Path,
    assets_dir: &Path,
) -> Result<usize> {
    let markdown_files = find_markdown_files(input_dir)?;
    let count = markdown_files.len();

    for md_path in &markdown_files {
        let relative = md_path
            .strip_prefix(input_dir)
            .with_context(|| format!("{} is not under {}", md_path.display(), input_dir.display()))?;

        let html_relative = relative.with_extension("html");
        let out_path = output_dir.join(html_relative);

        build_page(md_path, template_path, &out_path)?;
    }

    copy_static_assets(assets_dir, output_dir, input_dir)?;
    Ok(count)
}

fn copy_static_assets(assets_dir: &Path, output_dir: &Path, input_dir: &Path) -> Result<()> {
    if !assets_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(assets_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_hidden(e.path()))
    {
        let entry = entry.with_context(|| format!("failed to walk {}", assets_dir.display()))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "md") {
            continue;
        }

        let relative = if path.starts_with(input_dir) {
            path.strip_prefix(input_dir)
                .with_context(|| format!("{} is not under {}", path.display(), input_dir.display()))?
        } else {
            path.strip_prefix(assets_dir)
                .with_context(|| format!("{} is not under {}", path.display(), assets_dir.display()))?
        };

        let dest = output_dir.join(relative);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        std::fs::copy(path, &dest)
            .with_context(|| format!("failed to copy {} to {}", path.display(), dest.display()))?;
    }

    Ok(())
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}
