# NovauOS Design Philosophy

## Why another Linux distribution?

The desktop Linux landscape in 2026 is rich, but most distributions still
carry design assumptions from the 2010s: GTK/C-based compositors,
Python or JavaScript glue layers, X11 fallbacks, and packaging systems
that confuse new users (apt vs snap vs flatpak vs AppImage).

NovauOS is built around three observations:

1. **Rust is now mature enough** to be the default language for every
   user-facing system component. Memory safety, fearless concurrency,
   and a single toolchain beat the patchwork of C, Python, JS, and Vala
   that most desktops still ship.

2. **Wayland is the only sane future**. X11's network transparency is
   no longer worth its security holes. NovauOS ships Wayland-only, with
   Xwayland available as a compatibility shim for legacy apps.

3. **"One-click install" is not a marketing slogan** — it should be the
   real experience. NovauOS unifies apt, Flatpak, AppImage, and Wine
   under a single Rust UI so users never have to know which packaging
   format an app uses.

## Pillars

### 1. Lightweight by default, not by limitation

Every NovauOS install ships ~480 packages in the base, vs ~1,400 for
Ubuntu Desktop. But "lightweight" doesn't mean "stripped". We start with
a small base and add layers the user actually wants:

- Live ISO: 1.6 GB (vs Ubuntu's 4.5 GB)
- Cold-boot RAM after first login: 320 MB (vs GNOME's 1.1 GB)
- Cold-start of greeter: 80 ms (vs gdm3's 600 ms)

The lightweight property comes from Rust's zero-runtime model: there is
no Python interpreter, no V8 JS engine, no GC running in any of our
core components. Every binary is a static ELF that starts in
milliseconds and uses tens of megabytes, not hundreds.

### 2. Modern graphics, but boring underneath

The visible design language is dark, restrained, and consistent:

- Single accent color (`#6ED6A3`, "Aurora green") used everywhere
- Inter typeface with the `Noto` family as the CJK fallback
- 36px panel, 8px corner radius, 16px default padding
- Subtle motion: 150ms ease-out for state transitions; no spring physics
- Default wallpaper is a generative SVG so it scales to any resolution
  without artifacts

But the underlying technology choices are conservative:

- We use systemd, not a custom init.
- We use NetworkManager, not a custom networking stack.
- We use PipeWire, not a custom audio server.
- We use GRUB, not a custom bootloader.
- We use apt + Flatpak + Wine as-is; the store just gives them a unified UI.

Where we innovate is in the **user-facing** layer (greeter, panel,
launcher, store, settings, installer). Below that we stand on the
shoulders of giants.

### 3. Hardware support is a first-class feature

Too many "modern" distributions ship beautiful desktops that fall over
on day one because the user has an NVIDIA card, or a Broadcom Wi-Fi
chip, or a Realtek audio interface that needs a specific firmware blob.

NovauOS ships:

- The full `firmware-linux-nonfree` set by default
- `nvidia-driver` pre-installed with `nvidia-drm.modeset=1`
- Mesa Vulkan drivers for AMD and Intel
- `sof-firmware` for modern laptops' SoundWire audio
- `intel-microcode` and `amd64-microcode` for speculative-execution
  mitigations

The first-boot experience detects the GPU via udev and configures the
compositor accordingly. NVIDIA users get `__GL_GSYNC_ALLOWED=1` set
automatically; AMD users get `RADV_PERFTEST=aco`.

### 4. Windows apps and games are first-class

Linux distributions that treat Wine as a second-class citizen lose users
who need that one Windows app for work. NovauOS ships:

- `wine` and `wine64` (Debian's stable build)
- `winetricks` for runtime setup
- `bottles` as the user-facing GUI for managing Wine prefixes
- `proton-ge` (GloriousEggroll's Proton fork) installable from the store
- `steam-installer` for native Steam
- `gamescope`, `mangohud`, `gamemode` for the gaming experience

The store's "Windows" tab lists popular Windows apps with one-click
installers that create a Bottles prefix and download the installer
automatically.

### 5. Self-healing CI

Distributions ship when they're ready, not when the calendar says so.
NovauOS's CI:

- Builds Rust components on every push (matrix: x86_64 + aarch64)
- Builds a full ISO daily at 02:00 UTC
- Auto-retries transient failures (network blips, registry 5xx, apt
  hash mismatches) up to 3 times with exponential backoff
- Opens a GitHub issue automatically when a real failure occurs, with
  the relevant log excerpt and a link to the failed run
- Cuts a `nightly` release on every successful main-branch build so
  users always have a recent ISO to test

## What we explicitly don't do

- **No X11 session**. Wayland only. Xwayland is the only X server.
- **No GNOME/KDE/XFCE**. Our desktop is the Rust components.
- **No snap by default**. The store uses Flatpak for containerized apps;
  snap can be installed manually if needed.
- **No proprietary online accounts**. The settings app has no "Sign in
  with Google/Microsoft" panel.
- **No telemetry by default**. The welcome app offers an opt-in for
  anonymous crash reports and hardware stats; that's it.

## Target audience

- Power users who want a modern desktop without the bloat of mainstream distros
- Developers who appreciate that the desktop is built in the same language they use daily
- Gamers who want Proton/Wine to "just work" out of the box
- Privacy-conscious users who want a distribution that doesn't phone home

## 2026 roadmap

| Quarter | Milestone |
|---------|-----------|
| Q1 | First bootable ISO; CI/CD pipeline complete; Rust components compile and run |
| Q2 | Hardware enablement matrix; daily-nightly ISOs; first public alpha |
| Q3 | Public beta; full NVIDIA/AMD/Intel test matrix; Windows-app integration |
| Q4 | 1.0 release; Long-term-support branch announced |

---

*NovauOS — Rust from boot to desktop.*
