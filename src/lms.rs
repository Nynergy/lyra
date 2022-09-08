use core::time::Duration;
use std::fmt;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    net,
};

type JsonValue = serde_json::Value;

pub struct LmsClient {
    client: reqwest::Client,
    socket: net::SocketAddr,
}

impl LmsClient {
    pub fn from(socket_str: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_millis(3000))
                .build()
                .expect("Could not build reqwest client"),
            socket: socket_str.parse()
                .expect("Unable to parse socket address"),
        }
    }

    pub async fn query(
        &self,
        command: JsonValue
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(format!(
                "http://{}/jsonrpc.js",
                self.socket.to_string()
            ))
            .json(&json!({
                "method": "slim.request",
                "params": command
            }))
            .send().await
    }
}

#[derive(Debug, Deserialize)]
pub struct LmsResponse {
    result: HashMap::<String, JsonValue>,
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
    ) -> Result<&Vec<JsonValue>, String> {
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

#[derive(Clone, Debug, Deserialize)]
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

impl PlaylistMode {
    pub fn from(text: &str) -> Self {
        match text {
            "play" => PlaylistMode::PLAY,
            "stop" => PlaylistMode::STOP,
            "pause" => PlaylistMode::PAUSE,
            _ => unreachable!()
        }
    }
}

impl fmt::Display for PlaylistMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlaylistMode::STOP => write!(f, "STOPPED"),
            PlaylistMode::PLAY => write!(f, "PLAYING"),
            PlaylistMode::PAUSE => write!(f, "PAUSED"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RepeatMode {
    NONE,
    TRACK,
    PLAYLIST,
}

impl RepeatMode {
    pub fn from(num: u64) -> Self {
        match num {
            1 => RepeatMode::TRACK,
            2 => RepeatMode::PLAYLIST,
            _ => RepeatMode::NONE,
        }
    }
}

impl fmt::Display for RepeatMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RepeatMode::NONE => write!(f, "-"),
            RepeatMode::TRACK => write!(f, "r"),
            RepeatMode::PLAYLIST => write!(f, "R"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ShuffleMode {
    NONE,
    TRACK,
    ALBUM,
}

impl ShuffleMode {
    pub fn from(num: u64) -> Self {
        match num {
            1 => ShuffleMode::TRACK,
            2 => ShuffleMode::ALBUM,
            _ => ShuffleMode::NONE,
        }
    }
}

impl fmt::Display for ShuffleMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShuffleMode::NONE => write!(f, "-"),
            ShuffleMode::TRACK => write!(f, "z"),
            ShuffleMode::ALBUM => write!(f, "Z"),
        }
    }
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

impl LmsSong {
    pub fn default() -> Self {
        Self {
            index: 0,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            duration: 0.1,
        }
    }
}
