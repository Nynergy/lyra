use crossterm::{
    event::{
        self,
        Event,
        KeyCode,
        KeyEvent
    },
};
use std::error::Error;

use crate::app::*;

type DynResult<T> = Result<T, Box<dyn Error>>;

pub async fn handle_events(app: &mut App) -> DynResult<()> {
    if let Event::Key(key) = event::read()? {
        match app.state {
            AppState::PlayerMenu => handle_player_menu_events(key, app).await?,
            AppState::Playlist => handle_playlist_events(key, app),
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
