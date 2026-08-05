//! iced-based UI for the greeter.
//!
//! Layout:
//!   ┌──────────────────────────────────────────────┐
//!   │                                              │
//!   │              NovauOS  logo                   │
//!   │                                              │
//!   │   [ user list  ]   [ password input    ]    │
//!   │                    [ session picker  ▾]      │
//!   │                    [ Login ]                 │
//!   │                                              │
//!   │              v0.1.0  (c) 2026                │
//!   └──────────────────────────────────────────────┘
//!
//! Theme: dark, accent #6ED6A3 (Novau green).
//!
//! Note: iced 0.12's `pick_list` requires `T: ToString + PartialEq + Clone`.
//! `stack` (overlay) was added in iced 0.13, so we don't use it here.

use crate::session::Session;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Application, Command, Element, Length, Theme};
use novau_common::SystemUser;

pub struct Greeter {
    users: Vec<SystemUser>,
    sessions: Vec<Session>,
    selected_user: Option<usize>,
    password: String,
    selected_session: Option<Session>,
    error: Option<String>,
    authenticating: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    UserSelected(usize),
    PasswordChanged(String),
    SessionSelected(Session),
    LoginPressed,
    LoginResult(Result<(), String>),
    Cancel,
}

pub struct GreeterState {
    pub users: Vec<SystemUser>,
    pub sessions: Vec<Session>,
}

impl GreeterState {
    pub fn new(users: Vec<SystemUser>, sessions: Vec<Session>) -> Self {
        Self { users, sessions }
    }
}

impl Application for Greeter {
    type Message = Message;
    type Theme = Theme;
    type Flags = GreeterState;
    type Executor = iced::executor::Default;

    fn new(flags: GreeterState) -> (Self, Command<Message>) {
        (
            Self {
                users: flags.users,
                sessions: flags.sessions,
                selected_user: None,
                password: String::new(),
                selected_session: None,
                error: None,
                authenticating: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("{} — Login", novau_common::DISTRO_NAME)
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::UserSelected(i) => {
                self.selected_user = Some(i);
                self.password.clear();
                self.error = None;
                self.selected_session = self.sessions.iter()
                    .find(|s| s.id == "novau")
                    .or_else(|| self.sessions.first())
                    .cloned();
            }
            Message::PasswordChanged(s) => self.password = s,
            Message::SessionSelected(s) => self.selected_session = Some(s),
            Message::LoginPressed if !self.authenticating => {
                let Some(uidx) = self.selected_user else {
                    self.error = Some("No user selected.".into());
                    return Command::none();
                };
                let user = self.users[uidx].name.clone();
                let pw = self.password.clone();
                let session = self.selected_session.clone();
                self.authenticating = true;
                self.error = None;
                return Command::perform(
                    async move {
                        if let Err(e) = crate::auth::authenticate(&user, &pw) {
                            return Err(format!("Authentication failed: {e}"));
                        }
                        if let Some(sess) = session.as_ref() {
                            if let Err(e) = crate::session::spawn(self_uid(&user), &user, sess) {
                                return Err(format!("Failed to start session: {e}"));
                            }
                        }
                        Ok(())
                    },
                    Message::LoginResult,
                );
            }
            Message::LoginPressed => {}
            Message::LoginResult(res) => {
                self.authenticating = false;
                if let Err(e) = res {
                    self.error = Some(e);
                    self.password.clear();
                }
            }
            Message::Cancel => {
                self.selected_user = None;
                self.password.clear();
                self.error = None;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let title = text(format!("{}", novau_common::DISTRO_NAME))
            .size(48);

        let subtitle = text("Sign in")
            .size(20);

        let body: Element<Message> = if let Some(i) = self.selected_user {
            self.password_view(i)
        } else {
            self.user_list_view()
        };

        let col = column![
            title,
            subtitle,
            body,
            text(format!("v{} — {}", novau_common::DISTRO_VERSION, novau_common::DISTRO_CODENAME))
                .size(12),
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .padding(80)
            .into()
    }

    fn theme(&self) -> Theme { Theme::Dark }
}

impl Greeter {
    fn user_list_view(&self) -> Element<Message> {
        let mut col = column![].spacing(8).width(Length::Fixed(360.0));
        for (i, u) in self.users.iter().enumerate() {
            let label = match &u.real_name {
                Some(r) => format!("{r}  ({})", u.name),
                None => u.name.clone(),
            };
            let btn = button(text(label).size(18))
                .width(Length::Fill)
                .on_press(Message::UserSelected(i));
            col = col.push(btn);
        }
        scrollable(col).width(Length::Fixed(360.0)).into()
    }

    fn password_view(&self, i: usize) -> Element<Message> {
        let user = &self.users[i];
        let name = user.real_name.clone().unwrap_or_else(|| user.name.clone());

        // iced 0.12: text_input takes (placeholder, value); chain `.on_input` for changes.
        let pw_input = text_input("Password", &self.password)
            .on_input(Message::PasswordChanged)
            .size(20)
            .width(Length::Fixed(320.0));

        let session_picker = pick_list(
            self.sessions.clone(),
            self.selected_session.clone(),
            Message::SessionSelected,
        )
        .width(Length::Fixed(320.0));

        let btns = if self.authenticating {
            row![
                button("Cancel").width(Length::Fixed(140.0)),
                button("Signing in…").width(Length::Fixed(160.0)),
            ].spacing(12)
        } else {
            row![
                button("Cancel").on_press(Message::Cancel).width(Length::Fixed(140.0)),
                button("Sign in").on_press(Message::LoginPressed).width(Length::Fixed(160.0)),
            ].spacing(12)
        };

        let mut col = column![
            text(name).size(28),
            pw_input,
            session_picker,
            btns,
        ]
        .spacing(14)
        .align_items(Alignment::Center);

        if let Some(e) = &self.error {
            col = col.push(text(e));
        }

        col.into()
    }
}

fn self_uid(user: &str) -> u32 {
    if let Ok(txt) = std::fs::read_to_string("/etc/passwd") {
        for line in txt.lines() {
            let f: Vec<&str> = line.split(':').collect();
            if f.len() > 2 && f[0] == user {
                if let Ok(u) = f[2].parse() { return u; }
            }
        }
    }
    1000
}
