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
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let app = App::from("192.168.0.188:9000".to_string()).await?;

    if let Some(playerid) = app.get_current_playerid() {
        // Determine how many tracks there are in the current playlist
        let command =
            serde_json::json!([playerid, ["status", 0, 9999, "tags:adl"]]);

        let res = app.client.query(command)
            .await?
            .json::<LmsResponse>()
            .await?;

        let playlist_loop = res.get_array("playlist_loop")
            .expect("Could not extract value");
        let mut playlist: Vec<LmsSong> = Vec::new();
        for track in playlist_loop.iter() {
            playlist.push(serde_json::from_str(&track.to_string()).unwrap());
        }
        let playlist = LmsPlaylist::from(playlist);

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
    let time_str = format!("{}:{}", minutes, seconds);

    time_str
}
