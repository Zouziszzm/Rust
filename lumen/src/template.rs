use crate::html::{extract_media, strip_html};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct RenderedCard {
    pub front: String,
    pub back: String,
    pub front_html: String,
    pub back_html: String,
    pub images: Vec<String>,
    pub audio: Vec<String>,
    pub video: Vec<String>,
}

enum ClozeSide {
    Front,
    Back,
    Both,
}

fn mustache_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{\{([^}]+)\}\}").unwrap())
}

fn cloze_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)\{\{c(\d+)::(.*?)(?:::(.*?))?\}\}").unwrap())
}

pub fn looks_like_cloze(fields: &HashMap<String, String>) -> bool {
    fields.values().any(|v| cloze_re().is_match(v))
}

pub fn render_card(
    front_template: &str,
    back_template: &str,
    fields: &HashMap<String, String>,
    card_ord: i64,
    is_cloze: bool,
) -> RenderedCard {
    let processed: HashMap<String, String> = fields
        .iter()
        .map(|(k, v)| {
            let value = if is_cloze {
                apply_cloze(v, card_ord + 1, ClozeSide::Both)
            } else {
                v.clone()
            };
            (k.clone(), value)
        })
        .collect();

    let mut front = fill(front_template, &processed, None);
    let mut back = fill(back_template, &processed, Some(&front));

    if front.trim().is_empty() && !fields.is_empty() {
        let values: Vec<&String> = fields.values().collect();
        front = if is_cloze {
            apply_cloze(values[0], card_ord + 1, ClozeSide::Front)
        } else {
            values[0].clone()
        };
        back = if values.len() > 1 {
            values[1].clone()
        } else {
            values[0].clone()
        };
        if is_cloze {
            back = apply_cloze(values[0], card_ord + 1, ClozeSide::Back);
        }
    }

    let media = extract_media(&format!("{front}\n{back}"));
    RenderedCard {
        front: strip_html(&front),
        back: strip_html(&back),
        front_html: front,
        back_html: back,
        images: media.images,
        audio: media.audio,
        video: media.video,
    }
}

fn apply_cloze(text: &str, cloze_n: i64, side: ClozeSide) -> String {
    cloze_re()
        .replace_all(text, |caps: &regex::Captures| {
            let n: i64 = caps[1].parse().unwrap_or(0);
            let answer = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let hint = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            if n == cloze_n {
                match side {
                    ClozeSide::Back => answer.to_string(),
                    ClozeSide::Front if !hint.is_empty() => format!("[{hint}]"),
                    _ => "[…]".to_string(),
                }
            } else {
                answer.to_string()
            }
        })
        .into_owned()
}

fn fill(template: &str, fields: &HashMap<String, String>, front_side: Option<&str>) -> String {
    let mut out = template.to_string();
    if let Some(front) = front_side {
        out = out.replace("{{FrontSide}}", front);
    }
    out = mustache_re()
        .replace_all(&out, |caps: &regex::Captures| {
            let raw = caps[1].trim();
            if raw == "FrontSide" {
                return front_side.unwrap_or("").to_string();
            }
            if let Some(name) = raw.strip_prefix("cloze:") {
                return fields.get(name).cloned().unwrap_or_default();
            }
            if let Some(name) = raw.strip_prefix("text:") {
                return strip_html(fields.get(name).map(String::as_str).unwrap_or(""));
            }
            if let Some(name) = raw.strip_prefix("type:") {
                return fields.get(name).cloned().unwrap_or_default();
            }
            if raw.starts_with('#') || raw.starts_with('/') || raw.starts_with('^') {
                return String::new();
            }
            fields.get(raw).cloned().unwrap_or_default()
        })
        .into_owned();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic() {
        let fields = HashMap::from([
            ("Front".into(), "Capital of France?".into()),
            ("Back".into(), "Paris".into()),
        ]);
        let card = render_card(
            "{{Front}}",
            "{{FrontSide}}<hr id=answer>{{Back}}",
            &fields,
            0,
            false,
        );
        assert_eq!(card.front, "Capital of France?");
        assert!(card.back.contains("Paris"));
    }

    #[test]
    fn renders_cloze() {
        let fields = HashMap::from([(
            "Text".into(),
            "The {{c1::Seine}} runs through Paris".into(),
        )]);
        let card = render_card("{{cloze:Text}}", "{{cloze:Text}}", &fields, 0, true);
        assert!(!card.front.contains("Seine"));
        assert!(card.front.contains('…') || card.front.contains("["));
    }
}
