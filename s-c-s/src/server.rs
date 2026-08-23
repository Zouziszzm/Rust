use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::protocol::{
    self, ChannelInfo, ClientMessage, Member, ServerMessage, encode_server, gen_channel_code,
};

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

fn next_client_id() -> u64 {
    NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}

struct ClientRecord {
    name: String,
    outbound: Sender<String>,
    channel: Option<String>,
}

struct ChannelMeta {
    members: HashSet<u64>,
    creator_name: String,
    save_chat: bool,
}

pub enum ServerEvent {
    Register {
        id: u64,
        name: String,
        outbound: Sender<String>,
    },
    ClientMessage {
        id: u64,
        message: ClientMessage,
    },
    Disconnect {
        id: u64,
    },
}

pub fn run_hub(event_rx: Receiver<ServerEvent>) {
    let mut clients: HashMap<u64, ClientRecord> = HashMap::new();
    let mut channels: HashMap<String, ChannelMeta> = HashMap::new();

    for event in event_rx {
        match event {
            ServerEvent::Register {
                id,
                name,
                outbound,
            } => {
                match crate::session::validate_nickname(&name) {
                    Err(message) => {
                        let _ = outbound.send(
                            encode_server(&ServerMessage::Error { message })
                                .map(|l| format!("{l}\n"))
                                .unwrap_or_default(),
                        );
                        continue;
                    }
                    Ok(name) => {
                        clients.insert(
                            id,
                            ClientRecord {
                                name: name.clone(),
                                outbound,
                                channel: None,
                            },
                        );
                        send_to_client(&clients, id, ServerMessage::Registered { name });
                    }
                }
            }
            ServerEvent::ClientMessage { id, message } => {
                handle_client_message(id, message, &mut clients, &mut channels);
            }
            ServerEvent::Disconnect { id } => {
                remove_client(id, &mut clients, &mut channels);
            }
        }
    }
}

fn handle_client_message(
    id: u64,
    message: ClientMessage,
    clients: &mut HashMap<u64, ClientRecord>,
    channels: &mut HashMap<String, ChannelMeta>,
) {
    match message {
        ClientMessage::Register { .. } => {
            send_error(clients, id, "already registered");
        }
        ClientMessage::CreateChannel { save_chat } => {
            if clients.get(&id).and_then(|c| c.channel.as_ref()).is_some() {
                send_error(clients, id, "leave your current channel first");
                return;
            }

            let creator_name = clients
                .get(&id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "unknown".into());

            let code = unique_channel_code(&channels);
            channels.insert(
                code.clone(),
                ChannelMeta {
                    members: HashSet::from([id]),
                    creator_name: creator_name.clone(),
                    save_chat,
                },
            );
            if let Some(client) = clients.get_mut(&id) {
                client.channel = Some(code.clone());
            }

            let info = channel_info(&code, &channels, &clients);
            send_to_client(clients, id, ServerMessage::ChannelCreated { info });
        }
        ClientMessage::JoinChannel { code } => {
            let code = code.trim().to_uppercase();
            if clients.get(&id).and_then(|c| c.channel.as_ref()).is_some() {
                send_error(clients, id, "leave your current channel first");
                return;
            }

            let Some(meta) = channels.get_mut(&code) else {
                send_error(clients, id, "channel not found — check the code");
                return;
            };

            meta.members.insert(id);
            if let Some(client) = clients.get_mut(&id) {
                client.channel = Some(code.clone());
            }

            let info = channel_info(&code, channels, clients);
            send_to_client(
                clients,
                id,
                ServerMessage::ChannelJoined {
                    info: info.clone(),
                },
            );
            broadcast_member_update(&code, clients, channels, Some(id));
        }
        ClientMessage::SetSaveChat { enabled } => {
            let Some(code) = clients.get(&id).and_then(|c| c.channel.clone()) else {
                send_error(clients, id, "join a channel first");
                return;
            };

            let Some(meta) = channels.get(&code) else {
                send_error(clients, id, "channel not found");
                return;
            };

            let name = clients.get(&id).map(|c| c.name.as_str()).unwrap_or("");
            if !name.eq_ignore_ascii_case(&meta.creator_name) {
                send_error(clients, id, "only the channel creator can change save chat");
                return;
            }

            if let Some(meta) = channels.get_mut(&code) {
                meta.save_chat = enabled;
            }

            let info = channel_info(&code, channels, clients);
            broadcast_save_chat_changed(&code, &info, clients, channels);
        }
        ClientMessage::LeaveChannel => {
            leave_channel(id, clients, channels);
        }
        ClientMessage::ChannelMessage { text } => {
            let text = text.trim().to_string();
            if text.is_empty() {
                return;
            }

            let Some(code) = clients.get(&id).and_then(|c| c.channel.clone()) else {
                send_error(clients, id, "join a channel first");
                return;
            };

            let from = clients
                .get(&id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "unknown".into());

            broadcast_channel_message(&code, &from, &text, clients, channels);
        }
        ClientMessage::DirectMessage { to, text } => {
            let text = text.trim().to_string();
            if text.is_empty() {
                return;
            }

            let Some(code) = clients.get(&id).and_then(|c| c.channel.clone()) else {
                send_error(clients, id, "join a channel first");
                return;
            };

            let from_name = clients.get(&id).map(|c| c.name.as_str()).unwrap_or("unknown");
            let target_id = find_member_in_channel(&code, &to, id, clients, channels);

            let Some(target_id) = target_id else {
                send_error(clients, id, "member not found in this channel");
                return;
            };

            send_to_client(
                clients,
                target_id,
                ServerMessage::DirectMessage {
                    from: from_name.to_string(),
                    text,
                },
            );
        }
    }
}

