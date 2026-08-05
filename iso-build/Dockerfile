# ─── NovauOS ISO builder ────────────────────────────────────────────────
# Reproducible build environment based on debian:bookworm-slim with
# live-build, debian-installer, and the Rust toolchain pre-installed.

FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

# Core build tools
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        xz-utils \
        sudo \
        debootstrap \
        live-build \
        systemd-container \
        squashfs-tools \
        xorriso \
        mtools \
        dosfstools \
        grub-pc-bin \
        grub-efi-amd64-bin \
        shim-signed \
        mtools \
        parted \
        rsync \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        libwayland-dev \
        libxkbcommon-dev \
        libdbus-1-dev \
        libpam0g-dev \
        libudev-dev \
        libsqlite3-dev \
        libgtk-3-dev \
        libpulse-dev \
        fonts-noto-core \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain (pinned via rust-toolchain.toml in the workspace)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --default-toolchain 1.86.0 --profile minimal && \
    /root/.cargo/bin/rustup component add rustfmt clippy && \
    /root/.cargo/bin/rustup target add x86_64-unknown-linux-gnu

ENV PATH="/root/.cargo/bin:${PATH}"

# Working directory
WORKDIR /build

# Default command: run the orchestrator
CMD ["/bin/bash", "/build/build.sh"]
