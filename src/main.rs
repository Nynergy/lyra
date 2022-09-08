use crossterm::{
    event::{
        self,
        Event,
        KeyCode,
        KeyEvent
    },
};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend},
    Terminal,
};

mod app;
mod lms;
mod tui_handling;
mod ui;

use app::*;
use tui_handling::*;
use ui::*;

type DynResult<T> = Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> DynResult<()> {
    // Panic Handling
    chain_hook();

    let mut terminal = init_terminal()?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(1000);
    let app = App::from("192.168.0.188:9000".to_string());
    let res = run_app(&mut terminal, app, tick_rate).await;

    terminal.show_cursor()?;
    reset_terminal()?;

    if let Err(err) = res {
        println!("{}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> DynResult<()> {
    app.on_tick().await?;
    let mut last_tick = Instant::now();

    loop {
        if app.quit {
            break;
        }

        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::PlayerMenu => handle_player_menu_events(key, &mut app).await?,
                    AppState::Playlist => handle_playlist_events(key, &mut app),
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}

async fn handle_player_menu_events(
    key: KeyEvent,
    app: &mut App
) -> DynResult<()> {
    match key.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Esc => app.quit = true,
        KeyCode::Char(' ') => {
            if !app.player_list.is_empty() {
                app.select_player().await?;
                app.change_state(AppState::Playlist);
            }
        },
        KeyCode::Enter => {
            if !app.player_list.is_empty() {
                app.select_player().await?;
                app.change_state(AppState::Playlist);
            }
        },
        KeyCode::Char('j') => app.list_down(),
        KeyCode::Down => app.list_down(),
        KeyCode::Char('k') => app.list_up(),
        KeyCode::Up => app.list_up(),
        KeyCode::Char('g') => app.jump_to_list_top(),
        KeyCode::Home => app.jump_to_list_top(),
        KeyCode::Char('G') => app.jump_to_list_bottom(),
        KeyCode::End => app.jump_to_list_bottom(),
        _ => {}
    }

    Ok(())
}

fn handle_playlist_events(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('p') => app.change_state(AppState::PlayerMenu),
        _ => {}
    }
}
