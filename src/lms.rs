use serde::Deserialize;
use std::{
    collections::HashMap,
    net,
};

pub struct LmsClient {
    client: reqwest::Client,
    socket: net::SocketAddr,
}

impl LmsClient {
    pub fn from(socket_str: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            socket: socket_str.parse()
                .expect("Unable to parse socket address"),
        }
    }

    pub async fn query(
        &self,
        command: serde_json::Value
    ) -> Result<reqwest::Response, reqwest::Error> {
        let res = self.client
            .post(format!(
                "http://{}/jsonrpc.js",
                self.socket.to_string()
            ))
            .json(&serde_json::json!({
                "method": "slim.request",
                "params": command
            }))
            .send()
            .await?;

        Ok(res)
    }
}

#[derive(Debug, Deserialize)]
pub struct LmsResponse {
    result: HashMap::<String, serde_json::Value>,
}

impl LmsResponse {
    pub fn get_u64(&self, key: &str) -> Result<u64, String> {
        if let Some(value) = self.result.get(key) {
            if value.is_u64() {
                Ok(value.as_u64().unwrap())
            } else {
                Err(format!("'{}' is not a u64!", key))
            }
        } else {
            Err(format!("'{}' does not exist!", key))
        }
    }

    pub fn get_f64(&self, key: &str) -> Result<f64, String> {
        if let Some(value) = self.result.get(key) {
            if value.is_f64() {
                Ok(value.as_f64().unwrap())
            } else {
                Err(format!("'{}' is not a f64!", key))
            }
        } else {
            Err(format!("'{}' does not exist!", key))
        }
    }

    pub fn get_str(&self, key: &str) -> Result<String, String> {
        if let Some(value) = self.result.get(key) {
            if value.is_string() {
                Ok(value.as_str().unwrap().to_string())
            } else {
                Err(format!("'{}' is not a string!", key))
            }
        } else {
            Err(format!("'{}' does not exist!", key))
        }
    }

    pub fn get_array(
        &self,
        key: &str
    ) -> Result<&Vec<serde_json::Value>, String> {
        if let Some(value) = self.result.get(key) {
            if value.is_array() {
                Ok(value.as_array().unwrap())
            } else {
                Err(format!("'{}' is not an array!", key))
            }
        } else {
            Err(format!("'{}' does not exist!", key))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LmsPlayer {
    pub name: String,
    pub playerid: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlaylistMode {
    STOP,
    PLAY,
    PAUSE,
}

#[derive(Clone, Debug)]
pub enum RepeatMode {
    NONE,
    TRACK,
    PLAYLIST,
}

#[derive(Clone, Debug)]
pub enum ShuffleMode {
    NONE,
    TRACK,
    ALBUM,
}

#[derive(Clone, Debug)]
pub struct LmsStatus {
    pub player_name: String,
    pub playlist_index: u64,
    pub playlist_repeat: RepeatMode,
    pub playlist_shuffle: ShuffleMode,
    pub playlist_mode: PlaylistMode,
    pub total_tracks: u64,
    pub elapsed_duration: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LmsPlaylist {
    pub tracks: Vec<LmsSong>,
}

impl LmsPlaylist {
    pub fn from(tracks: Vec<LmsSong>) -> Self {
        Self { tracks, }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LmsSong {
    #[serde(rename = "playlist index")]
    pub index: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
}
