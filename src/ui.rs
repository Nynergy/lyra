use tui::{
    backend::{Backend},
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
        Rect,
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
use unicode_truncate::UnicodeTruncateStr;

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

    render_status_header(f, chunks[0], app);
    render_playlist(f, chunks[1], app);
    render_playbar_footer(f, chunks[2], app);
}

fn render_status_header<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    app: &mut App
) {
    if let Some(status) = &app.status {
        render_status_info_left(f, chunk, status);
        render_status_info_center(f, chunk, status, app);
        render_status_info_right(f, chunk, status);
        render_status_bar(f, chunk);
    }
}

fn render_status_info_left<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    status: &LmsStatus
) {
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

    f.render_widget(left, chunk);
}

fn render_status_info_center<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    status: &LmsStatus,
    app: &App
) {
    let num_tracks = format!("{} Tracks", &status.total_tracks);
    let mut playlist_duration = 0.0;
    if let Some(playlist) = &app.playlist {
        playlist_duration = playlist
            .tracks
            .iter()
            .map(|t| { t.duration })
            .sum::<f64>();
    }
    let playlist_duration = format_time(playlist_duration, false);

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

    f.render_widget(center, chunk);
}

fn render_status_info_right<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    status: &LmsStatus
) {
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
            status.playlist_mode.to_string(),
            Style::default()
            .fg(mode_color)
            .add_modifier(Modifier::BOLD)
        ),
        Span::raw(" | ["),
        Span::styled(
            status.playlist_repeat.to_string(),
            Style::default()
            .fg(repeat_color)
            .add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            status.playlist_shuffle.to_string(),
            Style::default()
            .fg(shuffle_color)
            .add_modifier(Modifier::BOLD)
        ),
        Span::raw("]"),
    ]);

    let right = Paragraph::new(right)
        .block(Block::default())
        .alignment(Alignment::Right);

    f.render_widget(right, chunk);
}

fn render_status_bar<B: Backend>(f: &mut Frame<B>, chunk: Rect) {
    let bar = construct_bar(chunk.width);

    let bar = vec![
        // Empty line for status info
        Spans::from(vec![
            Span::raw(""),
        ]),
        Spans::from(vec![
            Span::raw(bar),
        ]),
    ];

    let bar = Paragraph::new(bar)
        .block(Block::default());

    f.render_widget(bar, chunk);
}

fn render_playlist<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    app: &mut App
) {
    if let Some(playlist) = &app.playlist {
        let items: Vec<ListItem> = playlist
            .tracks
            .iter()
            .map(|i| {
                ListItem::new(track_span(i, chunk.width))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default())
            .highlight_style(
                Style::default()
                .add_modifier(Modifier::REVERSED)
            );

        f.render_stateful_widget(list, chunk, &mut app.playlist_state);
    }
}

fn render_playbar_footer<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    app: &App
) {
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
                .split(chunk);

            let bar = construct_bar(chunks[2].width);
            let bar = Spans::from(vec![
                Span::raw(bar),
            ]);
            let bar = Paragraph::new(bar)
                .block(Block::default());

            f.render_widget(bar.clone(), chunks[0]);
            f.render_widget(bar, chunks[2]);

            let index = status.playlist_index as usize;
            let current_track: LmsSong;

            if status.total_tracks != 0 {
                current_track = playlist.tracks[index].clone();
            } else {
                current_track = LmsSong::default();
            }
            let elapsed = status.elapsed_duration;

            render_playbar_gauge(
                f,
                chunks[1],
                current_track.clone(),
                elapsed
            );

            render_now_playing_info(
                f,
                chunks[3],
                status,
                current_track,
                elapsed
            );
        }
    }
}

fn render_playbar_gauge<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    current_track: LmsSong,
    elapsed: f64
) {
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
        .split(chunk)[1];

    let playbar = Gauge::default()
        .block(Block::default())
        .gauge_style(
            Style::default()
            .fg(Color::Green)
        )
        .ratio(elapsed / current_track.duration)
        .label("");

    f.render_widget(playbar, playbar_chunk);
}

fn render_now_playing_info<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    status: &LmsStatus,
    current_track: LmsSong,
    elapsed: f64
) {
    render_now_playing_info_left(
        f,
        chunk,
        status,
        current_track.clone()
    );
    render_now_playing_info_right(
        f,
        chunk,
        current_track,
        elapsed
    );
}

