//! novau-welcome — first-boot onboarding.
//!
//! Pages: Welcome → Language → Timezone → Account picture → Telemetry → Done.
//! In Live mode, a prominent "Install NovauOS" button launches the installer.

use anyhow::Result;
use iced::widget::{button, column, container, image, row, text, text_input};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};

fn main() -> Result<()> {
    novau_common::init_logging("welcome");
    log::info!("starting novau-welcome");

    Welcome::run(Settings::with_flags(WelcomeState::default()))?;
    Ok(())
}

pub struct Welcome {
    state: WelcomeState,
}

#[derive(Default)]
pub struct WelcomeState {
    pub page: Page,
    pub language: String,
    pub timezone: String,
    pub live_mode: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    #[default]
    Welcome,
    Language,
    Timezone,
    Done,
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Install,
    SetLanguage(String),
    SetTimezone(String),
    Quit,
}

impl Application for Welcome {
    type Message = Message;
    type Theme = Theme;
    type Flags = WelcomeState;
    type Executor = iced::executor::Default;

    fn new(mut state: WelcomeState) -> (Self, Command<Message>) {
        state.live_mode = std::path::Path::new("/cdrom/.disk/info").exists()
            || std::env::var("NOVAU_LIVE").is_ok();
        (Self { state }, Command::none())
    }

    fn title(&self) -> String {
        format!("{} — Welcome", novau_common::DISTRO_NAME)
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Next => {
                self.state.page = match self.state.page {
                    Page::Welcome => Page::Language,
                    Page::Language => Page::Timezone,
                    Page::Timezone => Page::Done,
                    Page::Done => Page::Done,
                };
                Command::none()
            }
            Message::Install => {
                let _ = std::process::Command::new("novau-installer").spawn();
                Command::none()
            }
            Message::SetLanguage(s) => {
                self.state.language = s;
                Command::none()
            }
            Message::SetTimezone(s) => {
                self.state.timezone = s;
                Command::none()
            }
            Message::Quit => std::process::exit(0),
        }
    }

    fn view(&self) -> Element<Message> {
        let body: Element<_> = match self.state.page {
            Page::Welcome => {
                let mut c = column![
                    text("Welcome to NovauOS").size(36),
                    text("Lightweight. Modern. Built on Rust.").size(18),
                ]
                .spacing(12)
                .align_items(Alignment::Center);
                if self.state.live_mode {
                    c = c.push(
                        button(text("Install NovauOS").size(18))
                            .on_press(Message::Install)
                            .padding(12)
                            .width(Length::Fixed(220.0)),
                    );
                }
                c.into()
            }
            Page::Language => column![
                text("Choose your language").size(24),
                text_input("Language", &self.state.language)
                    .on_input(Message::SetLanguage)
                    .size(18),
            ]
            .spacing(12)
            .into(),
            Page::Timezone => column![
                text("Select your timezone").size(24),
                text_input("Timezone", &self.state.timezone)
                    .on_input(Message::SetTimezone)
                    .size(18),
            ]
            .spacing(12)
            .into(),
            Page::Done => column![
                text("You're all set!").size(32),
                text("Click Finish to start using NovauOS.").size(16),
                button("Finish").on_press(Message::Quit).padding(10),
            ]
            .spacing(12)
            .align_items(Alignment::Center)
            .into(),
        };

        let mut nav = row![];
        if self.state.page != Page::Done {
            nav = nav.push(
                button("Next")
                    .on_press(Message::Next)
                    .width(Length::Fixed(140.0)),
            );
        }

        let c = column![body, nav]
            .spacing(24)
            .align_items(Alignment::Center);
        container(c)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(60)
            .style(iced::theme::Container::Transparent)
            .into()
    }
}
