use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

fn img_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?i)<img[^>]+src=["']([^"']+)["']"#).unwrap())
}

fn video_src_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?i)<video[^>]+src=["']([^"']+)["']"#).unwrap())
}

fn sound_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[sound:([^\]]+)\]").unwrap())
}

fn br_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)<br\s*/?>").unwrap())
}

fn hr_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)<hr[^>]*>").unwrap())
}

fn divp_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)</(p|div)>").unwrap())
}

fn tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap())
}

fn entity_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"&(#\d+|#x[0-9a-fA-F]+|\w+);").unwrap())
}

fn dangerous_block_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?is)<(script|style|iframe|object|embed|form)[\s\S]*?</(?:script|style|iframe|object|embed|form)>").unwrap()
    })
}

fn dangerous_void_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)</?(script|style|iframe|object|embed|link|meta|form)[^>]*>").unwrap())
}

fn event_attr_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?i)\s+on[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)"#).unwrap())
}

fn js_url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?i)(?:href|src)\s*=\s*['"]\s*javascript:[^'"]*['"]"#).unwrap())
}

pub struct CardMedia {
    pub images: Vec<String>,
    pub audio: Vec<String>,
    pub video: Vec<String>,
}

pub fn is_video_file(name: &str) -> bool {
    matches!(
        Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "mp4" | "webm" | "mov" | "mkv" | "m4v" | "avi" | "ogv"
    )
}

pub fn extract_media(html: &str) -> CardMedia {
    let mut audio = Vec::new();
    let mut video = Vec::new();
    for cap in sound_re().captures_iter(html) {
        let name = cap[1].to_string();
        if is_video_file(&name) {
            video.push(name);
        } else {
            audio.push(name);
        }
    }
    for cap in video_src_re().captures_iter(html) {
        let name = cap[1].to_string();
        if !video.iter().any(|v| v == &name) {
            video.push(name);
        }
    }
    CardMedia {
        images: img_re()
            .captures_iter(html)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect(),
        audio,
        video,
    }
}

pub fn strip_html(html: &str) -> String {
    let mut text = br_re().replace_all(html, "\n").into_owned();
    text = hr_re().replace_all(&text, "\n").into_owned();
    text = divp_re().replace_all(&text, "\n").into_owned();
    text = sound_re().replace_all(&text, "").into_owned();
    text = tag_re().replace_all(&text, "").into_owned();
    text = entity_re()
        .replace_all(&text, |caps: &regex::Captures| match &caps[1] {
            "nbsp" => " ".to_string(),
            "amp" => "&".to_string(),
            "lt" => "<".to_string(),
            "gt" => ">".to_string(),
            "quot" => "\"".to_string(),
            "apos" => "'".to_string(),
            other => format!("&{other};"),
        })
        .into_owned();

    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn rewrite_media_src(html: &str, filename_to_path: &[(String, String)]) -> String {
    let mut out = html.to_string();
    for (name, path) in filename_to_path {
        out = out.replace(&format!("src=\"{name}\""), &format!("src=\"{path}\""));
        out = out.replace(&format!("src='{name}'"), &format!("src='{path}'"));
    }
    out
}

pub fn media_url(filename: &str) -> String {
    let encoded: String = filename
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    format!("lumenmedia://localhost/{encoded}")
}

pub fn prepare_card_html(html: &str, media: &[(String, String)]) -> String {
    let mut out = sound_re()
        .replace_all(html, |caps: &regex::Captures| {
            let raw = caps[1].trim();
            let name = Path::new(raw)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(raw);
            player_tag(name)
        })
        .into_owned();

    for (name, path) in media {
        let url = media_url(name);
        for original in [name.as_str(), path.as_str()] {
            out = out.replace(&format!("src=\"{original}\""), &format!("src=\"{url}\""));
            out = out.replace(&format!("src='{original}'"), &format!("src='{url}'"));
        }
    }

    sanitize_html(&out)
}

fn player_tag(filename: &str) -> String {
    let src = media_url(filename);
    let label = html_escape(filename);
    if is_video_file(filename) {
        format!(
            r#"<div class="lumen-media lumen-media-video"><video class="lumen-video" controls playsinline preload="metadata" src="{src}" data-filename="{label}"></video></div>"#
        )
    } else {
        format!(
            r#"<div class="lumen-media lumen-media-audio"><audio class="lumen-audio" controls preload="auto" src="{src}" data-filename="{label}"></audio></div>"#
        )
    }
}

pub fn sanitize_html(html: &str) -> String {
    let mut out = dangerous_block_re().replace_all(html, "").into_owned();
    out = dangerous_void_re().replace_all(&out, "").into_owned();
    out = event_attr_re().replace_all(&out, "").into_owned();
    out = js_url_re().replace_all(&out, "").into_owned();
    out
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_sound_to_audio() {
        let html = prepare_card_html("Listen [sound:clip.mp3]", &[]);
        assert!(html.contains("<audio"));
        assert!(html.contains("lumen-media-audio"));
        assert!(html.contains("lumenmedia://localhost/clip.mp3"));
        assert!(!html.contains("[sound:"));
    }

    #[test]
    fn converts_video_sound_tag() {
        let html = prepare_card_html("[sound:clip.mp4]", &[]);
        assert!(html.contains("<video"));
        assert!(html.contains("clip.mp4"));
    }

    #[test]
    fn strips_scripts() {
        let html = prepare_card_html("<script>alert(1)</script>Safe", &[]);
        assert!(!html.to_lowercase().contains("script"));
        assert!(html.contains("Safe"));
    }
}
