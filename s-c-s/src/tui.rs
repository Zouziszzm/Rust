use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossbeam_channel::Receiver;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::history;
use crate::protocol::{ChannelInfo, ClientMessage, ServerMessage};
use crate::session::{
    dm_candidates, member_style, ChatLine, ChannelState, InputAction, SessionEnd, parse_input,
};

const CHANNEL_ACTIONS: &[&str] = &["Create a channel", "Join a channel", "Back to menu"];
const SAVE_CHAT_OPTIONS: &[&str] = &["Save chat: On", "Save chat: Off"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerKind {
    DmRecipient,
    ChannelAction,
    SaveChatOnCreate,
    SaveChatToggle,
}

#[derive(Debug, Clone)]
struct Picker {
    kind: PickerKind,
    title: String,
    options: Vec<String>,
    selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputMode {
    Normal,
    DmMessage { to: String },
    JoinCode,
}

struct ChannelApp {
    state: ChannelState,
    in_channel: bool,
    messages: Vec<ChatLine>,
    input: String,
    input_mode: InputMode,
    scroll: usize,
    show_help: bool,
    picker: Option<Picker>,
    status: String,
    done: Option<SessionEnd>,
    pending_switch: bool,
}

impl ChannelApp {
    fn new(state: ChannelState) -> Self {
        let code = state.code.clone();
        let mut app = Self {
            state,
            in_channel: true,
            messages: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            scroll: 0,
            show_help: false,
            picker: None,
            status: "Type a message or /help".into(),
            done: None,
            pending_switch: false,
        };
        app.load_local_history();
        app.messages.push(ChatLine::System(format!(
            "Joined channel {code} — share the code so others can join"
        )));
        if app.state.save_chat {
            if let Ok(dir) = history::history_dir_display(&app.state.code) {
                app.messages.push(ChatLine::System(format!(
                    "Save chat on — stored at {dir}"
                )));
            }
        }
        app.scroll_to_bottom();
        app
    }

    fn load_local_history(&mut self) {
        if !self.state.save_chat {
            return;
        }
        if let Ok(mut past) = history::load_channel_history(&self.state.code) {
            if !past.is_empty() {
                past.push(ChatLine::System("─── saved history ───".into()));
                past.append(&mut self.messages);
                self.messages = past;
            }
        }
    }

    fn apply_info(&mut self, info: ChannelInfo) {
        let my_name = self.state.my_name.clone();
        self.state = ChannelState::from_info(info, &my_name);
    }

    fn maybe_save(&self, line: &ChatLine) {
        if !self.state.save_chat {
            return;
        }
        match line {
            ChatLine::Channel { .. } | ChatLine::DmReceived { .. } | ChatLine::DmSent { .. } => {
                let _ = history::append_line(&self.state.code, line);
            }
            _ => {}
        }
    }

    fn push_message(&mut self, line: ChatLine) {
        self.maybe_save(&line);
        self.messages.push(line);
        self.scroll_to_bottom();
    }

    fn open_dm_picker(&mut self) {
        let options = dm_candidates(&self.state);
        if options.is_empty() {
            self.status = "No other members to DM".into();
            return;
        }
        self.picker = Some(Picker {
            kind: PickerKind::DmRecipient,
            title: "Direct message — Tab to cycle, Enter to select".into(),
            options,
            selected: 0,
        });
        self.show_help = false;
    }

    fn open_channel_picker(&mut self) {
        self.picker = Some(Picker {
            kind: PickerKind::ChannelAction,
            title: "Change channel — Tab to cycle, Enter to select".into(),
            options: CHANNEL_ACTIONS.iter().map(|s| s.to_string()).collect(),
            selected: 0,
        });
        self.in_channel = false;
        self.show_help = false;
        self.status = "Pick an action".into();
    }

    fn picker_next(&mut self) {
        if let Some(picker) = &mut self.picker {
            if !picker.options.is_empty() {
                picker.selected = (picker.selected + 1) % picker.options.len();
            }
        }
    }

    fn picker_prev(&mut self) {
        if let Some(picker) = &mut self.picker {
            if !picker.options.is_empty() {
                picker.selected = if picker.selected == 0 {
                    picker.options.len() - 1
                } else {
                    picker.selected - 1
                };
            }
        }
    }

    fn cancel_overlay(&mut self) {
        self.picker = None;
        self.input_mode = InputMode::Normal;
        self.input.clear();
        if self.in_channel {
            self.status = "Cancelled".into();
        }
    }

    fn confirm_picker(&mut self, outgoing: &crossbeam_channel::Sender<ClientMessage>) {
        let Some(picker) = self.picker.take() else {
            return;
        };
        let Some(choice) = picker.options.get(picker.selected).cloned() else {
            return;
        };

        match picker.kind {
            PickerKind::DmRecipient => {
                self.input_mode = InputMode::DmMessage { to: choice };
                self.input.clear();
                self.status = "Type your DM and press Enter".into();
            }
            PickerKind::ChannelAction => match choice.as_str() {
                "Create a channel" => {
                    self.picker = Some(Picker {
                        kind: PickerKind::SaveChatOnCreate,
                        title: "Save chat on this device? Tab to cycle".into(),
                        options: SAVE_CHAT_OPTIONS.iter().map(|s| s.to_string()).collect(),
                        selected: 1,
                    });
                }
                "Join a channel" => {
                    self.input_mode = InputMode::JoinCode;
                    self.input.clear();
                    self.status = "Enter channel code and press Enter".into();
                }
                _ => {
                    self.done = Some(SessionEnd::Switch);
                }
            },
            PickerKind::SaveChatOnCreate => {
                let save_chat = picker.selected == 0;
                let _ = outgoing.send(ClientMessage::CreateChannel { save_chat });
                self.status = "Creating channel…".into();
            }
            PickerKind::SaveChatToggle => {
                if !self.state.is_creator() {
                    self.status = "Only the channel creator can change save chat".into();
                    return;
                }
                let enabled = picker.selected == 0;
                let _ = outgoing.send(ClientMessage::SetSaveChat { enabled });
                self.status = if enabled {
                    "Save chat turned on".into()
                } else {
                    "Save chat turned off".into()
                };
            }
        }
    }

    fn open_save_chat_picker(&mut self) {
        if !self.state.is_creator() {
            self.status = "Only the channel creator can change save chat".into();
            return;
        }
        self.picker = Some(Picker {
            kind: PickerKind::SaveChatToggle,
            title: "Save chat on all devices — Tab to cycle".into(),
            options: SAVE_CHAT_OPTIONS.iter().map(|s| s.to_string()).collect(),
            selected: if self.state.save_chat { 0 } else { 1 },
        });
        self.show_help = false;
    }

    fn handle_server(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::ChannelMessage { from, text } => {
                self.push_message(ChatLine::Channel { from, text });
            }
            ServerMessage::DirectMessage { from, text } => {
                self.push_message(ChatLine::DmReceived { from, text });
            }
            ServerMessage::MemberUpdate { info } => {
                let count = info.members.len();
                self.apply_info(info);
                self.in_channel = true;
                self.push_message(ChatLine::System(format!("{count} member(s) in channel")));
            }
            ServerMessage::ChannelCreated { info } => {
                let code = info.code.clone();
                self.apply_info(info);
                self.in_channel = true;
                self.input_mode = InputMode::Normal;
                self.messages.clear();
                self.load_local_history();
                self.messages
                    .push(ChatLine::System(format!("Channel created — code: {code}")));
                if self.state.save_chat {
                    self.messages.push(ChatLine::System(
                        "Save chat on — new messages saved on each device".into(),
                    ));
                }
                self.scroll_to_bottom();
                self.status = format!("Share code: {code}");
            }
            ServerMessage::ChannelJoined { info } => {
                let code = info.code.clone();
                self.apply_info(info);
                self.in_channel = true;
                self.input_mode = InputMode::Normal;
                self.messages.clear();
                self.load_local_history();
                self.messages
                    .push(ChatLine::System(format!("Joined channel {code}")));
                self.scroll_to_bottom();
                self.status = "Joined channel".into();
            }
            ServerMessage::SaveChatChanged { info } => {
                let on = info.save_chat;
                self.apply_info(info);
                self.messages.push(ChatLine::System(format!(
                    "Save chat {} by {}",
                    if on { "enabled" } else { "disabled" },
                    self.state.creator
                )));
                self.scroll_to_bottom();
                self.status = if on {
                    "Save chat is on".into()
                } else {
                    "Save chat is off".into()
                };
            }
            ServerMessage::Error { message } => {
                self.status = message;
            }
            ServerMessage::ChannelLeft => {
                if self.pending_switch {
                    self.pending_switch = false;
                    self.open_channel_picker();
                }
            }
            ServerMessage::Registered { .. } => {}
        }
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll = self.messages.len().saturating_sub(1);
    }

    fn submit(&mut self, outgoing: &crossbeam_channel::Sender<ClientMessage>) {
        if self.picker.is_some() {
            self.confirm_picker(outgoing);
            return;
        }

        match &self.input_mode {
            InputMode::DmMessage { to } => {
                let text = self.input.trim().to_string();
                if text.is_empty() {
                    return;
                }
                let to = to.clone();
                let line = ChatLine::DmSent {
                    to: to.clone(),
                    text: text.clone(),
                };
                self.push_message(line);
                let _ = outgoing.send(ClientMessage::DirectMessage { to, text });
                self.input.clear();
                self.input_mode = InputMode::Normal;
                self.scroll_to_bottom();
                self.status = "DM sent".into();
                return;
            }
            InputMode::JoinCode => {
                let code = self.input.trim().to_uppercase();
                if code.is_empty() {
                    self.status = "Enter a channel code".into();
                    return;
                }
                let _ = outgoing.send(ClientMessage::JoinChannel { code });
                self.input.clear();
                self.status = "Joining channel…".into();
                return;
            }
            InputMode::Normal => {}
        }

        let line = std::mem::take(&mut self.input);
        if line.trim().is_empty() {
            return;
        }

        if !self.in_channel {
            self.status = "Join or create a channel first".into();
            return;
        }

        match parse_input(&line, &self.state) {
            None => {}
            Some(InputAction::ChannelMessage(text)) => {
                let _ = outgoing.send(ClientMessage::ChannelMessage { text });
                self.status = "Message sent".into();
            }
            Some(InputAction::OpenDmPicker) => self.open_dm_picker(),
            Some(InputAction::OpenSaveChatPicker) => self.open_save_chat_picker(),
            Some(InputAction::ShowMembers) => {
                self.show_help = false;
                self.status = format!("{} member(s) — see panel →", self.state.members.len());
            }
            Some(InputAction::Help) => {
                self.show_help = !self.show_help;
                self.status = if self.show_help {
                    "Help open — /help to close".into()
                } else {
                    "Help closed".into()
                };
            }
            Some(InputAction::Switch) => {
                self.pending_switch = true;
                let _ = outgoing.send(ClientMessage::LeaveChannel);
                self.status = "Leaving channel…".into();
            }
            Some(InputAction::Quit) => {
                if self.in_channel {
                    let _ = outgoing.send(ClientMessage::LeaveChannel);
                }
                self.done = Some(SessionEnd::Quit);
            }
            Some(InputAction::Error(msg)) => self.status = msg,
        }
    }

    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        outgoing: &crossbeam_channel::Sender<ClientMessage>,
    ) -> Option<SessionEnd> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            if self.in_channel {
                let _ = outgoing.send(ClientMessage::LeaveChannel);
            }
            return Some(SessionEnd::Quit);
        }

        if self.picker.is_some() {
            match key.code {
                KeyCode::Tab | KeyCode::Down => self.picker_next(),
                KeyCode::BackTab | KeyCode::Up => self.picker_prev(),
                KeyCode::Enter => self.confirm_picker(outgoing),
                KeyCode::Esc => self.cancel_overlay(),
                _ => {}
            }
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                if self.input_mode != InputMode::Normal {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    self.status = "Cancelled".into();
                } else {
                    self.input.clear();
                    self.show_help = false;
                }
            }
            KeyCode::Enter => self.submit(outgoing),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                if self.input_mode == InputMode::JoinCode {
                    if c.is_ascii_alphanumeric() {
                        self.input.push(c.to_ascii_uppercase());
                    }
                } else {
                    self.input.push(c);
                }
                self.show_help = false;
            }
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::PageDown => {
                self.scroll = (self.scroll + 10).min(self.messages.len().saturating_sub(1));
            }
            _ => {}
        }

        None
    }
}

