//! Store UI — search bar + result grid.

use crate::backend::{Backend, Package};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Application, Color, Command, Element, Length, Theme};
use std::sync::Arc;

pub struct Store {
    state: StoreState,
}

pub struct StoreState {
    pub backend: Arc<Backend>,
    pub query: String,
    pub results: Vec<Package>,
    pub installing: Option<String>,
    pub status: Option<String>,
}

impl StoreState {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend: Arc::new(backend),
            query: String::new(),
            results: Vec::new(),
            installing: None,
            status: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    Search,
    SearchResult(Vec<Package>),
    Install(Package),
    InstallResult(String, Result<(), String>),
}

impl Application for Store {
    type Message = Message;
    type Theme = Theme;
    type Flags = StoreState;
    type Executor = iced::executor::Default;

    fn new(state: StoreState) -> (Self, Command<Message>) {
        (Self { state }, Command::none())
    }

    fn title(&self) -> String {
        "Novau Store".into()
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::QueryChanged(q) => {
                self.state.query = q;
                Command::none()
            }
            Message::Search => {
                let b = Arc::clone(&self.state.backend);
                let q = self.state.query.clone();
                Command::perform(
                    async move { b.search(&q).await.unwrap_or_default() },
                    Message::SearchResult,
                )
            }
            Message::SearchResult(r) => {
                self.state.results = r;
                Command::none()
            }
            Message::Install(pkg) => {
                let id = pkg.id.clone();
                self.state.installing = Some(id.clone());
                self.state.status = Some(format!("Installing {id}…"));
                let b = Arc::clone(&self.state.backend);
                Command::perform(
                    async move {
                        let r = b.install(&pkg).await.map_err(|e| e.to_string());
                        (id.clone(), r)
                    },
                    |(id, r)| Message::InstallResult(id, r),
                )
            }
            Message::InstallResult(id, res) => {
                self.state.installing = None;
                self.state.status = Some(match res {
                    Ok(()) => format!("Installed {id} ✓"),
                    Err(e) => format!("Failed: {e}"),
                });
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let input = text_input("Search applications, games, tools…", &self.state.query)
            .on_input(Message::QueryChanged)
            .on_submit(Message::Search)
            .size(18)
            .width(Length::Fixed(600.0));

        let mut grid = column![].spacing(8).max_width(900);
        for pkg in &self.state.results {
            let kind_badge = match pkg.kind {
                crate::backend::Kind::Apt => "[apt]",
                crate::backend::Kind::Flatpak => "[flatpak]",
                crate::backend::Kind::Wine => "[wine]",
                crate::backend::Kind::AppImage => "[appimage]",
            };
            let r = row![
                text(format!("{}  {}", kind_badge, pkg.name)).size(16),
                text(pkg.summary.clone())
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                button("Install").on_press(Message::Install(pkg.clone())),
            ]
            .spacing(12)
            .align_items(Alignment::Center)
            .padding(8);
            grid = grid.push(container(r).style(iced::theme::Container::Box));
        }

        let mut col = column![input].spacing(12).align_items(Alignment::Center);
        if let Some(s) = &self.state.status {
            col = col.push(text(s).size(14));
        }
        col = col.push(scrollable(grid).height(Length::Fill));

        container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .center_x()
            .style(iced::theme::Container::Transparent)
            .into()
    }
}
