use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::thread;

use anyhow::{Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use inquire::{Confirm, Select, Text};
use owo_colors::OwoColorize;

use crate::protocol::{
    ClientMessage, ServerMessage, decode_server, encode_client,
};
use crate::session::{validate_nickname, ChannelState, SessionEnd};
use crate::style::{paint_channel_code, paint_error, paint_member, paint_success};
use crate::tui;
use crate::{normalize_addr, DEFAULT_ADDR};

pub fn run() -> Result<()> {
    print_banner();

    let addr = prompt_server_addr()?;
    let nickname = prompt_nickname()?;

    let mut stream = TcpStream::connect(&addr)
        .with_context(|| format!("could not connect to {addr} — is chat-server running?"))?;
    stream.set_nodelay(true)?;

    send(&mut stream, ClientMessage::Register { name: nickname.clone() })?;

    let (incoming_tx, incoming_rx) = unbounded::<ServerMessage>();
    let (outgoing_tx, outgoing_rx) = unbounded::<ClientMessage>();

    let reader_stream = stream.try_clone().context("failed to clone stream")?;
    let writer_stream = stream;

    thread::spawn(move || read_loop(reader_stream, incoming_tx));
    thread::spawn(move || write_loop(writer_stream, outgoing_rx));

    wait_for_registered(&incoming_rx)?;

    loop {
        match channel_menu(&incoming_rx, &outgoing_tx, &nickname)? {
            MenuChoice::Enter(channel) => match tui::run_channel_session(channel, &incoming_rx, &outgoing_tx)? {
                SessionEnd::Switch => continue,
                SessionEnd::Quit => break,
            },
            MenuChoice::Quit => break,
        }
    }

    println!("{}", "Goodbye.".bright_black());
    Ok(())
}

fn print_banner() {
    println!();
    println!(
        "  {} — simple channel chat",
        "s-c-s".cyan().bold()
    );
    println!("  {}\n", "text-only CLI".bright_black());
}

fn prompt_server_addr() -> Result<String> {
    let input = Text::new("Server IP")
        .with_default(DEFAULT_ADDR)
        .with_help_message("127.0.0.1 for local, or LAN / Tailscale IP")
        .prompt()?;
    Ok(normalize_addr(&input))
}

fn prompt_nickname() -> Result<String> {
    loop {
        let name = Text::new("Nickname")
            .with_help_message("No spaces — use letters, numbers, _ or - (e.g. farhaan_2)")
            .prompt()?;
        match validate_nickname(&name) {
            Ok(name) => return Ok(name),
            Err(msg) => println!("{}", paint_error(&msg)),
        }
    }
}

fn wait_for_registered(incoming_rx: &Receiver<ServerMessage>) -> Result<()> {
    match incoming_rx.recv().context("server disconnected")? {
        ServerMessage::Registered { name } => {
            println!("Connected as {}.\n", paint_member(&name));
            Ok(())
        }
        ServerMessage::Error { message } => anyhow::bail!(message),
        other => anyhow::bail!("unexpected server message: {other:?}"),
    }
}

fn channel_menu(
    incoming_rx: &Receiver<ServerMessage>,
    outgoing_tx: &Sender<ClientMessage>,
    my_name: &str,
) -> Result<MenuChoice> {
    loop {
        let choice = Select::new(
            "What would you like to do?",
            vec!["Create a channel", "Join a channel", "Quit"],
        )
        .prompt();

        match choice {
            Ok("Create a channel") => {
                let save_chat = Confirm::new("Save chat on this device?")
                    .with_help_message("When on, messages are stored locally in ~/.local/share/s-c-s")
                    .with_default(false)
                    .prompt()?;
                outgoing_tx.send(ClientMessage::CreateChannel { save_chat })?;
                return Ok(MenuChoice::Enter(wait_for_channel_entry(
                    incoming_rx, my_name,
                )?));
            }
            Ok("Join a channel") => {
                let code = Text::new("Channel code")
                    .with_help_message("6-character code from whoever created the channel")
                    .prompt()?;
                let code = code.trim().to_uppercase();
                if code.is_empty() {
                    println!("{}", paint_error("Code cannot be empty."));
                    continue;
                }
                outgoing_tx.send(ClientMessage::JoinChannel { code })?;
                return Ok(MenuChoice::Enter(wait_for_channel_entry(
                    incoming_rx, my_name,
                )?));
            }
            Ok("Quit") | Err(_) => return Ok(MenuChoice::Quit),
            _ => {}
        }
    }
}

enum MenuChoice {
    Enter(ChannelState),
    Quit,
}

fn wait_for_channel_entry(
    incoming_rx: &Receiver<ServerMessage>,
    my_name: &str,
) -> Result<ChannelState> {
    loop {
        match incoming_rx.recv().context("server disconnected")? {
            ServerMessage::ChannelCreated { info } => {
                println!();
                println!("{}", paint_success("Channel created!"));
                println!("Share this code: {}", paint_channel_code(&info.code));
                if info.save_chat {
                    println!("{}", "Save chat: on (this device)".bright_black());
                }
                println!();
                return Ok(ChannelState::from_info(info, my_name));
            }
            ServerMessage::ChannelJoined { info } => {
                println!();
                println!("Joined channel {}.", paint_channel_code(&info.code));
                if info.save_chat {
                    println!("{}", "Save chat: on (this device)".bright_black());
                }
                return Ok(ChannelState::from_info(info, my_name));
            }
            ServerMessage::Error { message } => anyhow::bail!(message),
            other => eprintln!("(ignored while joining: {other:?})"),
        }
    }
}

fn send(stream: &mut TcpStream, msg: ClientMessage) -> Result<()> {
    let line = encode_client(&msg)?;
    writeln!(stream, "{line}")?;
    stream.flush()?;
    Ok(())
}

fn read_loop(mut stream: TcpStream, incoming_tx: Sender<ServerMessage>) {
    let reader = io::BufReader::new(&mut stream);
    for line in reader.lines() {
        match line {
            Ok(line) if !line.trim().is_empty() => match decode_server(&line) {
                Ok(msg) => {
                    if incoming_tx.send(msg).is_err() {
                        break;
                    }
                }
                Err(err) => eprintln!("invalid server message: {err}"),
            },
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

fn write_loop(mut stream: TcpStream, outgoing_rx: Receiver<ClientMessage>) {
    for msg in outgoing_rx {
        if send(&mut stream, msg).is_err() {
            break;
        }
    }
}
