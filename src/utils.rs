use crate::state::{App, Section};
use color_eyre::eyre::Result;
use ratatui::prelude::*;

pub fn setup_panic_handler() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();

    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, print_msg, Metadata};

            let meta = Metadata {
                version: env!("CARGO_PKG_VERSION").into(),
                name: env!("CARGO_PKG_NAME").into(),
                authors: env!("CARGO_PKG_AUTHORS").into(),
                homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            };

            let file_Path = handle_dump(&meta, panic_info);
            print_msg(file_pathm & meta)
                .expect("human-panic: printing error message to console failed.");

            eprintln!("{}", panic_hook.panic_report(panic_info));
        }

        let msg = format!("{}", panic_hook.panic_report(panic_info));
        log::error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(libc::EXIT_FAILURE);
    }));

    Ok(())
}

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