fn unique_channel_code(channels: &HashMap<String, ChannelMeta>) -> String {
    loop {
        let code = gen_channel_code();
        if !channels.contains_key(&code) {
            return code;
        }
    }
}

fn channel_info(
    code: &str,
    channels: &HashMap<String, ChannelMeta>,
    clients: &HashMap<u64, ClientRecord>,
) -> ChannelInfo {
    let meta = channels.get(code);
    ChannelInfo {
        code: code.to_string(),
        members: members_in_channel(code, clients, channels),
        save_chat: meta.map(|m| m.save_chat).unwrap_or(false),
        creator: meta
            .map(|m| m.creator_name.clone())
            .unwrap_or_default(),
    }
}

fn members_in_channel(
    code: &str,
    clients: &HashMap<u64, ClientRecord>,
    channels: &HashMap<String, ChannelMeta>,
) -> Vec<Member> {
    let mut names: Vec<_> = channels
        .get(code)
        .into_iter()
        .flat_map(|m| m.members.iter())
        .filter_map(|id| clients.get(id).map(|c| Member { name: c.name.clone() }))
        .collect();
    names.sort_by(|a, b| a.name.cmp(&b.name));
    names
}

fn find_member_in_channel(
    code: &str,
    name: &str,
    self_id: u64,
    clients: &HashMap<u64, ClientRecord>,
    channels: &HashMap<String, ChannelMeta>,
) -> Option<u64> {
    let name = name.trim();
    channels.get(code)?.members.iter().copied().find(|&id| {
        id != self_id
            && clients
                .get(&id)
                .is_some_and(|c| c.name.eq_ignore_ascii_case(name))
    })
}

fn leave_channel(
    id: u64,
    clients: &mut HashMap<u64, ClientRecord>,
    channels: &mut HashMap<String, ChannelMeta>,
) {
    let Some(code) = clients.get(&id).and_then(|c| c.channel.clone()) else {
        send_error(clients, id, "you are not in a channel");
        return;
    };

    if let Some(client) = clients.get_mut(&id) {
        client.channel = None;
    }

    if let Some(meta) = channels.get_mut(&code) {
        meta.members.remove(&id);
        if meta.members.is_empty() {
            channels.remove(&code);
        } else {
            broadcast_member_update(&code, clients, channels, Some(id));
        }
    }

    send_to_client(clients, id, ServerMessage::ChannelLeft);
}

fn remove_client(
    id: u64,
    clients: &mut HashMap<u64, ClientRecord>,
    channels: &mut HashMap<String, ChannelMeta>,
) {
    let code = clients.get(&id).and_then(|c| c.channel.clone());
    clients.remove(&id);

    if let Some(code) = code {
        if let Some(meta) = channels.get_mut(&code) {
            meta.members.remove(&id);
            if meta.members.is_empty() {
                channels.remove(&code);
            } else {
                broadcast_member_update(&code, clients, channels, Some(id));
            }
        }
    }
}

fn broadcast_member_update(
    code: &str,
    clients: &HashMap<u64, ClientRecord>,
    channels: &HashMap<String, ChannelMeta>,
    except: Option<u64>,
) {
    let info = channel_info(code, channels, clients);
    let msg = ServerMessage::MemberUpdate { info };

    if let Some(meta) = channels.get(code) {
        for &id in &meta.members {
            if except == Some(id) {
                continue;
            }
            send_to_client(clients, id, msg.clone());
        }
    }
}

fn broadcast_save_chat_changed(
    code: &str,
    info: &ChannelInfo,
    clients: &HashMap<u64, ClientRecord>,
    channels: &HashMap<String, ChannelMeta>,
) {
    let msg = ServerMessage::SaveChatChanged {
        info: info.clone(),
    };
    if let Some(meta) = channels.get(code) {
        for &id in &meta.members {
            send_to_client(clients, id, msg.clone());
        }
    }
}

