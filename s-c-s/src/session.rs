use ratatui::style::{Color, Modifier, Style};

use crate::protocol::{ChannelInfo, Member};

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub code: String,
    pub members: Vec<Member>,
    pub my_name: String,
    pub save_chat: bool,
    pub creator: String,
}

impl ChannelState {
    pub fn from_info(info: ChannelInfo, my_name: &str) -> Self {
        Self {
            code: info.code,
            members: info.members,
            my_name: my_name.to_string(),
            save_chat: info.save_chat,
            creator: info.creator,
        }
    }

    pub fn is_creator(&self) -> bool {
        self.my_name.eq_ignore_ascii_case(&self.creator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEnd {
    Switch,
    Quit,
}

#[derive(Debug)]
pub enum InputAction {
    ChannelMessage(String),
    OpenDmPicker,
    OpenSaveChatPicker,
    ShowMembers,
    Help,
    Switch,
    Quit,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum ChatLine {
    Channel { from: String, text: String },
    DmReceived { from: String, text: String },
    DmSent { to: String, text: String },
    System(String),
    Error(String),
}

/// Nicknames: no spaces. Letters, numbers, `_` and `-` only.
pub fn validate_nickname(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("nickname cannot be empty".into());
    }
    if name.chars().any(char::is_whitespace) {
        return Err("nickname cannot contain spaces — use _ instead (e.g. farhaan_2)".into());
    }
    if name.len() > 32 {
        return Err("nickname too long (max 32 characters)".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("use only letters, numbers, _ or -".into());
    }
    Ok(name.to_string())
}

pub fn dm_candidates(state: &ChannelState) -> Vec<String> {
    state
        .members
        .iter()
        .map(|m| m.name.clone())
        .filter(|n| !n.eq_ignore_ascii_case(&state.my_name))
        .collect()
}

pub fn parse_input(line: &str, _state: &ChannelState) -> Option<InputAction> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    match line {
        "/help" => return Some(InputAction::Help),
        "/members" => return Some(InputAction::ShowMembers),
        "/switch" => return Some(InputAction::Switch),
        "/quit" => return Some(InputAction::Quit),
        "/dm" => return Some(InputAction::OpenDmPicker),
        "/savechat" => return Some(InputAction::OpenSaveChatPicker),
        _ => {}
    }

    if line.starts_with("/dm ") {
        return Some(InputAction::Error(
            "use /dm then Tab to pick a member".into(),
        ));
    }

    if line.starts_with("/savechat ") {
        return Some(InputAction::Error(
            "use /savechat then Tab to choose On or Off".into(),
        ));
    }

    Some(InputAction::ChannelMessage(line.to_string()))
}

pub fn member_style(name: &str) -> Style {
    Style::new()
        .fg(member_color(name))
        .add_modifier(Modifier::BOLD)
}

pub fn member_color(name: &str) -> Color {
    match crate::style::member_palette_index(name) {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Yellow,
        3 => Color::Blue,
        4 => Color::Magenta,
        5 => Color::Cyan,
        6 => Color::LightRed,
        7 => Color::LightGreen,
        8 => Color::LightBlue,
        _ => Color::LightMagenta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_spaces_in_nickname() {
        assert!(validate_nickname("farhaan 2").is_err());
        assert!(validate_nickname("farhaan_2").is_ok());
    }
}
