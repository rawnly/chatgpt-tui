mod cursor;
mod models;
mod openai;
mod stateful_list;

use models::*;
use std::fmt::Display;
use std::io::{self, Result};

use crate::openai::send_message;
use crate::stateful_list::StatefulList;

use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::Backend,
    layout::{Direction, Layout},
    prelude::*,
    widgets::*,
    Frame,
};

use crate::cursor::Cursor;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event};
use std::time::{Duration, Instant};
use std::usize::MAX;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // logic
    let app = App::default();
    let tick_rate = Duration::from_millis(250);

    run_app(&mut terminal, app, tick_rate).await?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum Role {
    User,
    Assistant,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Section {
    Chats,
    Messages,
    Input,
}

#[derive(Clone)]
struct App {
    loading: bool,
    active_chat: Option<usize>,
    list: StatefulList<Chat>,
    input: String,
    section: Section,
    active_section: Option<Section>,
    cursor: Cursor,
}

impl Default for App {
    fn default() -> Self {
        Self {
            loading: false,
            section: Section::Chats,
            active_section: Some(Section::Chats),
            input: String::new(),
            cursor: Cursor::default(),
            active_chat: None,
            list: StatefulList::with_items(vec![
                Chat::new("Demo"),
                Chat::with_messages(
                    "Christmas",
                    vec![
                        Message::new(
                            Role::User,
                            "What is christmas?"
                        ),
                        Message::new(
                            Role::Assistant,
                            "Christmas is a religious holiday celebrating the birth of Jesus as well as a cultural and commercial event. Learn about the history of Christmas, Santa Claus, and holiday traditions worldwide."
                        )
                    ]
                )
            ])
        }
    }
}

enum Action {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Char(char),
    Key(KeyCode),
    Backspace,
}

impl App {
    async fn dispatch(&mut self, action: Action) -> anyhow::Result<()> {
        match &self.active_section {
            None => match self.section {
                Section::Chats => match action {
                    Action::Enter => {
                        self.active_section = Some(Section::Chats);
                        self.list.next()
                    }
                    Action::Left | Action::Right => {
                        self.section = Section::Messages;
                    }
                    _ => {}
                },
                Section::Messages => match action {
                    Action::Enter => {
                        self.active_section = Some(Section::Messages);
                    }
                    Action::Left | Action::Right => {
                        self.section = Section::Chats;
                    }
                    Action::Up | Action::Down => {
                        self.section = Section::Input;
                    }
                    _ => {}
                },
                Section::Input => match action {
                    Action::Enter => {
                        self.active_section = Some(Section::Input);
                    }
                    Action::Left | Action::Right => {
                        self.section = Section::Chats;
                    }
                    Action::Up | Action::Down => {
                        self.section = Section::Messages;
                    }
                    _ => {}
                },
            },
            Some(section) => match section {
                Section::Chats => match action {
                    Action::Up => self.list.prev(),
                    Action::Down => self.list.next(),
                    Action::Esc => {
                        self.active_section = None;
                        self.list.unselect();
                        self.active_chat = None;
                    }
                    Action::Enter => {
                        self.active_chat = Some(self.list.state.selected().unwrap());
                        self.section = Section::Input;
                        self.active_section = Some(Section::Input);
                    }
                    _ => {}
                },
                Section::Messages => {
                    if let Some(chat) = self.get_active_chat_mut() {
                        match action {
                            Action::Backspace => self.delete_message(),
                            Action::Esc => {
                                self.active_section = None;
                            }
                            Action::Up => chat.messages.prev(),
                            Action::Down => chat.messages.next(),
                            _ => {}
                        }
                    }
                }
                Section::Input => match action {
                    Action::Enter => {
                        self.submit_message().await?;
                    }
                    Action::Char(to_enter) => self.enter_char(to_enter),
                    Action::Backspace => self.delete_char(),
                    Action::Left => self.cursor.left(),
                    Action::Right => self.cursor.right(),
                    Action::Esc => {
                        self.active_section = None;
                    }
                    _ => {}
                },
            },
        };

        Ok(())
    }

    fn get_active_chat_mut(&mut self) -> Option<&mut Chat> {
        match self.active_chat {
            Some(index) => self.list.items.get_mut(index),
            None => None,
        }
    }

    fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor.position, c);
        self.cursor.update_input_length(&self.input);
        self.cursor.right();
    }

    fn delete_message(&mut self) {
        if let Some(chat) = self.get_active_chat_mut() {
            if let Some(index) = chat.messages.state.selected() {
                chat.messages.prev();
                chat.messages.items.remove(index);
            }
        }
    }

    async fn submit_message(&mut self) -> anyhow::Result<()> {
        if self.input.is_empty() {
            self.input.clear();
            self.cursor.reset();

            return Ok(());
        }

        self.loading = true;

        let message = Message::new(Role::User, trim_spaces(&self.input.clone()).as_str());

        self.input.clear();
        self.cursor.reset();

        if let Some(active_chat_index) = self.active_chat {
            if let Some(chat) = self.list.items.get_mut(active_chat_index) {
                chat.append_message(message);

                if let Some(m) = send_message(chat.clone()).await? {
                    chat.append_message(m);
                }
            }
        }

        self.input.clear();
        self.cursor.reset();

        self.loading = false;

        Ok(())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = !self.cursor.is_at_start();

        if is_not_cursor_leftmost {
            let current_index = self.cursor.position;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);

            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.cursor.update_input_length(&self.input);
            self.cursor.left();
        }
    }
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> anyhow::Result<()> {
    app.list.select_first();

    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let elapsed = last_tick.elapsed();
        let timeout = tick_rate.saturating_sub(elapsed);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match &app.active_section {
                    Some(s) => match s {
                        Section::Input => match key.code {
                            KeyCode::Enter => app.dispatch(Action::Enter).await?,
                            KeyCode::Esc => app.dispatch(Action::Esc).await?,
                            KeyCode::Left => app.dispatch(Action::Left).await?,
                            KeyCode::Down => app.dispatch(Action::Down).await?,
                            KeyCode::Up => app.dispatch(Action::Up).await?,
                            KeyCode::Right => app.dispatch(Action::Right).await?,
                            KeyCode::Char(to_enter) => app.dispatch(Action::Char(to_enter)).await?,
                            KeyCode::Backspace => app.dispatch(Action::Backspace).await?,
                            keycode => app.dispatch(Action::Key(keycode)).await?,
                        },
                        _ => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Enter => app.dispatch(Action::Enter).await?,
                            KeyCode::Esc => app.dispatch(Action::Esc).await?,
                            KeyCode::Backspace => app.dispatch(Action::Backspace).await?,
                            KeyCode::Left | KeyCode::Char('h') => {
                                app.dispatch(Action::Left).await?
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.dispatch(Action::Down).await?
                            }
                            KeyCode::Up | KeyCode::Char('k') => app.dispatch(Action::Up).await?,
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.dispatch(Action::Right).await?
                            }
                            keycode => app.dispatch(Action::Key(keycode)).await?,
                        },
                    },
                    None => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Enter => app.dispatch(Action::Enter).await?,
                        KeyCode::Backspace => app.dispatch(Action::Backspace).await?,
                        KeyCode::Left | KeyCode::Char('h') => app.dispatch(Action::Left).await?,
                        KeyCode::Down | KeyCode::Char('j') => app.dispatch(Action::Down).await?,
                        KeyCode::Up | KeyCode::Char('k') => app.dispatch(Action::Up).await?,
                        KeyCode::Right | KeyCode::Char('l') => app.dispatch(Action::Right).await?,
                        keycode => app.dispatch(Action::Key(keycode)).await?,
                    },
                };
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(f.size());

    let messages_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(main_layout[1]);

    let chats: Vec<ListItem> = app
        .list
        .items
        .iter()
        .map(|chat| {
            let lines: Vec<Line> = vec![chat.title.clone().into()];

            ListItem::new(lines).style(Style::default())
        })
        .collect();

    let chats = List::new(chats)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(get_section_style(app, Section::Chats))
                .title("Chats"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("* ");

    f.render_stateful_widget(chats, main_layout[0], &mut app.list.state);

    let messages_style = get_section_style(app, Section::Messages);

    match app.get_active_chat_mut() {
        None => {
            let list = List::new(Vec::<ListItem>::new()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(messages_style)
                    .title("Messages (0)"),
            );

            f.render_widget(list, messages_chunks[0]);
        }
        Some(chat) => {
            let messages: Vec<ListItem> = chat
                .messages
                .items
                .iter()
                .map(|msg| {
                    // split content into lines of 30 characters
                    let mut content = msg.content.clone();
                    let alignment = match msg.role {
                        Role::User => Alignment::Right,
                        Role::Assistant => Alignment::Left,
                    };

                    let mut lines: Vec<Line> = Vec::new();

                    let max_line_length = 100; // messages_chunks[0].width as usize - 50;

                    if content.len() <= max_line_length {
                        let line = Line::raw(content).alignment(alignment);
                        lines.push(line);
                    } else {
                        let mut content_as_lines = into_lines(content.clone(), max_line_length);
                        lines.append(&mut content_as_lines);

                        // while content.len() >= max_line_length {
                        //     let line = Line::raw(trim_spaces(
                        //         &content.drain(..max_line_length).collect::<String>(),
                        //     ))
                        //     .alignment(alignment);
                        //
                        //     lines.push(line);
                        // }

                        // if !content.is_empty() {
                        //     let line = Line::raw(trim_spaces(&content)).alignment(alignment);
                        //     lines.push(line);
                        // }
                    }

                    ListItem::new(lines).style(Style::default())
                })
                .collect();

            let messages = List::new(messages)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(messages_style)
                        .title(format!("Messages ({})", chat.messages.items.len())),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(messages, messages_chunks[0], &mut chat.messages.state);
        }
    };

    let title = match app.loading {
        true => "Input (Loading...)".to_string(),
        false => format!("Input ({}/250)", app.input.len()),
    };

    f.render_widget(
        Paragraph::new(app.input.to_string())
            .style(match app.active_section {
                Some(Section::Input) => Style::default().fg(Color::Green),
                _ => Style::default(),
            })
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(get_section_style(app, Section::Input))
                    .title(title),
            ),
        messages_chunks[1],
    );

    if matches!(app.active_section, Some(Section::Input)) {
        f.set_cursor(
            messages_chunks[1].x + app.cursor.position as u16 + 1,
            messages_chunks[1].y + 1,
        );
    }
}

enum SectionStatus {
    Hovered,
    Focused,
    Normal,
}

fn get_style(status: SectionStatus) -> Style {
    match status {
        SectionStatus::Hovered => Style::default().fg(Color::Yellow),
        SectionStatus::Focused => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        SectionStatus::Normal => Style::default(),
    }
}

fn get_section_style(app: &mut App, section: Section) -> Style {
    match &app.active_section {
        Some(selected) if selected == &section => get_style(SectionStatus::Focused),
        _ => match &app.section {
            s if s == &section => get_style(SectionStatus::Hovered),
            _ => get_style(SectionStatus::Normal),
        },
    }
}

fn trim_spaces(s: &str) -> String {
    let re = regex::Regex::new(r"^\s+|\s+$").unwrap();

    re.replace_all(s, "").to_string()
}

fn into_lines(str: String, max_length: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();

    let mut current_line_length = 0;
    let mut content = String::new();
    let words = str.split(' ');

    for word in words {
        let word_length = word.len();

        if current_line_length + word_length > max_length {
            lines.push(Line::raw(trim_spaces(&content.clone())));
            content.clear();
            current_line_length = 0;
        } else {
            content.push_str(word);
            content.push(' ');
            current_line_length += word_length;
        }
    }

    lines
}
