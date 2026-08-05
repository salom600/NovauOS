# NovauOS Architecture

## Design Principles

1. **Rust where it matters** — every interactive surface is Rust. System utilities that are already excellent (systemd, NetworkManager, Mesa) are kept as-is; we do not rewrite for the sake of rewriting.
2. **Wayland-first, no X11** — removes a 40-year-old attack surface. Xwayland is included for legacy apps only.
3. **Modular, not monolithic** — each `novau-*` component is a separate Cargo crate with a stable IPC contract (Wayland + D-Bus). Any component can be swapped without touching the others.
4. **Reproducible builds** — ISO builds happen in Docker on Debian bookworm-slim, pinned by `Cargo.lock` and apt snapshot timestamps.
5. **Self-healing CI** — transient build failures trigger an automatic retry with a backoff. Real failures open an issue automatically.

## Layered architecture

```
┌──────────────────────────────────────────────────────────┐
│  User surfaces (all Rust)                                │
│  novau-greeter  novau-panel  novau-launcher              │
│  novau-store     novau-settings  novau-welcome           │
│  novau-installer                                          │
├──────────────────────────────────────────────────────────┤
│  Wayland compositor layer                                │
│  cosmic-comp / sway (selected at build time)             │
│  Xwayland (optional, for legacy games)                   │
├──────────────────────────────────────────────────────────┤
│  Session & IPC                                            │
│  systemd-logind · dbus · PipeWire ·xdg-desktop-portal    │
├──────────────────────────────────────────────────────────┤
│  Hardware enablement                                      │
│  Mesa · NVIDIA · AMD ROCm · Intel compute · PipeWire     │
│  novau-hardware (udev rules + driver auto-selection)     │
├──────────────────────────────────────────────────────────┤
│  Base system                                              │
│  Debian 12 (bookworm) — minimal ~480 packages             │
│  linux-image-amd64 · systemd · apt · live-build          │
├──────────────────────────────────────────────────────────┤
│  Bootloader                                               │
│  GRUB EFI + novau-plymouth-theme + initramfs-tools       │
└──────────────────────────────────────────────────────────┘
```

## Rust components

### novau-greeter
- Replaces `gdm3` / `sddm` / `lightdm`.
- Pure Wayland client (no X11 dependency).
- Built with `winit` + `iced` for the UI.
- Speaks `pam` directly via `pam-client` crate.
- Cold start: ~80ms on a SSD.
- Lists available sessions (Novau, sway, GNOME-classic fallback) and users from `/etc/passwd` + `accountsservice`.

### novau-panel
- Top panel: clock, battery, network, volume, notifications, tray.
- Built on `iced` + `layer-shell` Wayland protocol.
- Notifications via the FreeDesktop Notification API.
- Extensible via Lua plug-ins (single-file, sandboxed).

### novau-launcher
- Rofi/dmenu replacement.
- Fuzzy search over `.desktop` files.
- Calculation mode (type `=12*4`).
- SSH host mode (parses `~/.ssh/config`).

### novau-store
- One-click installer for: native apt packages, Flatpaks (Flathub), Wine runners (via `proton-ge`), AppImages.
- Built on `iced` + `rusqlite` (package cache).
- Sandboxed: store UI runs as user, install actions go through a polkit-authorized helper.

### novau-installer
- Calamares alternative.
- Single-binary Rust installer with steps: locale → disk → user → summary → install.
- Uses `parted` + `mkfs.*` via subprocess; everything else is native Rust.
- Supports: Btrfs + subvolumes, LUKS, dual-boot with Windows (preserves ESP).

### novau-settings
- Modular: Display, Sound, Network, Power, Users, About.
- Backed by `async-dbus` calls to existing services (BlueZ, NetworkManager, UPower).
- Settings stored in `~/.config/novau/` (RON format, human-readable).

### novau-welcome
- First-boot onboarding: language, timezone, account picture, optional telemetry opt-in.
- Live mode: prominent "Install NovauOS" button.

## IPC contracts

All components communicate via:

1. **D-Bus** — system-level signals (network, power, mounts).
2. **Wayland protocols** — `layer-shell` for panel & launcher; `xdg-shell` for windows; `wlr-foreign-toplevel` for panel window list.
3. **Unix sockets** — `novau-*` private IPC (e.g., greeter → panel "user logged in").

## Boot sequence

```
UEFI → GRUB → linux + initramfs → systemd
  → graphical.target
    → novau-greeter.service (PAM)
      → user logs in
      → novau-session.service (user unit)
        → compositor (sway / cosmic-comp)
        → novau-panel, novau-settings-daemon
        → novau-welcome (first boot only)
```

## CI/CD pipeline

| Workflow                | Trigger                       | Purpose                                     |
|-------------------------|-------------------------------|---------------------------------------------|
| `build-rust.yml`        | push to `main`, PR            | Build all Rust crates on Linux x86_64 + aarch64 |
| `build-iso.yml`         | push to `main`, daily 02:00 UTC | Build full ISO via Docker + live-build     |
| `release.yml`           | tag `v*`                      | Cut GitHub Release with signed ISO          |
| `self-heal.yml`         | workflow_run (on failure)     | Auto-retry up to 3× with backoff; if still failing, opens an issue |

All workflows use the built-in `GITHUB_TOKEN` for repo-scoped operations. No PATs are stored in code.

## Hardware support matrix

| Vendor    | Driver           | Out-of-box | Notes                              |
|-----------|------------------|------------|-------------------------------------|
| Intel iGPU| `mesa`           | ✓          | Iris/Xe via `iris`/`i915`           |
| AMD iGPU  | `mesa`           | ✓          | radeonsi + RADV                     |
| AMD dGPU  | `mesa` + ROCm    | ✓          | compute optional                    |
| NVIDIA    | `nvidia-driver`  | ✓ (auto)   | Wayland via `nvidia-drm.modeset=1`  |
| Wi-Fi     | `linux-firmware` | ✓          | all blobs shipped                   |
| Audio     | `pipewire` + ALSA| ✓          |                                     |

## Security model

- No `setuid` binaries shipped (everything uses polkit + capabilities).
- `/home` default `fscrypt` or LUKS-per-home optional.
- AppArmor profiles for `novau-store`, `novau-installer`.
- All Rust crates audited via `cargo-audit` in CI; denied on RUSTSEC advisories.
