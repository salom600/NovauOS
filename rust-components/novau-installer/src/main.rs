//! novau-installer — single-binary system installer.
//!
//! Steps:
//!   1. Welcome
//!   2. Language & timezone
//!   3. Disk selection (with partitioning scheme: whole-disk / alongside / manual)
//!   4. User creation
//!   5. Summary
//!   6. Install (partition → format → copy squashfs → chroot setup → bootloader)
//!   7. Done
//!
//! Backed by:
//!   - parted / mkfs.* via subprocess
//!   - rsync to copy squashfs-extracted rootfs
//!   - chroot + apt to set up bootloader, fstab, initramfs

mod steps;

use anyhow::Result;
use iced::{Application, Command, Settings};

fn main() -> Result<()> {
    novau_common::init_logging("installer");
    log::info!("starting novau-installer");

    Installer::run(Settings::with_flags(InstallerState::new()))?;
    Ok(())
}

pub struct Installer {
    state: InstallerState,
}

pub struct InstallerState {
    pub step: steps::Step,
    pub language: String,
    pub timezone: String,
    pub disk: Option<String>,
    pub user_name: String,
    pub user_full: String,
    pub user_pass: String,
    pub hostname: String,
    pub progress: f32,
    pub log: Vec<String>,
    pub error: Option<String>,
}

impl InstallerState {
    pub fn new() -> Self {
        Self {
            step: steps::Step::Welcome,
            language: "en_US.UTF-8".into(),
            timezone: "UTC".into(),
            disk: None,
            user_name: String::new(),
            user_full: String::new(),
            user_pass: String::new(),
            hostname: "novauos".into(),
            progress: 0.0,
            log: Vec::new(),
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    SetLanguage(String),
    SetTimezone(String),
    SetDisk(String),
    SetUserName(String),
    SetUserFull(String),
    SetUserPass(String),
    SetHostname(String),
    InstallProgress(f32, String),
    InstallDone,
    InstallFailed(String),
    Quit,
}

impl Application for Installer {
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = InstallerState;
    type Executor = iced::executor::Default;

    fn new(state: InstallerState) -> (Self, Command<Message>) {
        (Self { state }, Command::none())
    }

    fn title(&self) -> String { "NovauOS Installer".into() }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Next => { self.state.step = self.state.step.next(); Command::none() }
            Message::Back => { self.state.step = self.state.step.prev(); Command::none() }
            Message::SetLanguage(v) => { self.state.language = v; Command::none() }
            Message::SetTimezone(v) => { self.state.timezone = v; Command::none() }
            Message::SetDisk(v) => { self.state.disk = Some(v); Command::none() }
            Message::SetUserName(v) => { self.state.user_name = v; Command::none() }
            Message::SetUserFull(v) => { self.state.user_full = v; Command::none() }
            Message::SetUserPass(v) => { self.state.user_pass = v; Command::none() }
            Message::SetHostname(v) => { self.state.hostname = v; Command::none() }
            Message::InstallProgress(p, line) => {
                self.state.progress = p;
                self.state.log.push(line);
                Command::none()
            }
            Message::InstallDone => { self.state.step = steps::Step::Done; Command::none() }
            Message::InstallFailed(e) => { self.state.error = Some(e); Command::none() }
            Message::Quit => std::process::exit(0),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        steps::view(&self.state)
    }
}
