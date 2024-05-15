use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
        Margin,
        Rect,
    },
    style::{Color, Modifier, Style},
    symbols::line,
    text::{Line, Span},
    widgets::{
        Block,
        Clear,
        Gauge,
        List,
        ListItem,
        Paragraph,
        Widget,
        Wrap
    },
    Frame,
};
use unicode_truncate::UnicodeTruncateStr;

use crate::app::*;
use crate::lms::*;

macro_rules! raw_para {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_para = Vec::new();
            $(
                temp_para.push(
                    Line::from(
                        Span::raw($x)
                    )
                );
            )*
            temp_para
        }
    };
}

struct CustomBorder {
    title: String,
    title_style: Style,
    border_style: Style,
}

impl CustomBorder {
    fn new() -> Self {
        Self {
            title: "".to_string(),
            title_style: Style::default(),
            border_style: Style::default(),
        }
    }

    fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}

impl Widget for CustomBorder {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Border Lines
        let mut line = String::new();
        line.push_str(line::VERTICAL_RIGHT);
        for _ in 0..area.width - 2 {
            line.push_str(line::HORIZONTAL);
        }
        line.push_str(line::VERTICAL_LEFT);
        buf.set_string(area.left(), area.top(), line.clone(), self.border_style);
        buf.set_string(area.left(), area.bottom() - 1, line, self.border_style);

        // Title
        let offset = area.width / 2 - self.title.len() as u16 / 2;
        let title_x = area.left() + offset;
        let title_y = area.y;
        buf.set_string(title_x, title_y, self.title.clone(), self.title_style);

        // Title Tee's
        buf.set_string(
            title_x - 1,
            area.top(),
            line::VERTICAL_LEFT,
            self.border_style
        );
        buf.set_string(
            title_x + self.title.len() as u16,
            area.top(),
            line::VERTICAL_RIGHT,
            self.border_style
        );
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    if (f.size().height < 9) || (f.size().width < 20) {
        f.render_widget(Clear, f.size());
        return;
    }

    match app.state {
        AppState::PlayerMenu => render_player_menu_state(f, app),
        AppState::Playlist => render_playlist_state(f, app),
    }
}

fn render_player_menu_state(
    f: &mut Frame,
    app: &mut App
) {
    if f.size().height > 15 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                Constraint::Length(10),
                Constraint::Min(3),
                Constraint::Length(3),
                ]
                .as_ref()
            )
            .split(f.size());

        render_banner(f, chunks[0], app);

        let list_area = centered_rect(40, 100, chunks[1]);

        if app.player_list.is_empty() {
            render_empty_list_info(f, list_area);
        } else {
            render_player_list(f, list_area, app);
        }

        render_player_menu_footer(f, chunks[2]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                Constraint::Length(5),
                Constraint::Min(3),
                Constraint::Length(1),
                ]
                .as_ref()
            )
            .split(f.size());

        render_tiny_banner(f, chunks[0], app);

        let list_area = centered_rect(40, 100, chunks[1]);

        if app.player_list.is_empty() {
            render_empty_list_info(f, list_area);
        } else {
            render_player_list(f, list_area, app);
        }

        render_empty_line(f, chunks[2]);
    }
}

fn render_banner(
    f: &mut Frame,
    chunk: Rect,
    app: &App
) {
    let banner = raw_para!(
        "",
        "    __                ",
        "   / /_  ___________ _",
        "  / / / / / ___/ __ `/",
        " / / /_/ / /  / /_/ / ",
        "/_/\\__, /_/   \\__,_/  ",
        "  /____/              ",
        "",
        "An LMS Playlist Viewer for the Terminal",
        ""
    );

    let banner = Paragraph::new(banner)
        .block(Block::default())
        .style(
            Style::default()
            .fg(Color::Indexed(*app.config.color("Banner")))
            .add_modifier(Modifier::BOLD)
        )
        .alignment(Alignment::Center);

    f.render_widget(banner, chunk);
}

