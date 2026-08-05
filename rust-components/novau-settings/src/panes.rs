//! Settings panes.

use crate::{Message, SettingsState};
use iced::widget::{column, container, row, slider, text, toggler};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Display,
    Sound,
    Network,
    Power,
    About,
}

impl Pane {
    pub const ALL: &'static [Self] = &[
        Self::Display,
        Self::Sound,
        Self::Network,
        Self::Power,
        Self::About,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Self::Display => "Display",
            Self::Sound => "Sound",
            Self::Network => "Network",
            Self::Power => "Power",
            Self::About => "About",
        }
    }
}

pub fn view(s: &SettingsState) -> Element<Message> {
    let c = match s.active {
        Pane::Display => column![
            text("Display").size(24),
            text(format!("Scaling: {:.1}×", s.cfg.display.scaling)).size(14),
            slider(
                0.5..=2.0,
                s.cfg.display.scaling,
                |v| Message::SetBrightness((v * 100.0) as u32)
            ),
            text(format!("Brightness: {}%", s.cfg.display.brightness)).size(14),
            slider(0..=100, s.cfg.display.brightness, Message::SetBrightness),
            row![
                text("Night light").size(14),
                toggler(None, s.cfg.display.night_light, |_| Message::Save),
            ]
            .spacing(12),
        ],
        Pane::Sound => column![
            text("Sound").size(24),
            text(format!("Volume: {}%", s.cfg.sound.volume)).size(14),
            slider(0..=100, s.cfg.sound.volume, Message::SetVolume),
            row![
                text("Muted").size(14),
                toggler(None, s.cfg.sound.muted, |_| Message::Save),
            ]
            .spacing(12),
        ],
        Pane::Network => column![
            text("Network").size(24),
            row![
                text("Wi-Fi").size(14),
                toggler(None, s.cfg.network.wifi_enabled, |_| Message::Save)
            ]
            .spacing(12),
            row![
                text("Bluetooth").size(14),
                toggler(None, s.cfg.network.bluetooth_enabled, |_| Message::Save)
            ]
            .spacing(12),
            row![
                text("Airplane mode").size(14),
                toggler(None, s.cfg.network.airplane_mode, |_| Message::Save)
            ]
            .spacing(12),
        ],
        Pane::Power => column![
            text("Power").size(24),
            text(format!("Idle dim after {}s", s.cfg.power.idle_dim_seconds)).size(14),
            slider(10..=600, s.cfg.power.idle_dim_seconds, |_| Message::Save),
            text(format!("Sleep after {}s", s.cfg.power.sleep_seconds)).size(14),
            slider(60..=3600, s.cfg.power.sleep_seconds, |_| Message::Save),
        ],
        Pane::About => {
            let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
                .unwrap_or_default()
                .trim()
                .to_string();
            let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
            let mem_line = meminfo.lines().next().unwrap_or("").to_string();
            column![
                text("About").size(24),
                text(format!("OS:        {}", novau_common::long_version())).size(15),
                text("Base:      Debian 12 (bookworm)").size(15),
                text(format!("Kernel:    {}", kernel)).size(15),
                text(format!(
                    "Compositor: {:?}",
                    novau_common::Compositor::detect()
                ))
                .size(15),
                text(format!("Memory:    {}", mem_line)).size(15),
            ]
        }
    };
    container(c.spacing(14).width(Length::Fixed(600.0)))
        .padding(20)
        .style(iced::theme::Container::Box)
        .into()
}
