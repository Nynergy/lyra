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
            Span::styled(
                "Player: ",
                Style::default().add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                &status.player_name,
                Style::default().fg(Color::Red)
            ),
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
            Span::styled(
                num_tracks,
                Style::default().add_modifier(Modifier::BOLD)
            ),
            Span::raw(" | "),
            Span::styled(
                playlist_duration,
                Style::default().add_modifier(Modifier::BOLD)
            ),
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

        let mode_color = match status.playlist_mode {
            PlaylistMode::STOP => Color::Red,
            PlaylistMode::PLAY => Color::Green,
            PlaylistMode::PAUSE => Color::Yellow,
        };
        let repeat_color = match status.playlist_repeat {
            RepeatMode::NONE => Color::White,
            _ => Color::Magenta,
        };
        let shuffle_color = match status.playlist_shuffle {
            ShuffleMode::NONE => Color::White,
            _ => Color::Cyan,
        };

        let right = Spans::from(vec![
            Span::styled(
                playback,
                Style::default()
                .fg(mode_color)
                .add_modifier(Modifier::BOLD)
            ),
            Span::raw(" | ["),
            Span::styled(
                repeat,
                Style::default()
                .fg(repeat_color)
                .add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                shuffle,
                Style::default()
                .fg(shuffle_color)
                .add_modifier(Modifier::BOLD)
            ),
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
            let current_track: LmsSong;

            if status.total_tracks != 0 {
                current_track = playlist.tracks[index].clone();
            } else {
                current_track = LmsSong::default();
            }
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

            let mut now_playing: String;

            if status.total_tracks != 0 {
                now_playing = format!(
                    "Now Playing: {} - {}",
                    current_track.title,
                    current_track.artist
                );
            } else {
                now_playing = "Now Playing: N/A".to_string();
            }

            let max_length = chunks[3].width as usize - 25;
            if now_playing.len() > max_length {
                now_playing.truncate(max_length);
                now_playing = format!("{}...", now_playing);
            }
            let now_playing: Vec<&str> = now_playing.split(":").collect();
            let left = Spans::from(vec![
                Span::styled(
                    format!("{}:", now_playing[0]),
                    Style::default().add_modifier(Modifier::BOLD)
                ),
                Span::raw(now_playing[1]),
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
    let index = format!("{:2}", track.index + 1);
    let index_spaces = " ";
    let mut current_width = index.len() + index_spaces.len();

    let mut artist = String::new();
    let mut artist_spaces = String::new();
    if width > 50 {
        artist_spaces = String::new();
        let artist_width = width as usize / 4;
        let width_in_unicode = track.artist.chars()
            .map(|c| {
                if c.is_ascii() { 1 } else { 2 }
            })
            .sum::<usize>();
        let spaces = std::cmp::max(artist_width - width_in_unicode, 1);
        for _ in 0..spaces {
            artist_spaces.push_str(" ");
        }
        artist = track.artist.clone();
        artist.truncate(artist_width);
        current_width += artist.chars().count() + artist_spaces.len();
    }

    let mut album = String::new();
    let mut album_spaces = String::new();
    if width > 80 {
        let album_width = width as usize / 4;
        let width_in_unicode = track.album.chars()
            .map(|c| {
                if c.is_ascii() { 1 } else { 2 }
            })
            .sum::<usize>();
        let spaces = std::cmp::max(album_width - width_in_unicode, 1);
        for _ in 0..spaces {
            album_spaces.push_str(" ");
        }
        album = track.album.clone();
        album.truncate(album_width);
        current_width += album.chars().count() + album_spaces.len();
    }

    let duration = format_time(track.duration);
    current_width += duration.len();

    let mut title_spaces = String::new();
    let title_width = width as usize - current_width;
    let width_in_unicode = track.title.chars()
        .map(|c| {
            if c.is_ascii() { 1 } else { 2 }
        })
        .sum::<usize>();
    let spaces = title_width.checked_sub(width_in_unicode)
        .unwrap_or_else(|| { 1 });
    for _ in 0..spaces {
        title_spaces.push_str(" ");
    }
    let mut title = track.title.clone();
    title.truncate(title_width);
    if spaces == 1 { title.truncate(title_width - 1); }


    Spans::from(vec![
        Span::styled(
            index,
            Style::default().fg(Color::Magenta)
        ),
        Span::styled(
            index_spaces,
            Style::default().fg(Color::Magenta)
        ),
        Span::styled(
            title,
            Style::default().fg(Color::Yellow)
        ),
        Span::styled(
            title_spaces,
            Style::default().fg(Color::Yellow)
        ),
        Span::styled(
            artist,
            Style::default().fg(Color::Blue)
        ),
        Span::styled(
            artist_spaces,
            Style::default().fg(Color::Blue)
        ),
        Span::styled(
            album,
            Style::default().fg(Color::Red)
        ),
        Span::styled(
            album_spaces,
            Style::default().fg(Color::Red)
        ),
        Span::styled(
            duration,
            Style::default().fg(Color::Cyan)
        ),
    ])
}