pub fn run_channel_session(
    state: ChannelState,
    incoming_rx: &Receiver<ServerMessage>,
    outgoing_tx: &crossbeam_channel::Sender<ClientMessage>,
) -> Result<SessionEnd> {
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, state, incoming_rx, outgoing_tx);
    ratatui::restore();
    result
}

fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    state: ChannelState,
    incoming_rx: &Receiver<ServerMessage>,
    outgoing_tx: &crossbeam_channel::Sender<ClientMessage>,
) -> Result<SessionEnd> {
    let mut app = ChannelApp::new(state);

    loop {
        while let Ok(msg) = incoming_rx.try_recv() {
            app.handle_server(msg);
        }

        if let Some(end) = app.done.take() {
            if app.in_channel {
                drain_channel_left(incoming_rx);
            }
            return Ok(end);
        }

        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let Some(end) = app.handle_key(key, outgoing_tx) {
                    if app.in_channel {
                        drain_channel_left(incoming_rx);
                    }
                    return Ok(end);
                }
            }
        }

        if incoming_rx.is_empty()
            && matches!(
                incoming_rx.try_recv(),
                Err(crossbeam_channel::TryRecvError::Disconnected)
            )
        {
            return Ok(SessionEnd::Quit);
        }
    }
}

fn drain_channel_left(incoming_rx: &Receiver<ServerMessage>) {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        if let Ok(ServerMessage::ChannelLeft) = incoming_rx.recv_timeout(Duration::from_millis(200))
        {
            return;
        }
    }
}

