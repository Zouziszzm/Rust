use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub date: Option<String>,
}

pub struct ParsedMarkdown<'a> {
    pub front_matter: FrontMatter,
    pub body: &'a str,
}

pub fn parse_front_matter(raw: &str) -> Result<ParsedMarkdown<'_>> {
    let trimmed = raw.trim_start();

    if !trimmed.starts_with("---") {
        return Ok(ParsedMarkdown {
            front_matter: FrontMatter::default(),
            body: raw,
        });
    }

    let after_open = trimmed[3..].trim_start_matches(['\r', '\n']);
    let end = after_open
        .find("\n---")
        .context("front-matter opening --- found but no closing ---")?;

    let yaml = after_open[..end].trim();
    let body_start = end + 4;
    let body = after_open[body_start..].trim_start_matches(['\r', '\n']);

    let front_matter: FrontMatter = if yaml.is_empty() {
        FrontMatter::default()
    } else {
        serde_yaml::from_str(yaml).with_context(|| "failed to parse YAML front-matter")?
    };

    Ok(ParsedMarkdown { front_matter, body })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_front_matter() {
        let raw = "---\ntitle: Hello\ndate: 2026-08-24\n---\n# Body\n";
        let parsed = parse_front_matter(raw).unwrap();

        assert_eq!(parsed.front_matter.title.as_deref(), Some("Hello"));
        assert_eq!(parsed.front_matter.date.as_deref(), Some("2026-08-24"));
        assert_eq!(parsed.body.trim(), "# Body");
    }

    #[test]
    fn no_front_matter_returns_full_body() {
        let raw = "# Just markdown\n";
        let parsed = parse_front_matter(raw).unwrap();

        assert!(parsed.front_matter.title.is_none());
        assert_eq!(parsed.body, raw);
    }

    #[test]
    fn empty_body_after_front_matter() {
        let raw = "---\ntitle: Empty\n---\n";
        let parsed = parse_front_matter(raw).unwrap();

        assert_eq!(parsed.front_matter.title.as_deref(), Some("Empty"));
        assert_eq!(parsed.body, "");
    }

    #[test]
    fn malformed_yaml_errors() {
        let raw = "---\ntitle: [unclosed\n---\nbody\n";
        assert!(parse_front_matter(raw).is_err());
    }
}
