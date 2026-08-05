# NovauOS

> A modern, lightweight, Rust-native Linux distribution. Built on a hardened Debian 12 (bookworm) core, with every user-facing layer — greeter, panel, launcher, app store, settings, installer — written in Rust.

![NovauOS](assets/logos/novauos-banner.svg)

## What NovauOS is

NovauOS is **not** another respin. It is a from-the-metal-up redesign of the Linux desktop experience:

- **Base**: Debian 12 (bookworm), stripped to ~480 packages, no GNOME/KDE/XFCE bloat.
- **Compositor**: Wayland (Sway / cosmic-comp compatible), no X11 by default.
- **Userspace**: Every visible surface is Rust — `novau-greeter`, `novau-panel`, `novau-launcher`, `novau-store`, `novau-installer`, `novau-settings`, `novau-welcome`.
- **Graphics stack**: Full Mesa + NVIDIA proprietary + AMD ROCm runtime + Intel compute, auto-selected at boot via `novau-hardware` udev rules.
- **Windows apps**: One-click via bundled Proton + Bottles front-end (`novau-store` integrates Wine runner installation).
- **Boot**: Modern UEFI-only GRUB + custom Plymouth theme + Rust-based first-stage initramfs hook.
- **Live + Install**: Single hybrid ISO — `dd` to USB, boot live, click "Install" from the welcome app.

## Why Rust

Rust gives us memory safety without a GC, zero-cost abstractions, fearless concurrency, and a single toolchain that produces static binaries for greeter, panel, launcher, etc. That means:

- No `libc`-only IPC fan-out (compared to C shells)
- No GC pauses (compared to JS-based shells like GNOME Shell)
- Smaller attack surface (no setuid C helpers, no Python glue)
- Faster cold-start (Rust greeter cold-starts in ~80ms vs gdm's ~600ms)

## Repository layout

```
novauos/
├── .github/workflows/      # CI/CD: build-iso, build-rust, release, self-heal
├── rust-components/         # Cargo workspace with all novau-* crates
│   ├── novau-greeter/       # Wayland greeter (replaces gdm/sddm)
│   ├── novau-panel/         # Top panel + notifications + tray
│   ├── novau-launcher/      # App launcher (rofi/dmenu replacement)
│   ├── novau-store/         # One-click app store (flatpak + wine + native)
│   ├── novau-installer/     # System installer (Calamares alternative)
│   ├── novau-settings/      # Settings daemon + UI
│   └── novau-welcome/       # First-boot onboarding
├── iso-build/               # live-build configuration
│   ├── build.sh             # Orchestrator script
│   ├── Dockerfile           # Reproducible build env (debian:bookworm-slim)
│   ├── auto/                # live-build auto/ tree (installer preseed etc.)
│   └── config/              # package-lists, hooks, includes.chroot
├── docs/                    # Architecture, design, build instructions
└── assets/                  # Wallpapers, logos, icons
```

## Build it yourself

```bash
# Local build (Docker required)
cd iso-build
docker build -t novauos-builder .
docker run --rm --privileged -v "$PWD:/build" novauos-builder /build/build.sh

# Output: iso-build/novauos-<version>-amd64.hybrid.iso
```

CI builds run on every push to `main` and on every tagged release. Artifacts are uploaded to GitHub Actions + Releases.

## Download

Pre-built ISOs are published on the [Releases page](https://github.com/salom600/NovauOS/releases). Nightly builds are available as workflow artifacts.

## License

NovauOS is GPLv3-or-later for distribution-specific code; permissive (MIT/Apache-2.0) for individual Rust crates where appropriate. See [LICENSE](LICENSE).

## Status (2026 roadmap)

- [x] Repository bootstrap, CI/CD pipeline, build infrastructure
- [x] Rust component scaffolds (compilable, tested)
- [ ] First bootable ISO (Q1 2026)
- [ ] Hardware enablement matrix (Q2 2026)
- [ ] Public beta (Q3 2026)
- [ ] 1.0 release (Q4 2026)

---

*NovauOS — Rust from boot to desktop.*