fn draw(frame: &mut Frame, app: &ChannelApp) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let header_text = if app.in_channel {
        let save = if app.state.save_chat {
            Span::styled("  |  save: on", Style::new().fg(Color::Green))
        } else {
            Span::styled("  |  save: off", Style::new().fg(Color::DarkGray))
        };
        Line::from(vec![
            Span::styled("Channel ", Style::new().fg(Color::Cyan).bold()),
            Span::styled(
                app.state.code.clone(),
                Style::new().fg(Color::Yellow).bold(),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("{} member(s)", app.state.members.len()),
                Style::new().fg(Color::LightCyan),
            ),
            save,
            Span::raw("  |  "),
            Span::styled(
                format!("you: {}", app.state.my_name),
                member_style(&app.state.my_name),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("No channel", Style::new().fg(Color::Yellow).bold()),
            Span::raw("  |  "),
            Span::styled(
                format!("you: {}", app.state.my_name),
                member_style(&app.state.my_name),
            ),
        ])
    };

    frame.render_widget(
        Paragraph::new(header_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" s-c-s ")
                .border_style(Style::new().fg(Color::Cyan)),
        ),
        chunks[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
        .split(chunks[1]);

    draw_messages(frame, body[0], app);
    draw_members(frame, body[1], app);

    let input_title = match &app.input_mode {
        InputMode::Normal => " Message ".to_string(),
        InputMode::DmMessage { to } => format!(" DM to {to} "),
        InputMode::JoinCode => " Channel code ".to_string(),
    };

    frame.render_widget(
        Paragraph::new(app.input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(input_title)
                    .border_style(Style::new().fg(Color::DarkGray)),
            )
            .style(Style::new().fg(Color::White)),
        chunks[2],
    );

    let footer_hint = if app.picker.is_some() {
        "Tab next  Shift+Tab prev  Enter select  Esc cancel"
    } else {
        "Enter send  Esc cancel  /dm picker  /switch change channel  /help"
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(&app.status, Style::new().fg(Color::DarkGray)),
            Span::raw("  |  "),
            Span::styled(footer_hint, Style::new().fg(Color::DarkGray)),
        ])),
        chunks[3],
    );

    if app.show_help {
        draw_help_popup(frame, area);
    }
    if let Some(picker) = &app.picker {
        draw_picker_popup(frame, area, picker);
    }
}

