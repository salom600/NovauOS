//! novau-panel — Wayland top panel for NovauOS.
//!
//! Uses iced's `layer-shell` (via `iced_layershell` feature, abstracted here)
//! to anchor to the top of the screen. Renders:
//!   - left:   logo + workspaces + window list (via wlr-foreign-toplevel)
//!   - center: clock
//!   - right:  tray (StatusNotifierItem via D-Bus), volume, network, battery
//!
//! Notifications are received on the Novau IPC socket and shown as a
//! transient toast on the right.

mod clock;
mod tray;
mod windows;
mod notifications;

use anyhow::Result;
use iced::{Application, Command, Settings};
use std::time::Duration;

fn main() -> Result<()> {
    novau_common::init_logging("panel");
    log::info!("starting novau-panel");

    let state = PanelState::new();
    Panel::run(Settings::with_flags(state))?;
    Ok(())
}

pub struct Panel {
    state: PanelState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleLauncher,
    ClockTick(chrono::DateTime<chrono::Local>),
    WindowList(Vec<(String, String, bool)>),
    Notification(notifications::Notification),
    DismissNotification(usize),
    SetVolume(u32),
    ToggleMute,
}

#[derive(Default)]
pub struct PanelState {
    pub clock: Option<chrono::DateTime<chrono::Local>>,
    pub windows: Vec<(String, String, bool)>,
    pub notifications: Vec<notifications::Notification>,
    pub volume: u32,
    pub muted: bool,
}

impl PanelState {
    pub fn new() -> Self {
        Self {
            clock: Some(chrono::Local::now()),
            windows: Vec::new(),
            notifications: Vec::new(),
            volume: 50,
            muted: false,
        }
    }
}

impl Application for Panel {
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = PanelState;
    type Executor = iced::executor::Default;

    fn new(state: PanelState) -> (Self, Command<Message>) {
        let cmd = Command::batch(vec![
            Command::perform(async { chrono::Local::now() }, Message::ClockTick),
            Command::perform(clock::tick_loop(), |d| Message::ClockTick(d)),
            Command::perform(notifications::listen(), Message::Notification),
            Command::perform(windows::watch(), Message::WindowList),
        ]);
        (Self { state }, cmd)
    }

    fn title(&self) -> String { "Novau Panel".into() }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Tick => Command::none(),
            Message::ToggleLauncher => {
                // Send an IPC message to the launcher
                let _ = std::os::unix::net::UnixStream::connect(
                    novau_common::paths::runtime().join("launcher.sock"));
                Command::none()
            }
            Message::ClockTick(t) => { self.state.clock = Some(t); Command::none() }
            Message::WindowList(w) => { self.state.windows = w; Command::none() }
            Message::Notification(n) => {
                self.state.notifications.push(n);
                // Auto-dismiss after 5s
                let idx = self.state.notifications.len() - 1;
                Command::perform(tokio::time::sleep(Duration::from_secs(5)), move |_| {
                    Message::DismissNotification(idx)
                })
            }
            Message::DismissNotification(i) => {
                if i < self.state.notifications.len() {
                    self.state.notifications.remove(i);
                }
                Command::none()
            }
            Message::SetVolume(v) => { self.state.volume = v; Command::none() }
            Message::ToggleMute => { self.state.muted = !self.state.muted; Command::none() }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        use iced::widget::{button, column, container, row, text};
        use iced::{Alignment, Color, Length};

        let logo = button(text("❖").size(20).style(iced::theme::Text::Color(Color::from_rgb(0.43, 0.84, 0.64))))
            .on_press(Message::ToggleLauncher)
            .padding(4);

        let wins: iced::Element<_> = if self.state.windows.is_empty() {
            text("").into()
        } else {
            let mut r = row![].spacing(6);
            for (app, _title, focused) in self.state.windows.iter().take(8) {
                let lbl = text(app.as_str()).size(13);
                let b = if *focused {
                    button(lbl).style(iced::theme::Button::Primary)
                } else {
                    button(lbl)
                };
                r = r.push(b.padding(2));
            }
            r.into()
        };

        let clock_text = self.state.clock
            .map(|t| t.format("%a %H:%M").to_string())
            .unwrap_or_default();
        let clock = text(clock_text).size(15);

        let vol_label = if self.state.muted { "🔇" } else { "🔊" };
        let vol = button(text(vol_label).size(14))
            .on_press(Message::ToggleMute)
            .padding(4);

        let mut notif_col = column![].spacing(4).max_width(360);
        for (i, n) in self.state.notifications.iter().enumerate() {
            notif_col = notif_col.push(
                iced::widget::container(
                    column![
                        text(format!("{}", n.app)).size(12).style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.85, 0.7))),
                        text(&n.summary).size(13),
                        text(&n.body).size(12).style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                    ].spacing(2)
                )
                .padding(8)
                .style(iced::theme::Container::Box)
            );
        }

        let left = row![logo, wins].spacing(8).align_items(Alignment::Center);
        let right = row![vol].spacing(8).align_items(Alignment::Center);

        let bar = row![left, clock, right]
            .spacing(16)
            .padding([4, 12])
            .align_items(Alignment::Center)
            .width(Length::Fill);

        let bar = container(bar)
            .width(Length::Fill)
            .height(Length::Fixed(36.0))
            .style(iced::theme::Container::Box);

        column![bar, notif_col].spacing(4).into()
    }
}
