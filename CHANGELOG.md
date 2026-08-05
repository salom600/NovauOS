# Changelog

All notable changes to NovauOS are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project repository bootstrap
- Rust workspace with 7 components: greeter, panel, launcher, store, installer, settings, welcome
- shared `novau-common` crate (paths, logging, types)
- shared `novau-ipc` crate (D-Bus + Unix socket contracts)
- Debian 12 (bookworm) live-build configuration
- Docker-based reproducible builder image
- 6 chroot hooks: system config, systemd units, Sway config, GPU drivers, Plymouth theme, GRUB menu
- GitHub Actions: `build-rust`, `build-iso`, `release`, `self-heal`
- Failure classification script for CI self-healing
- Documentation: README, ARCHITECTURE, BUILD, DESIGN, CONTRIBUTING

### Changed
- (none yet)

### Removed
- (none yet)

## [0.1.0] — Unreleased

Initial public skeleton. Not yet bootable. The Rust components compile
and run on an existing Debian 12 desktop for development purposes.
