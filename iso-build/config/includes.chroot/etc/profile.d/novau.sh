# ─── NovauOS environment defaults ──────────────────────────────────────
#
# Sourced by /etc/profile for all shells.

export NOVAU_VERSION="0.1.0"
export NOVAU_CODENAME="aurora"
export XDG_DATA_DIRS="/usr/local/share:/usr/share:/var/lib/flatpak/exports/share:${XDG_DATA_DIRS}"
export XDG_CONFIG_DIRS="/etc/xdg:${XDG_CONFIG_DIRS}"
export GTK_THEME="Adwaita:dark"
export QT_QPA_PLATFORMTHEME="gtk3"
export MOZ_ENABLE_WAYLAND=1
export SDL_VIDEODRIVER=wayland
export _JAVA_AWT_WM_NONREPARENTING=1

# Detect live mode: live-boot mounts the ISO at /run/live/medium (bookworm)
# or /cdrom (older). Either presence means we're on the live ISO.
if [ -d /run/live/medium ] || [ -d /cdrom ] || [ -e /run/live/medium/live/filesystem.squashfs ]; then
    export NOVAU_LIVE=1
fi
