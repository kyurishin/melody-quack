//! Remembered between launches: name, critter, server, volume.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub name: String,
    pub avatar: u8,
    pub server: String,
    pub dj_key: String,
    pub volume: f32,
    pub crt: bool,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            name: String::new(),
            avatar: 0,
            server: option_env!("MQ_DEFAULT_SERVER").unwrap_or("ws://127.0.0.1:7777").to_string(),
            dj_key: String::new(),
            volume: 0.8,
            crt: true,
            fullscreen: false,
        }
    }
}

fn path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let mut p = PathBuf::from(home);
    if cfg!(target_os = "macos") {
        p.push("Library/Application Support/Melody Quack");
    } else if cfg!(windows) {
        p.push("AppData/Roaming/Melody Quack");
    } else {
        p.push(".config/melody-quack");
    }
    Some(p.join("settings.json"))
}

impl Settings {
    pub fn load() -> Settings {
        if std::env::var_os("MQ_AUTOPILOT").is_some() {
            return Settings::default();
        }
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if std::env::var_os("MQ_AUTOPILOT").is_some() {
            return;
        }
        if let Some(p) = path() {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(s) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, s);
            }
        }
    }
}
