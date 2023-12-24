use crate::state::{App, Section};
use ratatui::prelude::*;

enum SectionStatus {
    Hovered,
    Focused,
    Normal,
}

fn get_border_style(status: SectionStatus) -> Style {
    match status {
        SectionStatus::Hovered => Style::default().fg(Color::Yellow),
        SectionStatus::Focused => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        SectionStatus::Normal => Style::default(),
    }
}

pub fn get_section_border_style(app: &mut App, section: Section) -> Style {
    match &app.focus {
        Some(selected) if selected == &section => get_border_style(SectionStatus::Focused),
        _ => match &app.section {
            s if s == &section => get_border_style(SectionStatus::Hovered),
            _ => get_border_style(SectionStatus::Normal),
        },
    }
}

// String Utils

pub fn trim_spaces(s: &str) -> String {
    let re = regex::Regex::new(r"^\s+|\s+$").unwrap();

    re.replace_all(s, "").to_string()
}

pub fn into_lines(str: String, max_length: usize) -> Vec<Line<'static>> {
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
