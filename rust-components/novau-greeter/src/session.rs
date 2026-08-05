//! Enumerate installable sessions and spawn them.

use crate::auth::PamSession;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: String,           // file stem, e.g. "novau"
    pub name: String,         // pretty name, e.g. "NovauOS"
    pub exec: String,         // command line
    pub icon: Option<String>,
    pub kind: SessionKind,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionKind { Wayland, X11 }

const SESSION_DIRS: &[&str] = &[
    "/usr/share/wayland-sessions",
    "/usr/share/xsessions",
];

pub fn enumerate() -> Result<Vec<Session>> {
    let mut out = Vec::new();
    for dir in SESSION_DIRS {
        let p = Path::new(dir);
        if !p.is_dir() { continue; }
        let kind = if dir.ends_with("xsessions") { SessionKind::X11 } else { SessionKind::Wayland };
        for entry in std::fs::read_dir(p)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") { continue; }
            if let Some(s) = parse_desktop_file(&path, kind) { out.push(s); }
        }
    }
    // Novau session first if present
    out.sort_by_key(|s| !(s.id == "novau"));
    Ok(out)
}

fn parse_desktop_file(path: &Path, kind: SessionKind) -> Option<Session> {
    let txt = std::fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    for line in txt.lines() {
        if let Some(v) = line.strip_prefix("Name=") { name = Some(v.trim().to_string()); }
        else if let Some(v) = line.strip_prefix("Exec=") { exec = Some(v.trim().to_string()); }
        else if let Some(v) = line.strip_prefix("Icon=") { icon = Some(v.trim().to_string()); }
    }
    let id = path.file_stem()?.to_string_lossy().into_owned();
    Some(Session {
        id,
        name: name.unwrap_or_else(|| "Unknown".into()),
        exec: exec.unwrap_or_default(),
        icon,
        kind,
    })
}

/// Spawn a session for `user` after successful auth.
///
/// Uses `systemd-run` to start the session in a scope so that
/// `systemd --user` activates correctly and logind tracks the session.
pub fn spawn(uid: u32, user: &str, session: &Session) -> Result<()> {
    log::info!("spawning session {:?} for uid={} user={}", session.id, uid, user);

    // Open the PAM session — this creates the `XDG_RUNTIME_DIR` and
    // sets up logind. Held for the lifetime of the spawned session.
    let _pam = PamSession::open(user, "")
        .map_err(|e| anyhow!("pam_open_session: {e}"))?;

    // Build env vector
    let mut env: Vec<(&str, String)> = Vec::new();
    env.push(("XDG_SESSION_TYPE",
        if session.kind == SessionKind::Wayland { "wayland".into() } else { "x11".into() }));
    env.push(("XDG_SESSION_DESKTOP", session.id.clone()));
    env.push(("XDG_CURRENT_DESKTOP", session.id.to_uppercase()));
    env.push(("USER", user.to_string()));
    env.push(("LOGNAME", user.to_string()));
    env.push(("HOME", format!("/home/{user}")));

    // systemd-run --scope --uid=<uid> -- env <session.exec>
    let mut cmd = std::process::Command::new("systemd-run");
    cmd.arg("--scope")
       .arg("--uid").arg(uid.to_string())
       .arg("--setenv").arg("XDG_SESSION_TYPE=wayland")
       .arg("--");
    for tok in shell_split(&session.exec) {
        cmd.arg(tok);
    }
    cmd.env_clear();
    for (k, v) in env {
        cmd.env(k, v);
    }

    let status = cmd.status()
        .map_err(|e| anyhow!("systemd-run: {e}"))?;
    if !status.success() {
        return Err(anyhow!("session exited with status {status}"));
    }
    Ok(())
}

fn shell_split(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

/// Helper: resolve `/usr/share/wayland-sessions/novau.desktop` etc.
pub fn desktop_file_path(id: &str) -> Option<PathBuf> {
    for dir in SESSION_DIRS {
        let p = Path::new(dir).join(format!("{id}.desktop"));
        if p.exists() { return Some(p); }
    }
    None
}
