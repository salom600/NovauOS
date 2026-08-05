# Building NovauOS Locally

This document explains how to build the NovauOS ISO on your own machine.
The CI does the same thing inside GitHub Actions; replicating it locally
is useful for development and offline testing.

## Prerequisites

- Docker 24+ (or Podman 4+ with Docker compat layer)
- ~10 GB free disk space
- A fast network (the first build downloads ~3 GB of Debian packages)

## Quick start

```bash
git clone https://github.com/salom600/NovauOS.git
cd NovauOS/iso-build

# Build the builder image (cached after first run)
docker build -t novauos-builder .

# Run the build (privileged is required so live-build can chroot)
docker run --rm --privileged \
    -v "$PWD:/build" \
    -e NOVAU_VERSION=0.1.0 \
    novauos-builder

# Output:
#   novauos-0.1.0-amd64.hybrid.iso
#   novauos-0.1.0-amd64.hybrid.iso.sha256sum
```

## What the build does

1. **Compiles the Rust components** (greeter, panel, launcher, store, installer, settings, welcome) inside the container using the pinned 1.83.0 toolchain.
2. **Copies the release binaries** into `iso-build/binaries/`.
3. **Runs `lb config`** with our overrides (bookworm, hybrid ISO, GRUB for both BIOS + UEFI).
4. **Overlays** `iso-build/config/` into the live-build tree:
   - `package-lists/` — what apt installs into the squashfs
   - `hooks/normal/*.hook.chroot` — runs in-chroot: writes `/etc/os-release`, PAM service, systemd units, Sway config, GPU drivers, Plymouth theme
   - `hooks/normal/*.hook.binary` — runs on the final ISO tree: GRUB menu customization
   - `includes.chroot/` — files dropped verbatim into the squashfs
5. **Runs `lb build`** which:
   - Bootstraps a minimal Debian rootfs via debootstrap
   - Installs all packages from our lists
   - Runs the chroot hooks
   - Compresses the rootfs as squashfs
   - Builds the GRUB EFI + BIOS images
   - Stitches everything into a hybrid ISO

## Booting the ISO

```bash
# Verify
sha256sum -c novauos-0.1.0-amd64.hybrid.iso.sha256sum

# Flash to USB (replace /dev/sdX with your device — double-check!)
sudo dd if=novauos-0.1.0-amd64.hybrid.iso \
        of=/dev/sdX bs=4M conv=fsync status=progress
sync

# Or run in QEMU for testing
qemu-system-x86_64 -m 4G -enable-kvm \
    -cdrom novauos-0.1.0-amd64.hybrid.iso \
    -boot d -display gtk
```

## Iterating on components

For faster iteration you don't need to rebuild the whole ISO each time.
Build the Rust components locally:

```bash
cd rust-components
cargo build --release --workspace
```

Then `scp` the resulting binaries onto a running NovauOS VM, replacing
`/usr/bin/novau-*`, and restart the relevant systemd unit:

```bash
# On the VM, after replacing the binary:
sudo systemctl restart novau-greeter
# or, for user units:
systemctl --user restart novau-panel
```

## Customizing

- **Add a package:** append its name to `iso-build/config/package-lists/novau-base.list.chroot`.
- **Remove a package:** add `-name` to `iso-build/config/package-lists/novau-removals.list.chroot`.
- **Run a setup step in the chroot:** add a new `iso-build/config/hooks/normal/NN-*.hook.chroot`.
- **Add a file to the squashfs:** drop it under `iso-build/config/includes.chroot/<path>`.
- **Change the GRUB menu:** edit `iso-build/config/hooks/normal/06-novau-grub.hook.binary`.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `lb build` fails with "debootstrap failed to download" | Network issue; rerun. |
| `Could not compile` in Rust crates | `cargo clean` and rebuild; check you're on 1.83.0. |
| ISO boots to GRUB but kernel panics | Check `boot=live` is in the kernel cmdline; verify the squashfs is in `/live/`. |
| NVIDIA driver doesn't load | Confirm `nvidia-drm.modeset=1` is set; check `dmesg \| grep -i nvidia`. |
| Wayland session doesn't start | Verify `novau-greeter.service` started: `systemctl status novau-greeter`. |
| Disk full during build | `docker system prune -a`; build needs ~10 GB. |

## Build artifacts

After a successful build:

```
iso-build/
├── novauos-0.1.0-amd64.hybrid.iso        ← the ISO (~2 GB)
├── novauos-0.1.0-amd64.hybrid.iso.sha256sum
├── build.log                              ← full log
└── .build/                                ← live-build working dir
```
