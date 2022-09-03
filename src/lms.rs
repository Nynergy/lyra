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
    pub fn dump(&self) {
        println!("{:?}", self.result);
    }

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
}
