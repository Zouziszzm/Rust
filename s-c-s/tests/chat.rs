use std::io::{BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use scs::protocol::{ClientMessage, ServerMessage, decode_server, encode_client};
use scs::server::{handle_client, run_hub};

fn send_json(stream: &mut TcpStream, msg: &ClientMessage) {
    let line = encode_client(msg).unwrap();
    writeln!(stream, "{line}").unwrap();
    stream.flush().unwrap();
}

fn read_json(stream: &mut TcpStream) -> ServerMessage {
    let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    decode_server(line.trim()).unwrap()
}

#[test]
fn channel_create_join_and_dm() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");

    let (event_tx, event_rx) = mpsc::channel();
    thread::spawn(move || run_hub(event_rx));

    let accept = thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let event_tx = event_tx.clone();
            thread::spawn(move || {
                let _ = handle_client(stream, event_tx);
            });
        }
    });

    let mut alice = TcpStream::connect(addr).expect("alice");
    send_json(
        &mut alice,
        &ClientMessage::Register {
            name: "alice".into(),
        },
    );
    let _ = read_json(&mut alice);

    send_json(
        &mut alice,
        &ClientMessage::CreateChannel { save_chat: true },
    );
    let code = match read_json(&mut alice) {
        ServerMessage::ChannelCreated { info } => {
            assert!(info.save_chat);
            info.code
        }
        other => panic!("expected ChannelCreated, got {other:?}"),
    };

    let mut bob = TcpStream::connect(addr).expect("bob");
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
    let _ = read_json(&mut alice);

    send_json(
        &mut alice,
        &ClientMessage::DirectMessage {
            to: "bob".into(),
            text: "psst".into(),
        },
    );

    thread::sleep(Duration::from_millis(50));
    let dm = read_json(&mut bob);
    assert!(matches!(dm, ServerMessage::DirectMessage { .. }));

    drop(alice);
    drop(bob);
    drop(accept);
}
