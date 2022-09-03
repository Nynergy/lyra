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
    // NOTE: Used for debugging only
    #[allow(dead_code)]
    pub fn dump(&self) {
        println!("{:?}", self.result);
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_object(
        &self,
        key: &str
    ) -> Result<&serde_json::Map<String, serde_json::Value>, String> {
        if let Some(value) = self.result.get(key) {
            if value.is_object() {
                Ok(value.as_object().unwrap())
            } else {
                Err(format!("'{}' is not an object!", key))
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

#[derive(Debug, Deserialize)]
pub struct LmsPlaylist {
    pub tracks: Vec<LmsSong>,
}

impl LmsPlaylist {
    pub fn from(tracks: Vec<LmsSong>) -> Self {
        Self { tracks, }
    }
}

#[derive(Debug, Deserialize)]
pub struct LmsSong {
    #[serde(rename = "playlist index")]
    pub index: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
}
