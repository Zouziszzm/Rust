---
title: Simple Channel Chat
subtext: Text-only CLI chat with channels. Create a channel, share a code, and let others join.
order: 3
portfolioMode: summary
stack: [Rust, ratatui, TCP]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# s-c-s — Simple Channel Chat

## Portfolio

**s-c-s** (Simple Channel Chat) is a text-only CLI chat app built around channels. Create a room, share a short code, and anyone with the code can join — on the same machine, over LAN, or through Tailscale without opening ports on the public internet.

There are no accounts and no web UI. A small TCP server holds channels in memory; clients connect with a nickname and either create a new channel or join an existing one by code. Broadcast messages go to everyone in the channel; `/dm` sends a private message to one member. The creator can toggle save-chat so each client keeps a local history file under `~/.local/share/s-c-s/`.

The interface is split into two phases: **setup** (server IP, nickname, create/join) via `inquire` arrow-key prompts, then **in-channel chat** as a `ratatui` TUI — messages on the left, member list on the right, input box at the bottom. Each nickname gets a stable color via `owo-colors`, so conversations stay readable in a busy channel.

I wanted something lightweight for quick coordination between machines I already control — pairing sessions, debugging together, or chatting on a LAN without installing Slack or Discord.

### UI

- **Setup** uses `inquire` — arrow-key menus and prompts for server IP, nickname, create/join
- **In-channel chat** uses `ratatui` — split view with messages, member list, and input box
- **Colors** via `owo-colors` + ratatui styles — each member has a stable color

```
┌─ s-c-s ──────────────────────────────────────────────────────┐
│ Channel ABC123  |  2 member(s)  |  you: alice                │
├──────────────────────────────┬───────────────────────────────┤
│ Messages                     │ Members (2)                   │
│ [alice] hello                │ alice (you)                   │
│ [bob] hi there               │ bob                           │
├──────────────────────────────┴───────────────────────────────┤
│ Message                                                      │
│ > hello_                                                     │
├──────────────────────────────────────────────────────────────┤
│ Type a message or /help  |  Enter send  PgUp/Dn scroll       │
└──────────────────────────────────────────────────────────────┘
```

**TUI keys:** Enter send · Esc clear · PgUp/PgDn scroll · `/help` toggle help · Ctrl+C quit

## Development

```bash
cargo run --bin chat-server
cargo run
cargo test
```
