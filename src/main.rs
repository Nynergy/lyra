mod lms;

use lms::*;

struct App {
    client: LmsClient,
    player: Option<LmsPlayer>,
}

impl App {
    async fn from(server_str: String) -> Result<Self, reqwest::Error> {
        let client = LmsClient::from(server_str);
        let command = serde_json::json!(["-", ["serverstatus", 0]]);
        let res = client.query(command)
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

        Ok(Self { client, player })
    }

    fn get_current_playerid(&self) -> Option<String> {
        if let Some(player) = &self.player {
            Some(player.playerid.clone())
        } else {
            None
        }
    }

    async fn get_current_playlist(&self) -> Result<LmsPlaylist, reqwest::Error> {
        let playerid = self.get_current_playerid();
        let command =
            serde_json::json!([playerid, ["status", 0, 9999, "tags:adl"]]);

        let res = self.client.query(command)
            .await?
            .json::<LmsResponse>()
            .await?;

        let playlist_loop = res.get_array("playlist_loop")
            .expect("Could not extract value");
        let mut playlist: Vec<LmsSong> = Vec::new();
        for track in playlist_loop.iter() {
            playlist.push(serde_json::from_str(&track.to_string()).unwrap());
        }

        Ok(LmsPlaylist::from(playlist))
    }

    async fn get_current_status(&self) -> Result<LmsStatus, reqwest::Error> {
        let playerid = self.get_current_playerid();
        let command =
            serde_json::json!([playerid, ["status", 0, 9999]]);

        let res = self.client.query(command)
            .await?
            .json::<LmsResponse>()
            .await?;

        let player_name = res.get_str("player_name")
            .expect("Could not extract value");
        let playlist_index = res.get_str("playlist_cur_index")
            .expect("Could not extract value")
            .parse::<u64>()
            .expect("Playlist index is not a u64");
        let playlist_repeat = res.get_u64("playlist repeat")
            .expect("Could not extract value");
        let playlist_shuffle = res.get_u64("playlist shuffle")
            .expect("Could not extract value");
        let playlist_mode = res.get_str("mode")
            .expect("Could not extract value");
        let total_tracks = res.get_u64("playlist_tracks")
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

        let command =
            serde_json::json!([playerid, ["time", "?"]]);

        let res = self.client.query(command)
            .await?
            .json::<LmsResponse>()
            .await?;

        let elapsed_duration = res.get_f64("_time")
            .expect("Could not extract value");

        Ok(LmsStatus {
            player_name,
            playlist_index,
            playlist_repeat,
            playlist_shuffle,
            playlist_mode,
            total_tracks,
            elapsed_duration,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let app = App::from("192.168.0.188:9000".to_string()).await?;

    if let Some(_playerid) = app.get_current_playerid() {
        let status = app.get_current_status().await?;

        println!("{:?}", status);

        let playlist = app.get_current_playlist().await?;

        for track in playlist.tracks.iter() {
            println!(
                "{}: {} - {} - {} ({})",
                track.index + 1,
                track.title,
                track.artist,
                track.album,
                format_time(track.duration)
            );
        }
    } else {
        println!("Could not connect to a player.");
    }

    Ok(())
}

fn format_time(duration: f64) -> String {
    let minutes = duration as u64 / 60;
    let seconds = duration as u64 % 60;
    let time_str = format!("{}:{:02}", minutes, seconds);

    time_str
}
