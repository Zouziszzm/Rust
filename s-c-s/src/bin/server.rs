use std::net::TcpListener;

use anyhow::Result;
use clap::Parser;
use scs::tailscale::listen_addresses;
use scs::{default_bind_addr, DEFAULT_PORT};

#[derive(Parser)]
#[command(name = "chat-server", about = "TCP chat server for s-c-s channels")]
struct Args {
    /// Address to bind (0.0.0.0 accepts LAN and Tailscale connections)
    #[arg(long, default_value_t = default_bind_addr())]
    bind: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let port = args
        .bind
        .rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let listener = TcpListener::bind(&args.bind)?;
    print_listen_info(&args.bind, port);

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || scs::server::run_hub(event_rx));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let event_tx = event_tx.clone();
                std::thread::spawn(move || {
                    if let Err(err) = scs::server::handle_client(stream, event_tx) {
                        eprintln!("client error: {err:#}");
                    }
                });
            }
            Err(err) => eprintln!("accept error: {err}"),
        }
    }

    Ok(())
}

fn print_listen_info(bind: &str, port: u16) {
    let addrs = listen_addresses(bind, port);

    println!("chat-server listening on {}", addrs.bind);
    println!("  local:     {}", addrs.localhost);

    if let Some(lan) = &addrs.lan {
        println!("  lan:       {lan}");
    }

    match &addrs.tailscale {
        Some(ts) => println!("  tailscale: {ts}"),
        None => println!("  tailscale: (not detected)"),
    }

    println!();
    println!("Clients run: cargo run  (then enter one of the addresses above)");
}
