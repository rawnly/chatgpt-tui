use crate::components::*;
use crate::models::*;
use crate::state::*;
use crate::utils::*;

use ratatui::{
    layout::{Direction, Layout},
    prelude::*,
    widgets::*,
    Frame,
};

fn render_chats(f: &mut Frame, app: &mut App, area: Rect) {
    let chats: Vec<ListItem> = app
        .chats
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
                .border_style(get_section_border_style(app, Section::Chats))
                .title("Chats"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("* ");

    f.render_stateful_widget(chats, area, &mut app.chats.state);
}

fn render_messages(f: &mut Frame, app: &mut App, area: Rect) {
    let messages_style = get_section_border_style(app, Section::Messages);

    match app.get_active_chat_mut() {
        None => {
            let list = List::new(Vec::<ListItem>::new()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(messages_style)
                    .title("Messages (0)"),
            );

            f.render_widget(list, area);
        }
        Some(chat) => {
            let messages: Vec<ListItem> = chat
                .messages
                .items
                .iter()
                .map(|msg| {
                    // split content into lines of 30 characters
                    let content = msg.content.clone();
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

            f.render_stateful_widget(messages, area, &mut chat.messages.state);
        }
    };
}

fn render_chat_input(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match app.loading {
        true => "Input (Loading...)".to_string(),
        false => format!("Input ({}/250)", app.input.text.len()),
    };

    f.render_widget(
        Paragraph::new(app.input.text.to_string())
            .style(match app.focus {
                Some(Section::Input) => Style::default().fg(Color::Green),
                _ => Style::default(),
            })
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(get_section_border_style(app, Section::Input))
                    .title(title),
            ),
        area,
    );

    if matches!(app.focus, Some(Section::Input)) {
        f.set_cursor(area.x + app.input.cursor_position() as u16 + 1, area.y + 1);
    }
}

fn render_help(f: &mut Frame, app: &mut App, area: Rect) {
    let mut text = match app.section {
        Section::Chats => "Q to quit, ENTER to select",
        Section::Messages => "Q to quit, ENTER to select",
        Section::Input => "Q to quit, ENTER to select",
        _ => "Q to quit",
    };

    if let Some(focus) = &app.focus {
        text = match focus {
            Section::Chats => "Esc to unfocus, Enter to select, H/J to move, N new, R rename",
            Section::Messages => "Esc to unfocus, Backspace to remove, H/J to move",
            Section::Input | Section::Modal => "Esc to unfocus",
        }
    }

    let help = Paragraph::new(text).style(Style::default());

    f.render_widget(help, area);
}

fn render_modal(f: &mut Frame, app: &mut App) {
    let area = f.size();

    let width = area.width / 2;
    let height = 3;

    let popup_area = Rect {
        x: area.width / 2 - width / 2,
        y: area.height / 2 - 10,
        width,
        height,
    };

    let title = match app.modal {
        Some(Modal::NewChat) => "New Chat",
        Some(Modal::RenameChat) => "Rename Chat",
        _ => "",
    };

    let popup = Popup::default()
        .content(app.modal_input.text.as_str())
        .title(title)
        .style(Style::new().yellow())
        .title_style(Style::new().white().bold())
        .border_style(Style::new().green().bold());

    f.render_widget(popup, popup_area);

    f.set_cursor(
        popup_area.x + app.modal_input.cursor_position() as u16 + 1,
        popup_area.y + 1,
    );
}

pub fn render(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(f.size());

    let messages_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // help
            Constraint::Percentage(5),
            // messages
            Constraint::Percentage(85),
            // input
            Constraint::Percentage(10),
        ])
        .split(main_layout[1]);

    render_chats(f, app, main_layout[0]);

    render_help(f, app, messages_chunks[0]);
    render_messages(f, app, messages_chunks[1]);
    render_chat_input(f, app, messages_chunks[2]);

    if app.modal.is_some() {
        render_modal(f, app);
    }
}
