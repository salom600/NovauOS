//! novau-settings — settings UI + daemon.
//!
//! Panes: Display · Sound · Network · Power · Users · About
//! Backed by D-Bus calls to existing services (BlueZ, NetworkManager,
//! UPower, systemd-logind, colord).

mod config;
mod panes;

use anyhow::Result;
use iced::{Application, Settings};

fn main() -> Result<()> {
    novau_common::init_logging("settings");
    log::info!("starting novau-settings");

    let cfg = config::Settings::load()?;
    SettingsApp::run(Settings::with_flags(SettingsState {
        cfg,
        active: panes::Pane::Display,
    }))?;
    Ok(())
}

pub struct SettingsApp {
    state: SettingsState,
}

pub struct SettingsState {
    pub cfg: config::Settings,
    pub active: panes::Pane,
}

#[derive(Debug, Clone)]
pub enum Message {
    Switch(panes::Pane),
    SetWallpaper(String),
    SetDark(bool),
    SetVolume(u32),
    SetBrightness(u32),
    Save,
}

impl Application for SettingsApp {
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = SettingsState;
    type Executor = iced::executor::Default;

    fn new(state: SettingsState) -> (Self, Command<Message>) {
        (Self { state }, Command::none())
    }

    fn title(&self) -> String {
        "Novau Settings".into()
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Switch(p) => {
                self.state.active = p;
                Command::none()
            }
            Message::SetWallpaper(p) => {
                self.state.cfg.appearance.wallpaper = p;
                Command::none()
            }
            Message::SetDark(d) => {
                self.state.cfg.appearance.dark = d;
                Command::none()
            }
            Message::SetVolume(v) => {
                self.state.cfg.sound.volume = v;
                Command::none()
            }
            Message::SetBrightness(v) => {
                self.state.cfg.display.brightness = v;
                Command::none()
            }
            Message::Save => {
                if let Err(e) = self.state.cfg.save() {
                    log::error!("save settings: {e}");
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        use iced::widget::{button, column, container, row, text};
        use iced::{Alignment, Length};

        let mut sidebar = column![].spacing(4).width(Length::Fixed(180.0));
        for pane in panes::Pane::ALL {
            let b = button(text(pane.title()).size(15))
                .width(Length::Fill)
                .on_press(Message::Switch(*pane))
                .style(if self.state.active == *pane {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                });
            sidebar = sidebar.push(b);
        }
        sidebar = sidebar.push(button("Save").on_press(Message::Save).width(Length::Fill));

        let pane_view = panes::view(&self.state);

        let r = row![sidebar, pane_view]
            .spacing(16)
            .align_items(Alignment::Start);

        container(r)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(iced::theme::Container::Transparent)
            .into()
    }
}

use iced::Command;