fn render_tiny_banner(
    f: &mut Frame,
    chunk: Rect,
    app: &App
) {
    let banner = raw_para!(
        "",
        "lyra",
        "",
        "An LMS Playlist Viewer for the Terminal",
        ""
    );

    let banner = Paragraph::new(banner)
        .block(Block::default())
        .style(
            Style::default()
            .fg(Color::Indexed(*app.config.color("Banner")))
            .add_modifier(Modifier::BOLD)
        )
        .alignment(Alignment::Center);

    f.render_widget(banner, chunk);
}

fn render_empty_list_info(f: &mut Frame, chunk: Rect) {
    let mut info = raw_para!(
        "There are currently no connected players."
    );

    for _ in 0..chunk.height / 2 - 2 {
        info.insert(0, Line::from(Span::raw("")));
    }

    let info = Paragraph::new(info)
        .block(Block::default())
        .style(
            Style::default()
            .add_modifier(Modifier::BOLD)
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(info, chunk);
}

fn render_player_list(
    f: &mut Frame,
    chunk: Rect,
    app: &mut App
) {
    let highlight = Style::default()
        .add_modifier(Modifier::REVERSED);

    let container = CustomBorder::new()
        .title("Players".to_string());

    f.render_widget(container, chunk);

    let list_area = shrink_rect(chunk, 1);

    let items: Vec<ListItem> = app.player_list
        .players
        .iter()
        .map(|p| {
            ListItem::new(
                Span::raw(p.name.clone())
            )
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(highlight);

    f.render_stateful_widget(
        list,
        list_area,
        &mut app.player_list.state
    );
}

fn render_player_menu_footer(
    f: &mut Frame,
    chunk: Rect
) {
    let info = raw_para!(
        "",
        "lyra v1.0.0 by Ben Buchanan (https://github.com/Nynergy)"
    );

    let info = Paragraph::new(info)
        .block(Block::default())
        .alignment(Alignment::Center);

    f.render_widget(info, chunk);
}

fn render_playlist_state(f: &mut Frame, app: &mut App) {
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

fn render_status_header(
    f: &mut Frame,
    chunk: Rect,
    app: &mut App
) {
    if let Some(status) = &app.status {
        render_status_info_left(f, chunk, status, app);
        if f.size().width > 50 {
            render_status_info_center(f, chunk, status, app);
        }
        render_status_info_right(f, chunk, status, app);
        render_status_bar(f, chunk);
    }
}

fn render_status_info_left(
    f: &mut Frame,
    chunk: Rect,
    status: &LmsStatus,
    app: &App
) {
    let left = Line::from(vec![
        Span::styled(
            "Player: ",
            Style::default().add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            &status.player_name,
            Style::default()
            .fg(Color::Indexed(*app.config.color("PlayerName")))
        ),
    ]);

    let left = Paragraph::new(left)
        .block(Block::default());

    f.render_widget(left, chunk);
}

fn render_status_info_center(
    f: &mut Frame,
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

    let center = Line::from(vec![
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

fn render_status_info_right(
    f: &mut Frame,
    chunk: Rect,
    status: &LmsStatus,
    app: &App
) {
    let mode_color = match status.playlist_mode {
        PlaylistMode::STOP =>
            Color::Indexed(*app.config.color("StoppedIndicator")),
        PlaylistMode::PLAY =>
            Color::Indexed(*app.config.color("PlayingIndicator")),
        PlaylistMode::PAUSE =>
            Color::Indexed(*app.config.color("PausedIndicator")),
    };
    let repeat_color = match status.playlist_repeat {
        RepeatMode::NONE => Color::White,
        _ => Color::Indexed(*app.config.color("RepeatIndicator")),
    };
    let shuffle_color = match status.playlist_shuffle {
        ShuffleMode::NONE => Color::White,
        _ => Color::Indexed(*app.config.color("ShuffleIndicator")),
    };

    let right = Line::from(vec![
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

fn render_status_bar(f: &mut Frame, chunk: Rect) {
    let bar = construct_bar(chunk.width);

    let bar = vec![
        // Empty line for status info
        Line::from(vec![
            Span::raw(""),
        ]),
        Line::from(vec![
            Span::raw(bar),
        ]),
    ];

    let bar = Paragraph::new(bar)
        .block(Block::default());

    f.render_widget(bar, chunk);
}

fn render_playlist(
    f: &mut Frame,
    chunk: Rect,
    app: &mut App
) {
    if let Some(playlist) = &app.playlist {
        let items: Vec<ListItem> = playlist
            .tracks
            .iter()
            .map(|i| {
                ListItem::new(track_span(i, chunk.width, app))
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

fn render_playbar_footer(
    f: &mut Frame,
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
            let bar = Line::from(vec![
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
                elapsed,
                app
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

fn render_playbar_gauge(
    f: &mut Frame,
    chunk: Rect,
    current_track: LmsSong,
    elapsed: f64,
    app: &App
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
            .fg(Color::Indexed(*app.config.color("PlaybarGauge")))
        )
        .ratio(elapsed / current_track.duration)
        .label("");

    f.render_widget(playbar, playbar_chunk);
}

fn render_now_playing_info(
    f: &mut Frame,
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

fn render_now_playing_info_left(
    f: &mut Frame,
    chunk: Rect,
    status: &LmsStatus,
    current_track: LmsSong
) {
    if chunk.width > 33 {
        let mut now_playing: String;

        if status.total_tracks != 0 {
            now_playing = format!(
                "{} - {}",
                current_track.title,
                current_track.artist
            );
        } else {
            now_playing = "N/A".to_string();
        }

        let max_length = chunk.width as usize - 33;
        let width_in_unicode = now_playing.chars()
            .map(|c| {
                if c.is_ascii() { 1 } else { 2 }
            })
            .sum::<usize>();
        if width_in_unicode > max_length {
            let (now_playing_str, _) = now_playing.unicode_truncate(max_length);
            now_playing = format!("{}...", now_playing_str);
        }
        let left = Line::from(vec![
            Span::styled(
                format!("Now Playing: "),
                Style::default().add_modifier(Modifier::BOLD)
            ),
            Span::raw(now_playing),
        ]);

        let left = Paragraph::new(left)
            .block(Block::default());

        f.render_widget(left, chunk);
    }
}

fn render_now_playing_info_right(
    f: &mut Frame,
    chunk: Rect,
    current_track: LmsSong,
    elapsed: f64
) {
    let elapsed = format_time(elapsed, false);
    let duration = format_time(current_track.duration, false);
    let right = Line::from(vec![
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

fn render_empty_line(f: &mut Frame, chunk: Rect) {
    let line = raw_para!("");
    let line = Paragraph::new(line)
        .block(Block::default());

    f.render_widget(line, chunk);
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

fn track_span<'a>(
    track: &'a LmsSong,
    width: u16,
    app: &App
) -> Line<'a> {
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

    Line::from(vec![
        Span::styled(
            format!("{}{}", index, index_spaces),
            Style::default()
            .fg(Color::Indexed(*app.config.color("TrackIndex")))
        ),
        Span::styled(
            format!("{}{}", title, title_spaces),
            Style::default()
            .fg(Color::Indexed(*app.config.color("TrackTitle")))
        ),
        Span::styled(
            format!("{}{}", artist, artist_spaces),
            Style::default()
            .fg(Color::Indexed(*app.config.color("TrackArtist")))
        ),
        Span::styled(
            format!("{}{}", album, album_spaces),
            Style::default()
            .fg(Color::Indexed(*app.config.color("TrackAlbum")))
        ),
        Span::styled(
            duration,
            Style::default()
            .fg(Color::Indexed(*app.config.color("TrackDuration")))
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

fn centered_rect(percent_x: usize, percent_y: usize, size: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) as u16 / 2),
                Constraint::Percentage(percent_y as u16),
                Constraint::Percentage((100 - percent_y) as u16 / 2),
            ]
            .as_ref(),
        )
        .split(size);

    let popup_rect = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) as u16 / 2),
                Constraint::Percentage(percent_x as u16),
                Constraint::Percentage((100 - percent_x) as u16 / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1];

    popup_rect
}

fn shrink_rect(rect: Rect, amount: u16) -> Rect {
    let margin = Margin { vertical: amount, horizontal: amount };
    rect.inner(&margin)
}
