//! User settings model.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default)]
    pub display: Display,
    #[serde(default)]
    pub sound: Sound,
    #[serde(default)]
    pub power: Power,
    #[serde(default)]
    pub network: Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appearance {
    pub dark: bool,
    pub accent: String,
    pub wallpaper: String,
    pub font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub scaling: f32,
    pub brightness: u32,
    pub night_light: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    pub volume: u32,
    pub muted: bool,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Power {
    pub idle_dim_seconds: u32,
    pub sleep_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_mode: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            dark: true,
            accent: "#6ED6A3".into(),
            wallpaper: "/usr/share/backgrounds/novau/novau-default.png".into(),
            font: "Inter".into(),
        }
    }
}
impl Default for Display {
    fn default() -> Self {
        Self {
            scaling: 1.0,
            brightness: 80,
            night_light: false,
        }
    }
}
impl Default for Sound {
    fn default() -> Self {
        Self {
            volume: 50,
            muted: false,
            output: "auto".into(),
        }
    }
}
impl Default for Power {
    fn default() -> Self {
        Self {
            idle_dim_seconds: 120,
            sleep_seconds: 600,
        }
    }
}
impl Default for Network {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            bluetooth_enabled: false,
            airplane_mode: false,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let p = novau_common::paths::config().join("settings.ron");
        if p.exists() {
            let txt = std::fs::read_to_string(&p)?;
            let s: Self = ron::from_str(&txt)?;
            Ok(s)
        } else {
            Ok(Self {
                appearance: Appearance::default(),
                display: Display::default(),
                sound: Sound::default(),
                power: Power::default(),
                network: Network::default(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let p = novau_common::paths::config().join("settings.ron");
        novau_common::ensure_dir(p.parent().unwrap())?;
        let txt = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(&p, txt)?;
        Ok(())
    }
}
