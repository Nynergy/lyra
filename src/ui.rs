use tui::{
    backend::{Backend},
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout
    },
    style::{Color, Modifier, Style},
    symbols::line,
    text::{Span, Spans},
    widgets::{
        Block,
        Gauge,
        List,
        ListItem,
        Paragraph
    },
    Frame,
};

use crate::app::*;
use crate::lms::*;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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

    // Status
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

    // Playlist
    if let Some(playlist) = &app.playlist {
        let items: Vec<ListItem> = playlist
            .tracks
            .iter()
            .map(|i| {
                ListItem::new(track_span(i, chunks[1].width))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default())
            .highlight_style(
                Style::default()
                .add_modifier(Modifier::REVERSED)
            );

        f.render_stateful_widget(list, chunks[1], &mut app.playlist_state);
    }

    // Playbar
    if let Some(playlist) = &app.playlist {
        if let Some(status) = &app.status {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref()
                )
                .split(chunks[2]);

            let index = status.playlist_index as usize;
            let current_track = playlist.tracks[index].clone();
            let elapsed = status.elapsed_duration;

            let mut bar = String::from(line::VERTICAL_RIGHT);
            for _ in 0..chunks[2].width - 2 {
                bar.push_str(line::HORIZONTAL);
            }
            bar.push_str(line::VERTICAL_LEFT);

            let bar = Spans::from(vec![
                Span::raw(bar),
            ]);

            let bar = Paragraph::new(bar)
                .block(Block::default());

            f.render_widget(bar.clone(), chunks[0]);
            f.render_widget(bar, chunks[2]);

            let playbar_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ]
                    .as_ref()
                )
                .split(chunks[1])[1];

            let playbar = Gauge::default()
                .block(Block::default())
                .gauge_style(
                    Style::default()
                    .fg(Color::Green)
                )
                .ratio(elapsed / current_track.duration)
                .label("");

            f.render_widget(playbar, playbar_chunk);

            let mut now_playing = format!(
                "Now Playing: {} - {}",
                current_track.title,
                current_track.artist
            );
            let max_length = chunks[3].width as usize / 3 * 2;
            if now_playing.len() > max_length {
                now_playing.truncate(max_length);
                now_playing = format!("{}...", now_playing);
            }
            let left = Spans::from(vec![
                Span::raw(now_playing),
            ]);

            let left = Paragraph::new(left)
                .block(Block::default());

            f.render_widget(left, chunks[3]);

            let elapsed = format_time(elapsed);
            let duration = format_time(current_track.duration);
            let right = Spans::from(vec![
                Span::raw("("),
                Span::raw(elapsed),
                Span::raw("/"),
                Span::raw(duration),
                Span::raw(")"),
            ]);

            let right = Paragraph::new(right)
                .block(Block::default())
                .alignment(Alignment::Right);

            f.render_widget(right, chunks[3]);
        }
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

fn track_span<'a>(track: &'a LmsSong, width: u16) -> Spans<'a> {
    let index = format!("{:2}", track.index);
    let index_spaces = " ";
    let mut current_width = index.len() + index_spaces.len();

    let mut artist_spaces = String::new();
    let artist_width = width as usize / 11 * 3;
    let spaces = std::cmp::max(artist_width - track.artist.len(), 1);
    for _ in 0..spaces {
        artist_spaces.push_str(" ");
    }
    let mut artist = track.artist.clone();
    artist.truncate(artist_width);
    current_width += artist.len() + artist_spaces.len();

    let mut album_spaces = String::new();
    let album_width = width as usize / 11 * 3;
    let spaces = std::cmp::max(album_width - track.album.len(), 1);
    for _ in 0..spaces {
        album_spaces.push_str(" ");
    }
    let mut album = track.album.clone();
    album.truncate(album_width);
    current_width += album.len() + album_spaces.len();

    let duration = format_time(track.duration);
    current_width += duration.len();

    let mut title_spaces = String::new();
    let title_width = width as usize - current_width;
    let spaces = title_width.checked_sub(track.title.len())
        .unwrap_or_else(|| { 1 });
    for _ in 0..spaces {
        title_spaces.push_str(" ");
    }
    let mut title = track.title.clone();
    title.truncate(title_width);
    if spaces == 1 { title.truncate(title_width - 1); }


    Spans::from(vec![
        Span::raw(index),
        Span::raw(index_spaces),
        Span::raw(title),
        Span::raw(title_spaces),
        Span::raw(artist),
        Span::raw(artist_spaces),
        Span::raw(album),
        Span::raw(album_spaces),
        Span::raw(duration),
    ])
}
