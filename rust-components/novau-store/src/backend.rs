//! Backend: aggregates apt, flatpak, wine, appimage sources.

use crate::cache::Cache;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub kind: Kind,
    pub icon: Option<String>,
    pub rating: f32,
    pub installed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Apt,
    Flatpak,
    Wine,
    AppImage,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Apt => "apt",
            Kind::Flatpak => "flatpak",
            Kind::Wine => "wine",
            Kind::AppImage => "appimage",
        }
    }
}

pub struct Backend {
    pub cache: Cache,
}

impl Backend {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let mut out = Vec::new();
        out.extend(self.search_apt(query).await.unwrap_or_default());
        out.extend(self.search_flatpak(query).await.unwrap_or_default());
        out.extend(self.search_appimage(query).await.unwrap_or_default());
        Ok(out)
    }

    async fn search_apt(&self, q: &str) -> Result<Vec<Package>> {
        let out = tokio::process::Command::new("apt-cache")
            .arg("search")
            .arg(q)
            .output()
            .await?;
        let txt = String::from_utf8_lossy(&out.stdout);
        let mut pkgs = Vec::new();
        for line in txt.lines().take(50) {
            if let Some((id, summary)) = line.split_once(" - ") {
                pkgs.push(Package {
                    id: id.trim().to_string(),
                    name: id.trim().to_string(),
                    summary: summary.trim().to_string(),
                    kind: Kind::Apt,
                    icon: None,
                    rating: 0.0,
                    installed: false,
                });
            }
        }
        Ok(pkgs)
    }

    async fn search_flatpak(&self, q: &str) -> Result<Vec<Package>> {
        let out = tokio::process::Command::new("flatpak")
            .arg("search")
            .arg("--columns=application,description")
            .arg(q)
            .output()
            .await;
        let out = match out {
            Ok(o) => o,
            Err(_) => return Ok(Vec::new()),
        };
        if !out.status.success() {
            return Ok(Vec::new());
        }
        let txt = String::from_utf8_lossy(&out.stdout);
        let mut pkgs = Vec::new();
        for line in txt.lines().skip(1) {
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() == 2 {
                pkgs.push(Package {
                    id: parts[0].to_string(),
                    name: parts[0].to_string(),
                    summary: parts[1].to_string(),
                    kind: Kind::Flatpak,
                    icon: None,
                    rating: 0.0,
                    installed: false,
                });
            }
        }
        Ok(pkgs)
    }

    async fn search_appimage(&self, q: &str) -> Result<Vec<Package>> {
        // AppImageHub catalog: https://appimage.github.io/feed.json
        // For the scaffold, we just hit the feed.
        let url = format!("https://appimage.github.io/feed.json");
        if let Ok(resp) = reqwest::get(&url).await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let mut pkgs = Vec::new();
                if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
                    for item in items
                        .iter()
                        .filter(|i| {
                            i.get("name")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_lowercase().contains(&q.to_lowercase()))
                                .unwrap_or(false)
                        })
                        .take(20)
                    {
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        pkgs.push(Package {
                            id: name.clone(),
                            name: name.clone(),
                            summary: item
                                .get("abstract")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            kind: Kind::AppImage,
                            icon: None,
                            rating: 0.0,
                            installed: false,
                        });
                    }
                }
                return Ok(pkgs);
            }
        }
        Ok(Vec::new())
    }

    pub async fn install(&self, pkg: &Package) -> Result<()> {
        match pkg.kind {
            Kind::Apt => {
                let s = tokio::process::Command::new("pkexec")
                    .arg("apt-get")
                    .arg("install")
                    .arg("-y")
                    .arg(&pkg.id)
                    .status()
                    .await?;
                if !s.success() {
                    return Err(anyhow::anyhow!("apt install failed"));
                }
            }
            Kind::Flatpak => {
                let s = tokio::process::Command::new("flatpak")
                    .arg("install")
                    .arg("-y")
                    .arg("flathub")
                    .arg(&pkg.id)
                    .status()
                    .await?;
                if !s.success() {
                    return Err(anyhow::anyhow!("flatpak install failed"));
                }
            }
            Kind::Wine => {
                // Wine runner install handled by proton-ge integration
                return Err(anyhow::anyhow!("Wine runner install not implemented yet"));
            }
            Kind::AppImage => {
                let s = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("appimage-install {}", pkg.id))
                    .status()
                    .await?;
                if !s.success() {
                    return Err(anyhow::anyhow!("AppImage install failed"));
                }
            }
        }
        Ok(())
    }
}
