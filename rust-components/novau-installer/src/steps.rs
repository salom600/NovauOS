//! Installer wizard steps.

use crate::{InstallerState, Message};
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Locale,
    Disk,
    User,
    Summary,
    Install,
    Done,
}

impl Step {
    pub fn next(self) -> Self {
        match self {
            Self::Welcome => Self::Locale,
            Self::Locale => Self::Disk,
            Self::Disk => Self::User,
            Self::User => Self::Summary,
            Self::Summary => Self::Install,
            Self::Install => Self::Done,
            Self::Done => Self::Done,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::Welcome => Self::Welcome,
            Self::Locale => Self::Welcome,
            Self::Disk => Self::Locale,
            Self::User => Self::Disk,
            Self::Summary => Self::User,
            Self::Install => Self::Summary,
            Self::Done => Self::Install,
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Welcome => "Welcome",
            Self::Locale => "Locale",
            Self::Disk => "Disk",
            Self::User => "User",
            Self::Summary => "Summary",
            Self::Install => "Installing",
            Self::Done => "Done",
        };
        write!(f, "{s}")
    }
}

pub fn view(s: &InstallerState) -> Element<Message> {
    let title = text(format!("NovauOS Installer — {}", s.step)).size(22);

    let body: Element<Message> = match s.step {
        Step::Welcome => column![
            text("Welcome to NovauOS").size(28),
            text("This installer will guide you through setting up NovauOS on your computer.").size(16),
            text("You can quit at any time; no changes will be made until you confirm in the Summary step.").size(14),
        ].spacing(12).into(),

        Step::Locale => column![
            text("Language & timezone").size(20),
            text_input("Language", &s.language).on_input(Message::SetLanguage).size(18),
            text_input("Timezone", &s.timezone).on_input(Message::SetTimezone).size(18),
        ].spacing(12).into(),

        Step::Disk => {
            let disks = enumerate_disks();
            let mut col = column![text("Select installation disk").size(20)].spacing(8);
            for d in disks {
                let active = s.disk.as_deref() == Some(d.as_str());
                let b = button(text(format!("● {d}")).size(16))
                    .on_press(Message::SetDisk(d.clone()))
                    .style(if active { iced::theme::Button::Primary } else { iced::theme::Button::Secondary });
                col = col.push(b);
            }
            col.into()
        }

        Step::User => column![
            text("Create your user").size(20),
            text_input("Username", &s.user_name).on_input(Message::SetUserName).size(18),
            text_input("Full name", &s.user_full).on_input(Message::SetUserFull).size(18),
            text_input("Password", &s.user_pass).on_input(Message::SetUserPass).secure(true).size(18),
            text_input("Computer name", &s.hostname).on_input(Message::SetHostname).size(18),
        ].spacing(12).into(),

        Step::Summary => {
            let disk = s.disk.clone().unwrap_or_default();
            column![
                text("Review your choices").size(20),
                text(format!("Language:  {}", s.language)).size(15),
                text(format!("Timezone:  {}", s.timezone)).size(15),
                text(format!("Disk:       {disk}")).size(15),
                text(format!("User:       {} ({})", s.user_name, s.user_full)).size(15),
                text(format!("Hostname:   {}", s.hostname)).size(15),
                text("⚠ Clicking Install will partition the disk and write data. Back up your files first!").size(14),
            ].spacing(8).into()
        }

        Step::Install => column![
            text(format!("Installing… {:.0}%", s.progress * 100.0)).size(18),
            progress_bar(0.0..=1.0, s.progress),
            scrollable(text(s.log.join("\n")).size(12)).height(Length::Fill),
        ].spacing(12).into(),

        Step::Done => column![
            text("✓ Installation complete").size(28),
            text("You can now restart your computer and boot into NovauOS.").size(16),
            button("Restart").on_press(Message::Quit).width(Length::Fixed(180.0)),
        ].spacing(12).align_items(Alignment::Center).into(),
    };

    let mut nav = row![];
    if s.step != Step::Welcome && s.step != Step::Install && s.step != Step::Done {
        nav = nav.push(
            button("Back")
                .on_press(Message::Back)
                .width(Length::Fixed(140.0)),
        );
    }
    match s.step {
        Step::Welcome | Step::Locale | Step::Disk | Step::User => {
            nav = nav.push(
                button("Next")
                    .on_press(Message::Next)
                    .width(Length::Fixed(140.0)),
            );
        }
        Step::Summary => {
            nav = nav.push(
                button("Install")
                    .on_press(Message::Next)
                    .width(Length::Fixed(160.0))
                    .style(iced::theme::Button::Primary),
            );
        }
        _ => {}
    }

    let col = column![title, body, nav]
        .spacing(24)
        .align_items(Alignment::Center);
    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .padding(40)
        .style(iced::theme::Container::Transparent)
        .into()
}

fn enumerate_disks() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir("/sys/block") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            // Filter out loop/ram devices; keep nvme, sd, vd, mmcblk
            if name.starts_with("sd")
                || name.starts_with("nvme")
                || name.starts_with("vd")
                || name.starts_with("mmcblk")
            {
                out.push(format!("/dev/{name}"));
            }
        }
    }
    out
}