fn draw_picker_popup(frame: &mut Frame, area: Rect, picker: &Picker) {
    let popup = centered_rect(50, 40, area);
    frame.render_widget(Block::default().bg(Color::Black), popup);

    let items: Vec<ListItem> = picker
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let prefix = if i == picker.selected { "▸ " } else { "  " };
            let style = if i == picker.selected {
                if picker.kind == PickerKind::DmRecipient {
                    member_style(opt)
                } else {
                    Style::new().fg(Color::Yellow).bold()
                }
            } else if picker.kind == PickerKind::DmRecipient {
                member_style(opt).add_modifier(Modifier::DIM)
            } else {
                Style::new().fg(Color::Gray)
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::new().fg(Color::Cyan)),
                Span::styled(opt.clone(), style),
            ]))
        })
        .collect();

    let title = match picker.kind {
        PickerKind::DmRecipient => " Direct Message ",
        PickerKind::ChannelAction => " Change Channel ",
        PickerKind::SaveChatOnCreate | PickerKind::SaveChatToggle => " Save Chat ",
    };

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_bottom(picker.title.as_str())
                .border_style(Style::new().fg(Color::Magenta)),
        ),
        popup,
    );
}

fn draw_messages(frame: &mut Frame, area: Rect, app: &ChannelApp) {
    let lines: Vec<Line> = app
        .messages
        .iter()
        .map(|line| message_line(line, &app.state.my_name))
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Messages ")
                    .border_style(Style::new().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false })
            .scroll((app.scroll as u16, 0)),
        area,
    );
}

