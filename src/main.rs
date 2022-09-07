use crossterm::{
    event::{
        self,
        Event,
        KeyCode
    },
};
use std::{
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
) -> Result<(), Box<dyn std::error::Error>> {
    app.on_tick().await?;
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = Instant::now();
        }
    }
}
