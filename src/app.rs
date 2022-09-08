use serde_json::{from_str, json};
use tui::widgets::ListState;

use crate::config::*;
use crate::lms::*;

type ReqResult<T> = Result<T, reqwest::Error>;
type JsonValue = serde_json::Value;

pub struct PlayerList {
    pub players: Vec<LmsPlayer>,
    pub state: ListState,
}

impl PlayerList {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            players: Vec::new(),
            state
        }
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

pub enum AppState {
    PlayerMenu,
    Playlist
}

pub struct App {
    client: LmsClient,
    pub state: AppState,
    pub quit: bool,
    pub player: Option<LmsPlayer>,
    pub playlist: Option<LmsPlaylist>,
    pub status: Option<LmsStatus>,
    pub playlist_state: ListState,
    pub player_list: PlayerList,
    pub config: Config,
}

impl App {
    pub fn from(config: Config) -> Self {
        let client = LmsClient::from(
            format!("{}:{}", config.lms_ip, config.lms_port)
        );

        Self {
            client,
            state: AppState::PlayerMenu,
            quit: false,
            player: None,
            playlist: None,
            status: None,
            playlist_state: ListState::default(),
            player_list: PlayerList::default(),
            config
        }
    }

    pub async fn on_tick(&mut self) -> ReqResult<()> {
        match self.state {
            AppState::PlayerMenu => self.update_player_list().await?,
            AppState::Playlist => self.update_playlist_info().await?,
        }

        Ok(())
    }

    async fn update_player_list(&mut self) -> ReqResult<()> {
        let command = json!(["-", ["serverstatus", 0]]);
        let res = self.query(command).await?;

        let players = res.get_array("players_loop")
            .expect("Could not extract value");
        let player_list: Vec<LmsPlayer> = players
            .iter()
            .map(|p| {
                from_str(&p.to_string()).unwrap()
            })
            .collect();

        self.player_list.players = player_list.clone();
        if player_list.is_empty() {
            self.player_list.state.select(None);
        }

        Ok(())
    }

    async fn update_playlist_info(&mut self) -> ReqResult<()> {
        self.get_current_status().await?;
        self.get_current_playlist().await?;
        self.update_state();

        Ok(())
    }

    pub async fn select_player(&mut self) -> ReqResult<()> {
        let list = &self.player_list.players;
        if let Some(index) = self.player_list.state.selected() {
            self.player = Some(list[index].clone());
            self.update_playlist_info().await?;
        }

        Ok(())
    }

    pub fn change_state(&mut self, new_state: AppState) {
        self.state = new_state;
    }

    async fn query(&self, command: JsonValue) -> ReqResult<LmsResponse> {
        self.client.query(command).await?
            .json::<LmsResponse>().await
    }

    pub fn get_current_playerid(&self) -> Option<String> {
        if let Some(player) = &self.player {
            Some(player.playerid.clone())
        } else {
            None
        }
    }

    pub async fn get_current_playlist(&mut self) -> ReqResult<()> {
        if let Some(playerid) = self.get_current_playerid() {
            let command = json!([
                playerid,
                [ "status", 0, 9999, "tags:adl" ]
            ]);
            let res = self.query(command).await?;

            let empty = Vec::new();
            let playlist_loop = res.get_array("playlist_loop")
                .unwrap_or(&empty);
            let mut playlist: Vec<LmsSong> = Vec::new();
            for track in playlist_loop.iter() {
                playlist.push(
                    from_str(&track.to_string()).unwrap()
                );
            }

            self.playlist = Some(LmsPlaylist::from(playlist));
        } else {
            self.playlist = None;
        }

        Ok(())
    }

    pub async fn get_current_status(&mut self) -> ReqResult<()> {
        if let Some(playerid) = self.get_current_playerid() {
            let command = json!([
                playerid,
                [ "status", 0, 9999 ]
            ]);
            let res = self.query(command).await?;

            let player_name = res.get_str("player_name")
                .expect("Could not extract value");
            let total_tracks = res.get_u64("playlist_tracks")
                .expect("Could not extract value");

            let playlist_index: u64;
            if total_tracks == 0 {
                playlist_index = 0;
            } else {
                playlist_index = res.get_u64("playlist_cur_index")
                    .unwrap_or_else(|_| {
                        res.get_str("playlist_cur_index")
                            .expect("Could not extract value")
                            .parse::<u64>()
                            .expect("Playlist index is not a u64")
                    });
            }
            let playlist_repeat = RepeatMode::from(
                res.get_u64("playlist repeat")
                .expect("Could not extract value")
            );
            let playlist_shuffle = ShuffleMode::from(
                res.get_u64("playlist shuffle")
                .expect("Could not extract value")
            );
            let playlist_mode = PlaylistMode::from(
                res.get_str("mode")
                .expect("Could not extract value")
                .as_str()
            );

            let command = json!([
                playerid,
                [ "time", "?" ]
            ]);
            let res = self.query(command).await?;

            let elapsed_duration: f64;
            if total_tracks == 0 {
                elapsed_duration = 0.0;
            } else if playlist_mode == PlaylistMode::STOP {
                elapsed_duration = 0.0;
            } else {
                elapsed_duration = res.get_f64("_time")
                    .unwrap_or_else(|_| {
                        res.get_u64("_time")
                            .expect("Could not extract value") as f64
                    });
            }

            self.status = Some(LmsStatus {
                player_name,
                playlist_index,
                playlist_repeat,
                playlist_shuffle,
                playlist_mode,
                total_tracks,
                elapsed_duration,
            })
        } else {
            self.status = None;
        }

        Ok(())
    }

    fn update_state(&mut self) {
        if let Some(status) = &self.status {
            if status.total_tracks == 0 {
                self.playlist_state.select(None);
            } else {
                let index = status.playlist_index as usize;
                self.playlist_state.select(Some(index));
            }
        } else {
            self.playlist_state.select(None);
        }
    }

    pub fn list_down(&mut self) {
        if !self.player_list.is_empty() {
            let i = match self.player_list.state.selected() {
                Some(i) => {
                    if i >= self.player_list.players.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                },
                None => 0,
            };
            self.player_list.state.select(Some(i));
        }
    }

    pub fn list_up(&mut self) {
        if !self.player_list.is_empty() {
            let i = match self.player_list.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.player_list.players.len() - 1
                    } else {
                        i - 1
                    }
                },
                None => std::cmp::max(self.player_list.players.len() - 1, 0),
            };
            self.player_list.state.select(Some(i));
        }
    }

    pub fn jump_to_list_top(&mut self) {
        if let Some(_) = self.player_list.state.selected() {
            self.player_list.state.select(Some(0));
        }
    }

    pub fn jump_to_list_bottom(&mut self) {
        if let Some(_) = self.player_list.state.selected() {
            self.player_list.state.select(Some(self.player_list.players.len() - 1));
        }
    }
}
