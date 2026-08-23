//! Terminal styling — consistent per-member colors and message types.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use owo_colors::OwoColorize;

/// Pick a stable palette index for a nickname (same name → same color every session).
pub fn member_palette_index(name: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    name.to_lowercase().hash(&mut hasher);
    (hasher.finish() as usize) % 10
}

pub fn paint_member(name: &str) -> String {
    match member_palette_index(name) {
        0 => name.red().bold().to_string(),
        1 => name.green().bold().to_string(),
        2 => name.yellow().bold().to_string(),
        3 => name.blue().bold().to_string(),
        4 => name.magenta().bold().to_string(),
        5 => name.cyan().bold().to_string(),
        6 => name.bright_red().bold().to_string(),
        7 => name.bright_green().bold().to_string(),
        8 => name.bright_blue().bold().to_string(),
        _ => name.bright_magenta().bold().to_string(),
    }
}

pub fn paint_channel_header(code: &str, member_count: usize) -> String {
    format!(
        "{} {} {} {}",
        "===".cyan().bold(),
        format!("Channel {}", code.yellow().bold()),
        "|".cyan().bold(),
        format!("{member_count} member(s)").bright_cyan(),
    )
}

pub fn paint_channel_code(code: &str) -> String {
    code.yellow().bold().to_string()
}

pub fn paint_channel_message(from: &str, text: &str, my_name: &str) -> String {
    let name = paint_member(from);
    let body = if from == my_name {
        text.dimmed().to_string()
    } else {
        text.to_string()
    };
    format!("[{name}] {body}")
}

pub fn paint_dm_received(from: &str, text: &str) -> String {
    format!(
        "{} {} {}",
        "DM".magenta().bold(),
        format!("from {}", paint_member(from)).magenta(),
        text.bright_white()
    )
}

pub fn paint_dm_sent(to: &str, text: &str) -> String {
    format!(
        "{} {} {}",
        "DM".magenta().bold(),
        format!("to {}", paint_member(to)).bright_magenta(),
        text.dimmed()
    )
}

pub fn paint_system(text: &str) -> String {
    text.bright_black().italic().to_string()
}

pub fn paint_success(text: &str) -> String {
    text.green().bold().to_string()
}

pub fn paint_error(text: &str) -> String {
    text.red().bold().to_string()
}

pub fn paint_hint(text: &str) -> String {
    text.bright_black().to_string()
}

pub fn paint_prompt() -> String {
    "> ".bright_cyan().bold().to_string()
}

pub fn paint_banner_title() -> String {
    "s-c-s".cyan().bold().to_string()
}

pub fn paint_member_line(name: &str, is_you: bool) -> String {
    let painted = paint_member(name);
    if is_you {
        format!("  - {painted} {}", "(you)".bright_black())
    } else {
        format!("  - {painted}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_color_is_stable() {
        assert_eq!(
            member_palette_index("alice"),
            member_palette_index("alice")
        );
        assert_eq!(
            member_palette_index("Alice"),
            member_palette_index("alice")
        );
    }

    #[test]
    fn different_names_can_differ() {
        // Not guaranteed, but likely for these two
        let a = member_palette_index("alice");
        let b = member_palette_index("zzzzzz");
        // Just ensure they're in range
        assert!(a < 10 && b < 10);
    }
}