fn broadcast_channel_message(
    code: &str,
    from: &str,
    text: &str,
    clients: &HashMap<u64, ClientRecord>,
    channels: &HashMap<String, ChannelMeta>,
) {
    let msg = ServerMessage::ChannelMessage {
        from: from.to_string(),
        text: text.to_string(),
    };

    if let Some(meta) = channels.get(code) {
        for &id in &meta.members {
            send_to_client(clients, id, msg.clone());
        }
    }
}

fn send_error(clients: &HashMap<u64, ClientRecord>, id: u64, message: &str) {
    send_to_client(
        clients,
        id,
        ServerMessage::Error {
            message: message.to_string(),
        },
    );
}

fn send_to_client(clients: &HashMap<u64, ClientRecord>, id: u64, message: ServerMessage) {
    let Some(client) = clients.get(&id) else {
        return;
    };

    match encode_server(&message) {
        Ok(line) => {
            let _ = client.outbound.send(format!("{line}\n"));
        }
        Err(err) => eprintln!("encode error: {err}"),
    }
}

pub fn handle_client(stream: TcpStream, event_tx: Sender<ServerEvent>) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.set_nodelay(true)?;

    let id = next_client_id();
    let reader = stream.try_clone().context("failed to clone stream for reader")?;
    let mut writer = stream;

    let (outbound_tx, outbound_rx) = mpsc::channel::<String>();

    let writer_handle = thread::spawn(move || -> Result<()> {
        for line in outbound_rx {
            writer
                .write_all(line.as_bytes())
                .context("failed to write to client")?;
            writer.flush().ok();
        }
        Ok(())
    });

    let mut reader = BufReader::new(reader);
    let mut register_line = String::new();
    reader
        .read_line(&mut register_line)
        .context("failed to read register message")?;

    let register = protocol::decode_client(register_line.trim())
        .context("expected register message")?;

    let name = match register {
        ClientMessage::Register { name } => match crate::session::validate_nickname(&name) {
            Ok(name) => name,
            Err(message) => anyhow::bail!(message),
        },
        _ => anyhow::bail!("first message must be register"),
    };

    event_tx
        .send(ServerEvent::Register {
            id,
            name: name.clone(),
            outbound: outbound_tx,
        })
        .context("hub disconnected")?;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match protocol::decode_client(trimmed) {
                    Ok(message) => {
                        if event_tx
                            .send(ServerEvent::ClientMessage { id, message })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(err) => eprintln!("invalid client message from {id}: {err}"),
                }
            }
            Err(_) => break,
        }
    }

    let _ = event_tx.send(ServerEvent::Disconnect { id });
    drop(writer_handle);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{decode_server, encode_client};
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};

    fn send_json(stream: &mut TcpStream, msg: &ClientMessage) {
        let line = encode_client(msg).unwrap();
        writeln!(stream, "{line}").unwrap();
        stream.flush().unwrap();
    }

    fn read_json(stream: &mut TcpStream) -> ServerMessage {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode_server(line.trim()).unwrap()
    }

    #[test]
    fn create_join_and_message() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let (event_tx, event_rx) = mpsc::channel::<ServerEvent>();
        thread::spawn(move || run_hub(event_rx));

        let accept = thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let event_tx = event_tx.clone();
                thread::spawn(move || {
                    let _ = handle_client(stream, event_tx);
                });
            }
        });

        let mut alice = TcpStream::connect(addr)?;
        send_json(
            &mut alice,
            &ClientMessage::Register {
                name: "alice".into(),
            },
        );
        assert!(matches!(read_json(&mut alice), ServerMessage::Registered { .. }));

        send_json(&mut alice, &ClientMessage::CreateChannel { save_chat: false });
        let code = match read_json(&mut alice) {
            ServerMessage::ChannelCreated { info } => {
                assert_eq!(info.members.len(), 1);
                info.code
            }
            other => panic!("expected ChannelCreated, got {other:?}"),
        };

        let mut bob = TcpStream::connect(addr)?;
        send_json(
            &mut bob,
            &ClientMessage::Register {
                name: "bob".into(),
            },
        );
        let _ = read_json(&mut bob);

        send_json(
            &mut bob,
            &ClientMessage::JoinChannel {
                code: code.clone(),
            },
        );
        let _ = read_json(&mut bob);

        thread::sleep(Duration::from_millis(50));
        let update = read_json(&mut alice);
        assert!(matches!(update, ServerMessage::MemberUpdate { .. }));

        send_json(
            &mut alice,
            &ClientMessage::ChannelMessage {
                text: "hello bob".into(),
            },
        );

        thread::sleep(Duration::from_millis(50));
        let msg = read_json(&mut bob);
        assert!(matches!(
            msg,
            ServerMessage::ChannelMessage { .. }
        ));

        drop(alice);
        drop(bob);
        drop(accept);
        Ok(())
    }
}
