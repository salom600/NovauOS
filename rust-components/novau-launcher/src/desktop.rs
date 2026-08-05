//! `.desktop` file parser.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String, // file stem
    pub name: String,
    pub generic_name: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub terminal: bool,
    pub path: PathBuf,
}

const SEARCH_DIRS: &[&str] = &[
    "/usr/share/applications",
    "/usr/local/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];

pub fn load_all() -> Vec<App> {
    let mut out = Vec::new();
    for d in SEARCH_DIRS {
        let p = std::path::Path::new(d);
        if !p.is_dir() {
            continue;
        }
        if let Ok(rd) = std::fs::read_dir(p) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }
                if let Some(app) = parse(&path) {
                    out.push(app);
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn parse(path: &PathBuf) -> Option<App> {
    let txt = std::fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut generic = None;
    let mut exec = None;
    let mut icon = None;
    let mut cats = None;
    let mut terminal = false;
    let mut no_display = false;
    let mut in_desktop_entry = false;

    for line in txt.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_desktop_entry = t == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        if let Some(v) = t.strip_prefix("Name=") {
            name = Some(v.trim().to_string());
        } else if let Some(v) = t.strip_prefix("GenericName=") {
            generic = Some(v.trim().to_string());
        } else if let Some(v) = t.strip_prefix("Exec=") {
            exec = Some(v.trim().to_string());
        } else if let Some(v) = t.strip_prefix("Icon=") {
            icon = Some(v.trim().to_string());
        } else if let Some(v) = t.strip_prefix("Categories=") {
            cats = Some(
                v.split(';')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect(),
            );
        } else if let Some(v) = t.strip_prefix("Terminal=") {
            terminal = v.trim() == "true";
        } else if let Some(v) = t.strip_prefix("NoDisplay=") {
            no_display = v.trim() == "true";
        }
    }

    if no_display {
        return None;
    }
    let id = path.file_stem()?.to_string_lossy().into_owned();
    Some(App {
        id,
        name: name.unwrap_or_else(|| "Unknown".into()),
        generic_name: generic,
        exec: exec.unwrap_or_default(),
        icon,
        categories: cats.unwrap_or_default(),
        terminal,
        path: path.clone(),
    })
}
