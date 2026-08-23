use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::session::ChatLine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum StoredLine {
    Channel { from: String, text: String },
    DmReceived { from: String, text: String },
    DmSent { to: String, text: String },
    System { text: String },
}

pub fn data_root() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("SCS_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").context("HOME not set — cannot store chat history")?;
    Ok(PathBuf::from(home).join(".local/share/s-c-s"))
}

fn channel_dir(code: &str) -> Result<PathBuf> {
    let dir = data_root()?.join("channels").join(code);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

fn history_path(code: &str) -> Result<PathBuf> {
    Ok(channel_dir(code)?.join("messages.jsonl"))
}

pub fn load_channel_history(code: &str) -> Result<Vec<ChatLine>> {
    let path = history_path(code)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let stored: StoredLine = serde_json::from_str(&line)
            .with_context(|| format!("bad history line in {}", path.display()))?;
        lines.push(stored.into());
    }

    Ok(lines)
}

pub fn append_line(code: &str, line: &ChatLine) -> Result<()> {
    let path = history_path(code)?;
    let stored = StoredLine::from(line);
    let json = serde_json::to_string(&stored)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("append {}", path.display()))?;
    writeln!(file, "{json}")?;
    Ok(())
}

pub fn history_dir_display(code: &str) -> Result<String> {
    Ok(channel_dir(code)?.display().to_string())
}

impl From<&ChatLine> for StoredLine {
    fn from(line: &ChatLine) -> Self {
        match line {
            ChatLine::Channel { from, text } => StoredLine::Channel {
                from: from.clone(),
                text: text.clone(),
            },
            ChatLine::DmReceived { from, text } => StoredLine::DmReceived {
                from: from.clone(),
                text: text.clone(),
            },
            ChatLine::DmSent { to, text } => StoredLine::DmSent {
                to: to.clone(),
                text: text.clone(),
            },
            ChatLine::System(text) => StoredLine::System { text: text.clone() },
            ChatLine::Error(_) => StoredLine::System {
                text: "(error)".into(),
            },
        }
    }
}

impl From<StoredLine> for ChatLine {
    fn from(stored: StoredLine) -> Self {
        match stored {
            StoredLine::Channel { from, text } => ChatLine::Channel { from, text },
            StoredLine::DmReceived { from, text } => ChatLine::DmReceived { from, text },
            StoredLine::DmSent { to, text } => ChatLine::DmSent { to, text },
            StoredLine::System { text } => ChatLine::System(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_history_file() -> Result<()> {
        let dir = tempdir()?;
        std::env::set_var("SCS_DATA_DIR", dir.path());

        let line = ChatLine::Channel {
            from: "alice".into(),
            text: "hello".into(),
        };
        append_line("ABC123", &line)?;
        let loaded = load_channel_history("ABC123")?;
        assert_eq!(loaded.len(), 1);
        assert!(matches!(loaded[0], ChatLine::Channel { .. }));

        std::env::remove_var("SCS_DATA_DIR");
        Ok(())
    }
}
