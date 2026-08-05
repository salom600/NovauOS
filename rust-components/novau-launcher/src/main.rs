//! novau-launcher — application launcher.
//!
//! Rofi/dmenu replacement. Reads `.desktop` files, fuzzy-searches them,
//! launches the selected one via `systemd-run --user`.

mod desktop;
mod server;

use anyhow::Result;
use iced::widget::{button, column, container, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings};
use std::sync::Arc;

fn main() -> Result<()> {
    novau_common::init_logging("launcher");
    log::info!("starting novau-launcher");

    // Spawn the IPC server so the panel can wake us on hot-key.
    let apps = Arc::new(desktop::load_all());
    log::info!("indexed {} apps", apps.len());

    {
        let apps = Arc::clone(&apps);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(server::listen(apps));
        });
    }

    Launcher::run(Settings::with_flags(LauncherState {
        apps,
        query: String::new(),
        selected: 0,
    }))?;
    Ok(())
}

pub struct LauncherState {
    pub apps: Arc<Vec<desktop::App>>,
    pub query: String,
    pub selected: usize,
}

pub struct Launcher {
    state: LauncherState,
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    MoveSelection(i32),
    Launch(usize),
    LaunchSelected,
    Quit,
}

impl Application for Launcher {
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = LauncherState;
    type Executor = iced::executor::Default;

    fn new(state: LauncherState) -> (Self, Command<Message>) {
        (Self { state }, Command::none())
    }

    fn title(&self) -> String { "Novau Launcher".into() }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::QueryChanged(q) => { self.state.query = q; self.state.selected = 0; }
            Message::MoveSelection(d) => {
                let n = self.filtered().len();
                if n == 0 { return Command::none(); }
                let i = (self.state.selected as i32 + d).rem_euclid(n as i32) as usize;
                self.state.selected = i;
            }
            Message::Launch(i) => {
                let list = self.filtered();
                if let Some(app) = list.get(i) {
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&app.exec)
                        .spawn();
                }
                return Command::perform(async {}, |_| Message::Quit);
            }
            Message::LaunchSelected => {
                let idx = self.state.selected;
                return Command::perform(async {}, move |_| Message::Launch(idx));
            }
            Message::Quit => std::process::exit(0),
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let input = text_input("Search apps…", &self.state.query)
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchSelected)
            .size(22)
            .width(Length::Fixed(560.0));

        let list = self.filtered();
        let mut col = column![input].spacing(4).max_width(560);
        for (i, app) in list.iter().take(10).enumerate() {
            let b = button(text(format!("{}  —  {}", app.name, app.exec)).size(15))
                .width(Length::Fill)
                .on_press(Message::Launch(i));
            col = col.push(b);
        }

        let c = container(col)
            .padding(20)
            .width(Length::Fixed(600.0));

        container(c)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

impl Launcher {
    fn filtered(&self) -> Vec<desktop::App> {
        let q = self.state.query.trim().to_lowercase();
        if q.is_empty() {
            return self.state.apps.iter().take(10).cloned().collect();
        }
        // Simple substring + initialism match. Good enough for v0.1.
        self.state.apps.iter()
            .filter(|a| {
                let name = a.name.to_lowercase();
                let gen = a.generic_name.as_deref().unwrap_or("").to_lowercase();
                name.contains(&q) || gen.contains(&q) || is_initialism(&q, &name)
            })
            .take(10)
            .cloned()
            .collect()
    }
}

fn is_initialism(query: &str, target: &str) -> bool {
    // "nvim" matches "Neovim" by taking first letters of each word.
    let words: Vec<&str> = target.split_whitespace().collect();
    if words.is_empty() { return false; }
    let init: String = words.iter()
        .filter_map(|w| w.chars().next())
        .collect();
    init.to_lowercase().starts_with(query)
}
