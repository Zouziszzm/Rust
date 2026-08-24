---
title: Simple Channel Chat
subtext: Text-only CLI chat with channels. Create a channel, share a code, and let others join.
stack: [Rust, ratatui, TCP]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# s-c-s — Simple Channel Chat

Text-only CLI chat with channels. Create a channel, share a code, and let others join. Works on the same machine, LAN, or over [Tailscale](https://tailscale.com/).

## UI

- **Setup** uses [`inquire`](https://docs.rs/inquire) — arrow-key menus and prompts for server IP, nickname, create/join
- **In-channel chat** uses [`ratatui`](https://docs.rs/ratatui) — split view with messages, member list, and input box
- **Colors** via [`owo-colors`](https://docs.rs/owo-colors) + ratatui styles — each member has a stable color

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

## Test on one machine (no second computer)

Open **3 terminal tabs/windows** on the same Mac:

**Tab 1 — server**
```bash
cd s-c-s
cargo run --bin chat-server
```

**Tab 2 — first user**
```bash
cd s-c-s
cargo run
# Server IP: press Enter (uses 127.0.0.1)
# Nickname: alice
# Choose: 1 (Create a channel)
# Note the code, e.g. X7K9M2
```

**Tab 3 — second user (simulates another person)**
```bash
cd s-c-s
cargo run
# Server IP: press Enter
# Nickname: bob
# Choose: 2 (Join a channel)
# Enter code: X7K9M2
```

Now type in tab 2 and tab 3 — messages appear in both. Try `/dm bob hello` from alice's tab.

You can open a 4th tab for a third user anytime.

## Setup (normal use)

**On the host machine:**

```bash
cargo run --bin chat-server
```

**On each client:**

```bash
cargo run
```

## Colors

| Element | Style |
|---------|-------|
| Channel header | Cyan border, yellow code |
| Member names | Stable color per nickname (in messages + member panel) |
| Channel messages | Colored `[name]` + white text |
| Direct messages | Magenta `DM from/to` |
| System lines | Dim gray italic |

## Save chat (local history)

When **save chat** is enabled for a channel:

- Each client stores messages **on their own machine** at `~/.local/share/s-c-s/channels/<CODE>/messages.jsonl`
- Rejoining the same channel code loads your previous local history
- Only the **channel creator** can turn save chat on or off (`/savechat` in the TUI, Tab to pick On/Off)

When creating a channel you choose save chat on or off. Other members see the setting in the header (`save: on` / `save: off`).

## Commands (in the message box)

| Command | Action |
|---------|--------|
| `<message>` | Send to everyone in the channel |
| `/dm <name> <msg>` | Direct message to one member |
| `/members` | Highlight member list |
| `/savechat` | Toggle save chat — **creator only** (Tab picker) |
| `/switch` | Leave and create/join another channel |
| `/quit` | Leave channel and exit |
| `/help` | Toggle help overlay |

## Networking

| Scenario | Server IP to enter |
|----------|-------------------|
| Same machine | `127.0.0.1` (default) |
| Same Wi‑Fi / LAN | `lan` address from server output |
| Remote (no port forwarding) | `tailscale` address from server output |

Both machines need Tailscale installed and signed in for remote use.

## Tests

```bash
cargo test
```
