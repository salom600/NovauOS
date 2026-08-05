# NovauOS

A modern, lightweight, Rust-native Linux distribution. Built on a hardened Debian 12 (bookworm) core.

## Quick links

- [Architecture](docs/ARCHITECTURE.md)
- [Build instructions](docs/BUILD.md)
- [Design philosophy](docs/DESIGN.md)
- [Releases](https://github.com/salom600/NovauOS/releases)
- [Issue tracker](https://github.com/salom600/NovauOS/issues)

## Build status

| Workflow | Status |
|----------|--------|
| Rust components | ![build-rust](https://github.com/salom600/NovauOS/actions/workflows/build-rust.yml/badge.svg) |
| ISO build | ![build-iso](https://github.com/salom600/NovauOS/actions/workflows/build-iso.yml/badge.svg) |
| Self-heal | ![self-heal](https://github.com/salom600/NovauOS/actions/workflows/self-heal.yml/badge.svg) |
| Release | ![release](https://github.com/salom600/NovauOS/actions/workflows/release.yml/badge.svg) |

## Quick start

```bash
git clone https://github.com/salom600/NovauOS.git
cd NovauOS/iso-build
docker build -t novauos-builder .
docker run --rm --privileged -v "$PWD:/build" novauos-builder
```

See [docs/BUILD.md](docs/BUILD.md) for full instructions.