fn render_now_playing_info_left<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    status: &LmsStatus,
    current_track: LmsSong
) {
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

    let max_length = chunk.width as usize - 20;
    let width_in_unicode = now_playing.chars()
        .map(|c| {
            if c.is_ascii() { 1 } else { 2 }
        })
        .sum::<usize>();
    if width_in_unicode > max_length {
        let (now_playing_str, _) = now_playing.unicode_truncate(max_length);
        now_playing = format!("{}...", now_playing_str);
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

    f.render_widget(left, chunk);
}

fn render_now_playing_info_right<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    current_track: LmsSong,
    elapsed: f64
) {
    let elapsed = format_time(elapsed, false);
    let duration = format_time(current_track.duration, false);
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

    f.render_widget(right, chunk);
}

fn format_time(duration: f64, full_width: bool) -> String {
    let seconds = duration as u64 % 60;
    let minutes = duration as u64 / 60;
    let time_str: String;

    if minutes >= 60 {
        let hours = minutes / 60;
        let minutes = minutes % 60;
        time_str = match full_width {
            true => format!("{:2}:{:02}:{:02}", hours, minutes, seconds),
            false => format!("{}:{:02}:{:02}", hours, minutes, seconds),
        };
    } else {
        time_str = match full_width {
            true => format!("{:2}:{:02}", minutes, seconds),
            false => format!("{}:{:02}", minutes, seconds),
        };
    }

    time_str
}

fn construct_bar(length: u16) -> String {
        let mut bar = String::from(line::VERTICAL_RIGHT);
        for _ in 0..length - 2 {
            bar.push_str(line::HORIZONTAL);
        }
        bar.push_str(line::VERTICAL_LEFT);

        bar
}

fn track_span<'a>(track: &'a LmsSong, width: u16) -> Spans<'a> {
    let index = format!("{:2}", track.index + 1);
    let index_spaces = " ";
    let mut current_width = index.len() + index_spaces.len();

    let artist_width_limit = 50;
    let album_width_limit = 80;
    let mut width_factor = width as usize / 4;
    if width > album_width_limit {
        width_factor = width as usize / 11 * 3;
    } else if width > artist_width_limit {
        width_factor = width as usize / 3;
    }

    let mut artist = String::new();
    let mut artist_spaces = String::new();
    if width > artist_width_limit {
        (artist, artist_spaces, current_width) =
            construct_text_column(
                track.artist.clone(),
                width_factor,
                current_width
            );
    }

    let mut album = String::new();
    let mut album_spaces = String::new();
    if width > album_width_limit {
        (album, album_spaces, current_width) =
            construct_text_column(
                track.album.clone(),
                width_factor,
                current_width
            );
    }

    let duration = format_time(track.duration, true);
    current_width += duration.len();

    let (title, title_spaces);
    (title, title_spaces, _) =
        construct_text_column(
            track.title.clone(),
            (width as usize - current_width) + 1,
            current_width
        );

    Spans::from(vec![
        Span::styled(
            format!("{}{}", index, index_spaces),
            Style::default().fg(Color::Magenta)
        ),
        Span::styled(
            format!("{}{}", title, title_spaces),
            Style::default().fg(Color::Yellow)
        ),
        Span::styled(
            format!("{}{}", artist, artist_spaces),
            Style::default().fg(Color::Blue)
        ),
        Span::styled(
            format!("{}{}", album, album_spaces),
            Style::default().fg(Color::Red)
        ),
        Span::styled(
            duration,
            Style::default().fg(Color::Cyan)
        ),
    ])
}

fn construct_text_column(
    base_text: String,
    width_factor: usize,
    mut current_width: usize,
) -> (String, String, usize) {
    let mut text;
    let mut spaces = String::new();

    let text_width = width_factor - 1;
    let width_in_unicode = base_text.chars()
        .map(|c| {
            if c.is_ascii() { 1 } else { 2 }
        })
        .sum::<usize>();
    let num_spaces = std::cmp::max(
        text_width.checked_sub(width_in_unicode)
            .unwrap_or(1),
        1
    );
    for _ in 0..num_spaces {
        spaces.push_str(" ");
    }
    text = base_text.clone();
    let (text_str, new_width) = text
        .unicode_truncate(text_width - 1);
    text = text_str.to_string();
    current_width += new_width + num_spaces;

    (text, spaces, current_width)
}
