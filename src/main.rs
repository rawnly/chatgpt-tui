mod components;
mod cursor;
mod models;
mod openai;
mod state;
mod ui;
mod utils;

use std::io;

use crate::state::*;
use crate::ui::render;

use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{backend::Backend, prelude::*};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event};
use std::time::{Duration, Instant};

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

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> anyhow::Result<()> {
    app.chats.select_first();

    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| render(f, &mut app))?;

        let elapsed = last_tick.elapsed();
        let timeout = tick_rate.saturating_sub(elapsed);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match &app.focus {
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
