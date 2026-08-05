#!/usr/bin/env bash
# ─── NovauOS ISO builder ──────────────────────────────────────────────
#
# Runs inside the novauos-builder Docker container (or locally if all
# deps are present). Produces:
#
#   novauos-<version>-amd64.hybrid.iso
#
# Usage:
#   docker run --rm --privileged -v "$PWD:/build" novauos-builder
#
set -euo pipefail

# Locate the workspace root (the dir containing rust-components/).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORK_DIR="${SCRIPT_DIR}/.build"
VERSION="${NOVAU_VERSION:-0.1.0}"
ARCH="${NOVAU_ARCH:-amd64}"
ISO_NAME="novauos-${VERSION}-${ARCH}.hybrid.iso"

log()  { printf '\033[1;36m[build]\033[0m %s\n' "$*"; }
err()  { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; }
die()  { err "$*"; exit 1; }

command -v lb >/dev/null 2>&1 || die "live-build (lb) not found. Run inside the builder Docker image."
[ -d "${ROOT_DIR}/rust-components" ] || die "rust-components/ not found at ${ROOT_DIR}"

# ─── 1. Build Rust components ────────────────────────────────────────────
log "Building Rust components…"
pushd "${ROOT_DIR}/rust-components" >/dev/null
cargo build --release --workspace
popd >/dev/null

# Stash built binaries where the chroot hook can find them
BIN_STAGING="${SCRIPT_DIR}/binaries"
rm -rf "${BIN_STAGING}"
mkdir -p "${BIN_STAGING}"
for crate in novau-greeter novau-panel novau-launcher novau-store novau-installer novau-settings novau-welcome; do
    src="${ROOT_DIR}/rust-components/target/release/${crate}"
    [ -f "${src}" ] || die "missing binary: ${src}"
    cp "${src}" "${BIN_STAGING}/${crate}"
    log "  ✓ ${crate}"
done

# ─── 2. Prepare live-build workspace ────────────────────────────────────
log "Preparing live-build workspace at ${WORK_DIR}…"
rm -rf "${WORK_DIR}"
mkdir -p "${WORK_DIR}"
cd "${WORK_DIR}"

# Initialise live-build config
lb config \
    --distribution bookworm \
    --architecture "${ARCH}" \
    --archive-areas "main contrib non-free non-free-firmware" \
    --parent-archive-areas "main contrib non-free non-free-firmware" \
    --debian-installer none \
    --iso-volume "NOVAUOS ${VERSION}" \
    --iso-publisher "NovauOS Project; https://github.com/salom600/NovauOS" \
    --iso-application "NovauOS ${VERSION}" \
    --image-name "novauos-${VERSION}-${ARCH}" \
    --memtest none \
    --uefi-security-boot disabled \
    --bootloaders grub-pc,grub-efi \
    --binary-images iso-hybrid \
    --chroot-filesystem squashfs \
    --compression xz

# ─── 3. Copy our config overlays ────────────────────────────────────────
log "Overlaying NovauOS live-build configuration…"
cp -r "${SCRIPT_DIR}/config/." config/

# Copy built binaries into includes.chroot
for crate in novau-greeter novau-panel novau-launcher novau-store novau-installer novau-settings novau-welcome; do
    install -D -m 0755 "${BIN_STAGING}/${crate}" \
        "config/includes.chroot/usr/bin/${crate}"
done

# Copy the Rust source so hooks that need to rebuild in-chroot can do so.
# (We don't actually need this for binary-only installs, but it's useful
#  for hooks that want to rebuild against the chroot's libraries.)
mkdir -p config/includes.chroot/usr/src/novauos
rsync -a --exclude target --exclude .git \
    "${ROOT_DIR}/rust-components/" \
    config/includes.chroot/usr/src/novauos/rust-components/
rsync -a --exclude .git \
    "${ROOT_DIR}/iso-build/" \
    config/includes.chroot/usr/src/novauos/iso-build/

# ─── 4. Run live-build ──────────────────────────────────────────────────
log "Running live-build (this takes ~30–60 min on first run)…"
lb build 2>&1 | tee "${SCRIPT_DIR}/build.log"

# ─── 5. Locate & verify the ISO ─────────────────────────────────────────
ISO_PATH="${WORK_DIR}/novauos-${VERSION}-${ARCH}.hybrid.iso"
[ -f "${ISO_PATH}" ] || die "ISO not produced at ${ISO_PATH}"

# Move to script dir so docker -v mount sees it
mv "${ISO_PATH}" "${SCRIPT_DIR}/${ISO_NAME}"
log "✓ ISO ready: ${SCRIPT_DIR}/${ISO_NAME}"

# Optional: produce a sha256sum
( cd "${SCRIPT_DIR}" && sha256sum "${ISO_NAME}" > "${ISO_NAME}.sha256sum" )
log "✓ sha256: ${SCRIPT_DIR}/${ISO_NAME}.sha256sum"

log "Done."
