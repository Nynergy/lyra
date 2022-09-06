use tui::widgets::ListState;

use crate::lms::*;

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

    pub async fn on_tick(&mut self) -> Result<(), reqwest::Error>{
        self.get_player().await?;
        self.get_current_status().await?;
        self.get_current_playlist().await?;
        self.update_state();

        Ok(())
    }

    pub async fn get_player(&mut self) -> Result<(), reqwest::Error> {
        let command = serde_json::json!(["-", ["serverstatus", 0]]);
        let res = self.client.query(command)
            .await?
            .json::<LmsResponse>()
            .await?;

        let players = res.get_array("players_loop")
            .expect("Could not extract value");
        let player: Option<LmsPlayer>;
        if !players.is_empty() {
            player = Some(
                // For now, we just connect to the first player we find
                serde_json::from_str(&players[0].to_string())
                .unwrap());
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

    pub async fn get_current_playlist(&mut self) -> Result<(), reqwest::Error> {
        if let Some(playerid) = self.get_current_playerid() {
            let command = serde_json::json!([
                playerid,
                [ "status", 0, 9999, "tags:adl" ]
            ]);

            let res = self.client.query(command)
                .await?
                .json::<LmsResponse>()
                .await?;

            let playlist_loop = res.get_array("playlist_loop")
                .expect("Could not extract value");
            let mut playlist: Vec<LmsSong> = Vec::new();
            for track in playlist_loop.iter() {
                playlist.push(
                    serde_json::from_str(&track.to_string()).unwrap()
                );
            }

            self.playlist = Some(LmsPlaylist::from(playlist));
        } else {
            self.playlist = None;
        }

        Ok(())
    }

    pub async fn get_current_status(&mut self) -> Result<(), reqwest::Error> {
        if let Some(playerid) = self.get_current_playerid() {
            let command = serde_json::json!([
                playerid,
                [ "status", 0, 9999 ]
            ]);

            let res = self.client.query(command)
                .await?
                .json::<LmsResponse>()
                .await?;

            let player_name = res.get_str("player_name")
                .expect("Could not extract value");
            let total_tracks = res.get_u64("playlist_tracks")
                .expect("Could not extract value");

            let playlist_index: u64;
            if total_tracks == 0 {
                playlist_index = 0;
            } else {
                playlist_index = res.get_str("playlist_cur_index")
                    .unwrap_or_else(|_| {
                        format!("{}", res.get_u64("playlist_cur_index")
                            .expect("Could not extract value"))
                    })
                    .parse::<u64>()
                    .expect("Playlist index is not a u64");
            }
            let playlist_repeat = res.get_u64("playlist repeat")
                .expect("Could not extract value");
            let playlist_shuffle = res.get_u64("playlist shuffle")
                .expect("Could not extract value");
            let playlist_mode = res.get_str("mode")
                .expect("Could not extract value");

            let playlist_repeat = match playlist_repeat {
                0 => RepeatMode::NONE,
                1 => RepeatMode::TRACK,
                2 => RepeatMode::PLAYLIST,
                _ => RepeatMode::NONE
            };

            let playlist_shuffle = match playlist_shuffle {
                0 => ShuffleMode::NONE,
                1 => ShuffleMode::TRACK,
                2 => ShuffleMode::ALBUM,
                _ => ShuffleMode::NONE
            };

            let playlist_mode = match playlist_mode.as_str() {
                "play" => PlaylistMode::PLAY,
                "stop" => PlaylistMode::STOP,
                "pause" => PlaylistMode::PAUSE,
                _ => unreachable!()
            };

            let command = serde_json::json!([
                playerid,
                [ "time", "?" ]
            ]);

            let res = self.client.query(command)
                .await?
                .json::<LmsResponse>()
                .await?;

            let elapsed_duration: f64;
            if total_tracks == 0 {
                elapsed_duration = 0.0;
            } else if playlist_mode == PlaylistMode::STOP {
                elapsed_duration = 0.0;
            } else {
                elapsed_duration = res.get_f64("_time")
                    .expect("Could not extract value");
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
