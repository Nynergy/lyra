use tui::{
    backend::{Backend},
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout
    },
    symbols::line,
    text::{Span, Spans},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::app::*;
use crate::lms::*;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(4),
            ]
            .as_ref()
        )
        .split(f.size());

    if let Some(status) = &app.status {
        let left = Spans::from(vec![
            Span::raw("Connected: "),
            Span::raw(&status.player_name),
        ]);

        let left = Paragraph::new(left)
            .block(Block::default());

        f.render_widget(left, chunks[0]);

        let num_tracks = format!("{} Tracks", &status.total_tracks);
        let mut playlist_duration = 0.0;
        if let Some(playlist) = &app.playlist {
            for track in playlist.tracks.iter() {
                playlist_duration += track.duration;
            }
        }
        let playlist_duration = format_time(playlist_duration);

        let center = Spans::from(vec![
            Span::raw(num_tracks),
            Span::raw(" | "),
            Span::raw(playlist_duration),
        ]);

        let center = Paragraph::new(center)
            .block(Block::default())
            .alignment(Alignment::Center);

        f.render_widget(center, chunks[0]);

        let playback = match status.playlist_mode {
            PlaylistMode::STOP => "STOPPED",
            PlaylistMode::PLAY => "PLAYING",
            PlaylistMode::PAUSE => "PAUSED",
        };
        let repeat = match status.playlist_repeat {
            RepeatMode::NONE => "-",
            RepeatMode::TRACK => "r",
            RepeatMode::PLAYLIST => "R",
        };
        let shuffle = match status.playlist_shuffle {
            ShuffleMode::NONE => "-",
            ShuffleMode::TRACK => "z",
            ShuffleMode::ALBUM => "Z",
        };

        let right = Spans::from(vec![
            Span::raw(playback),
            Span::raw(" | ["),
            Span::raw(repeat),
            Span::raw(shuffle),
            Span::raw("]"),
        ]);

        let right = Paragraph::new(right)
            .block(Block::default())
            .alignment(Alignment::Right);

        f.render_widget(right, chunks[0]);

        let mut bar = String::from(line::VERTICAL_RIGHT);
        for _ in 0..chunks[0].width - 2 {
            bar.push_str(line::HORIZONTAL);
        }
        bar.push_str(line::VERTICAL_LEFT);

        let bar = vec![
            Spans::from(vec![
                Span::raw(""),
            ]),
            Spans::from(vec![
                Span::raw(bar),
            ]),
        ];

        let bar = Paragraph::new(bar)
            .block(Block::default());

        f.render_widget(bar, chunks[0]);
    }
}

fn format_time(duration: f64) -> String {
    let seconds = duration as u64 % 60;
    let minutes = duration as u64 / 60;
    let time_str: String;

    if minutes >= 60 {
        let hours = minutes / 60;
        let minutes = minutes % 60;
        time_str = format!("{}:{:02}:{:02}", hours, minutes, seconds);
    } else {
        time_str = format!("{}:{:02}", minutes, seconds);
    }

    time_str
}
