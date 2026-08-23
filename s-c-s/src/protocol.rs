use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Member {
    pub name: String,
}

/// Channel settings broadcast to all members.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelInfo {
    pub code: String,
    pub members: Vec<Member>,
    pub save_chat: bool,
    pub creator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Register { name: String },
    CreateChannel { save_chat: bool },
    JoinChannel { code: String },
    LeaveChannel,
    ChannelMessage { text: String },
    DirectMessage { to: String, text: String },
    SetSaveChat { enabled: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Registered { name: String },
    Error { message: String },
    ChannelCreated { info: ChannelInfo },
    ChannelJoined { info: ChannelInfo },
    ChannelLeft,
    MemberUpdate { info: ChannelInfo },
    SaveChatChanged { info: ChannelInfo },
    ChannelMessage { from: String, text: String },
    DirectMessage { from: String, text: String },
}

pub fn encode_client(msg: &ClientMessage) -> anyhow::Result<String> {
    Ok(serde_json::to_string(msg)?)
}

pub fn encode_server(msg: &ServerMessage) -> anyhow::Result<String> {
    Ok(serde_json::to_string(msg)?)
}

pub fn decode_client(line: &str) -> anyhow::Result<ClientMessage> {
    Ok(serde_json::from_str(line)?)
}

pub fn decode_server(line: &str) -> anyhow::Result<ServerMessage> {
    Ok(serde_json::from_str(line)?)
}

pub fn gen_channel_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    (0..6)
        .map(|i| {
            let idx = (seed as usize).wrapping_add(i * 17) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_client_message() {
        let msg = ClientMessage::CreateChannel { save_chat: true };
        let line = encode_client(&msg).unwrap();
        let decoded = decode_client(&line).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn channel_code_is_six_chars() {
        let code = gen_channel_code();
        assert_eq!(code.len(), 6);
    }
}
