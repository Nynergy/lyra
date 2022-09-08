use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend},
    Terminal,
};

mod app;
mod config;
mod events;
mod lms;
mod tui_handling;
mod ui;

use app::*;
use config::*;
use events::*;
use tui_handling::*;
use ui::*;

type DynResult<T> = Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> DynResult<()> {
    // Panic Handling
    chain_hook();

    let mut terminal = init_terminal()?;
    terminal.clear()?;

    let config = load_config()?;
    let app = App::from(config);

    let tick_rate = Duration::from_millis(1000);
    let res = run_app(&mut terminal, app, tick_rate).await;

    terminal.show_cursor()?;
    reset_terminal()?;

    if let Err(err) = res {
        println!("{}", err);
    }

    Ok(())
}

fn load_config() -> DynResult<Config> {
    let user_home = env::var("HOME");
    let user_home = user_home.unwrap_or("/".to_string());
    let path = PathBuf::from(&user_home);
    let path = path.join(".lyra");
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    env::set_current_dir(&path)?;

    let config: Config;
    let config_path = path.join("config.json");
    if let Ok(config_data) = fs::read_to_string(&config_path) {
        config = serde_json::from_str(&config_data)
            .unwrap_or(Config::default());
    } else {
        config = Config::default();
    }

    Ok(config)
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
            handle_events(&mut app).await?;
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}