fn message_line(line: &ChatLine, my_name: &str) -> Line<'static> {
    match line {
        ChatLine::Channel { from, text } => Line::from(vec![
            Span::raw("["),
            Span::styled(from.clone(), member_style(from)),
            Span::raw("] "),
            Span::styled(
                text.clone(),
                if from == my_name {
                    Style::new().fg(Color::DarkGray)
                } else {
                    Style::new().fg(Color::White)
                },
            ),
        ]),
        ChatLine::DmReceived { from, text } => Line::from(vec![
            Span::styled("DM ", Style::new().fg(Color::Magenta).bold()),
            Span::styled("from ", Style::new().fg(Color::Magenta)),
            Span::styled(from.clone(), member_style(from)),
            Span::raw(" "),
            Span::styled(text.clone(), Style::new().fg(Color::White)),
        ]),
        ChatLine::DmSent { to, text } => Line::from(vec![
            Span::styled("DM ", Style::new().fg(Color::Magenta).bold()),
            Span::styled("to ", Style::new().fg(Color::LightMagenta)),
            Span::styled(to.clone(), member_style(to)),
            Span::raw(" "),
            Span::styled(text.clone(), Style::new().fg(Color::DarkGray)),
        ]),
        ChatLine::System(text) => {
            Line::from(Span::styled(text.clone(), Style::new().fg(Color::DarkGray).italic()))
        }
        ChatLine::Error(text) => {
            Line::from(Span::styled(text.clone(), Style::new().fg(Color::Red).bold()))
        }
    }
}

fn draw_members(frame: &mut Frame, area: Rect, app: &ChannelApp) {
    let items: Vec<ListItem> = app
        .state
        .members
        .iter()
        .map(|m| {
            let you = if m.name == app.state.my_name {
                Span::styled(" (you)", Style::new().fg(Color::DarkGray))
            } else {
                Span::raw("")
            };
            ListItem::new(Line::from(vec![
                Span::styled(m.name.clone(), member_style(&m.name)),
                you,
            ]))
        })
        .collect();

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Members ({}) ", app.state.members.len()))
                .border_style(Style::new().fg(Color::DarkGray)),
        ),
        area,
    );
}

fn draw_help_popup(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 52, area);
    frame.render_widget(Block::default().bg(Color::Black), popup);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("Commands", Style::new().fg(Color::Cyan).bold())),
            Line::from(""),
            Line::from("<message>     send to channel"),
            Line::from("/dm           pick member (Tab/Enter)"),
            Line::from("/savechat     toggle save chat (creator only)"),
            Line::from("/switch       change channel (Tab/Enter)"),
            Line::from("/members      member list on the right"),
            Line::from("/quit         exit"),
            Line::from("/help         toggle this panel"),
            Line::from(""),
            Line::from(Span::styled("Esc to close", Style::new().fg(Color::DarkGray))),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::new().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true }),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
