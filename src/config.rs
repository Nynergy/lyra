use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub lms_ip: String,
    pub lms_port: String,
    #[serde(default = "Config::default_colors")]
    pub colors: HashMap<String, u8>,
    #[serde(default = "Config::default_colors")]
    default_colors: HashMap<String, u8>,
}

impl Config {
    fn default_colors() -> HashMap<String, u8> {
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

        colors
    }

    pub fn default() -> Self {
        Self {
            lms_ip: "127.0.0.1".to_string(),
            lms_port: "9000".to_string(),
            colors: Config::default_colors(),
            default_colors: Config::default_colors(),
        }
    }

    pub fn color(&self, name: &str) -> &u8 
    {
        self.colors.get(name)
            .unwrap_or_else(|| {
                self.default_color(name)
            })
    }

    fn default_color(&self, name: &str) -> &u8 {
        self.default_colors.get(name)
            .expect(
                &format!("'{}' is not a valid config option", name)
            )
    }
}
