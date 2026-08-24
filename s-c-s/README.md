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

Text-only CLI chat with channels. Create a channel, share a code, and let others join. Works on the same machine, LAN, or over Tailscale.

### UI

- **Setup** uses `inquire` — arrow-key menus and prompts for server IP, nickname, create/join
- **In-channel chat** uses `ratatui` — split view with messages, member list, and input box
- **Colors** via `owo-colors` + ratatui styles — each member has a stable color

```
┌─ s-c-s ────────────────────────────────────────────────────────┐
│ Channel ABC123  |  2 member(s)  |  you: alice                 │
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
