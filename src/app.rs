use serde_json::{from_str, json};
use tui::widgets::ListState;

use crate::lms::*;

type ReqResult<T> = Result<T, reqwest::Error>;
type JsonValue = serde_json::Value;

pub struct App {
    client: LmsClient,
    pub player: Option<LmsPlayer>,
    pub playlist: Option<LmsPlaylist>,
    pub status: Option<LmsStatus>,
    pub playlist_state: ListState,
}

impl App {
    pub fn from(server_str: String) -> Self {
        let client = LmsClient::from(server_str);

        Self {
            client,
            player: None,
            playlist: None,
            status: None,
            playlist_state: ListState::default(),
        }
    }

    pub async fn on_tick(&mut self) -> ReqResult<()>{
        self.get_player().await?;
        self.get_current_status().await?;
        self.get_current_playlist().await?;
        self.update_state();

        Ok(())
    }

    async fn query(&self, command: JsonValue) -> ReqResult<LmsResponse> {
        self.client.query(command).await?
            .json::<LmsResponse>().await
    }

    pub async fn get_player(&mut self) -> ReqResult<()> {
        let command = json!(["-", ["serverstatus", 0]]);
        let res = self.query(command).await?;

        let players = res.get_array("players_loop")
            .expect("Could not extract value");
        let player: Option<LmsPlayer>;
        if !players.is_empty() {
            player = Some(
                // For now, we just connect to the first player we find
                from_str(&players[0].to_string()).unwrap());
        } else {
            player = None;
        }

        self.player = player;
        Ok(())
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
}
