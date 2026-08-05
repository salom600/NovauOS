//! novau-store — one-click app store.
//!
//! Backend unifies four sources:
//!   1. apt     — `apt-cache search` + `PackageKit` for install
//!   2. flatpak — `flatpak search` + `flatpak install -y`
//!   3. wine    — Proton-GE / Wine-GE runner tarballs from GitHub releases
//!   4. appimage— AppImageHub catalog
//!
//! A local SQLite cache mirrors metadata so the UI is snappy.

mod backend;
mod cache;
mod ui;

use anyhow::Result;
use iced::{Application, Settings};

fn main() -> Result<()> {
    novau_common::init_logging("store");
    log::info!("starting novau-store");

    let cache = cache::Cache::open()?;
    let backend = backend::Backend::new(cache);
    let flags = ui::StoreState::new(backend);

    ui::Store::run(Settings::with_flags(flags))?;
    Ok(())
}
