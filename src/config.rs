use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub lms_ip: String,
    pub lms_port: String,
    pub colors: HashMap<String, u8>,
}

impl Config {
    pub fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("Banner".to_string(), 2);
        colors.insert("PlayerName".to_string(), 1);
        colors.insert("PlayingIndicator".to_string(), 2);
        colors.insert("PausedIndicator".to_string(), 3);
        colors.insert("StoppedIndicator".to_string(), 1);
        colors.insert("RepeatIndicator".to_string(), 5);
        colors.insert("ShuffleIndicator".to_string(), 6);
        colors.insert("TrackIndex".to_string(), 5);
        colors.insert("TrackTitle".to_string(), 3);
        colors.insert("TrackArtist".to_string(), 4);
        colors.insert("TrackAlbum".to_string(), 1);
        colors.insert("TrackDuration".to_string(), 6);
        colors.insert("PlaybarGauge".to_string(), 2);

        Self {
            lms_ip: "127.0.0.1".to_string(),
            lms_port: "9000".to_string(),
            colors,
        }
    }

    pub fn color(&self, name: &str) -> &u8 
    {
        self.colors.get(name)
            .expect(
                &format!("'{}' is not a valid color name", name)
            )
    }
}
