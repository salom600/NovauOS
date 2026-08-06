//! novau-welcome — first-boot onboarding.
//!
//! Pages: Welcome → Language → Timezone → Done.
//! In Live mode, the Welcome page shows two prominent buttons:
//!   - "Install NovauOS" (primary, green)
//!   - "Try NovauOS" (secondary, just closes the welcome window)
//!
//! This matches the Ubuntu/Fedora/Mint convention so users immediately
//! see how to install vs. just explore the live session.

use anyhow::Result;
use iced::widget::{button, column, container, row, text, text_input};
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
    TryLive,
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
        // Live mode is detected three ways (in order of reliability):
        //   1. /run/live/medium — live-boot bookworm's mount point
        //   2. /cdrom — older live-boot mount point
        //   3. NOVAU_LIVE env var — set by /etc/profile.d/novau.sh
        state.live_mode = std::path::Path::new("/run/live/medium").exists()
            || std::path::Path::new("/cdrom").exists()
            || std::env::var("NOVAU_LIVE").is_ok();
        log::info!("live_mode = {}", state.live_mode);
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
                log::info!("user clicked Install — launching novau-installer");
                match std::process::Command::new("novau-installer").spawn() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("failed to launch novau-installer: {e}");
                        // Fall back to a terminal-based message
                        let _ = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(format!(
                                "xterm -e 'echo Cannot launch installer: {e}; echo; read -p \"Press Enter to close...\"'"
                            ))
                            .spawn();
                    }
                }
                Command::none()
            }
            Message::TryLive => {
                // User chose to explore the live session — close the welcome window.
                std::process::exit(0);
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
                    text(format!("Welcome to {}", novau_common::DISTRO_NAME)).size(40),
                    text("Lightweight. Modern. Built on Rust.").size(20),
                ]
                .spacing(12)
                .align_items(Alignment::Center);

                if self.state.live_mode {
                    // Two-button layout: prominent Install + secondary Try
                    c = c.push(text("").size(12)); // spacer
                    let install_btn = button(text("  Install NovauOS  ").size(20))
                        .on_press(Message::Install)
                        .padding(16)
                        .width(Length::Fixed(280.0))
                        .style(iced::theme::Button::Primary);

                    let try_btn = button(text("  Try NovauOS  ").size(16))
                        .on_press(Message::TryLive)
                        .padding(12)
                        .width(Length::Fixed(200.0));

                    c = c.push(install_btn);
                    c = c.push(text("(explore the live session without installing)").size(12));
                    c = c.push(try_btn);
                } else {
                    // Installed system: just show the Next button to continue onboarding.
                    c = c.push(
                        button(text("  Get Started  ").size(18))
                            .on_press(Message::Next)
                            .padding(14)
                            .width(Length::Fixed(220.0))
                            .style(iced::theme::Button::Primary),
                    );
                }
                c.into()
            }
            Page::Language => column![
                text("Choose your language").size(28),
                text("Examples: en_US.UTF-8, fr_FR.UTF-8, ar_SA.UTF-8").size(12),
                text_input("Language", &self.state.language)
                    .on_input(Message::SetLanguage)
                    .size(18),
            ]
            .spacing(12)
            .into(),
            Page::Timezone => column![
                text("Select your timezone").size(28),
                text("Examples: America/New_York, Europe/London, Asia/Dubai").size(12),
                text_input("Timezone", &self.state.timezone)
                    .on_input(Message::SetTimezone)
                    .size(18),
            ]
            .spacing(12)
            .into(),
            Page::Done => column![
                text("You're all set!").size(36),
                text("Click Finish to start using NovauOS.").size(18),
                button(text("  Finish  ").size(16))
                    .on_press(Message::Quit)
                    .padding(12)
                    .width(Length::Fixed(160.0))
                    .style(iced::theme::Button::Primary),
            ]
            .spacing(12)
            .align_items(Alignment::Center)
            .into(),
        };

        let mut nav = row![];
        // Show "Skip onboarding" in live mode (lets user dismiss without finishing)
        if self.state.live_mode && self.state.page != Page::Welcome {
            nav = nav.push(
                button("Skip")
                    .on_press(Message::TryLive)
                    .width(Length::Fixed(120.0)),
            );
        }
        if self.state.page != Page::Done && self.state.page != Page::Welcome {
            nav = nav.push(
                button("Next")
                    .on_press(Message::Next)
                    .width(Length::Fixed(140.0))
                    .style(iced::theme::Button::Secondary),
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
