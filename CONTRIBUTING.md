# Contributing to NovauOS

Thanks for your interest in improving NovauOS! This document explains
how to set up a development environment and what we expect from
contributions.

## Development setup

```bash
# Clone
git clone https://github.com/salom600/NovauOS.git
cd NovauOS

# Build Rust components
cd rust-components
cargo build --workspace
cargo test --workspace

# Run a component in isolation (e.g. the launcher, on your existing desktop)
cargo run --bin novau-launcher
```

For ISO builds, see [docs/BUILD.md](docs/BUILD.md).

## Code style

- **Rust:** `cargo fmt` is enforced. `cargo clippy -- -D warnings` is
  expected to pass (we tolerate `clippy::needless_range_loop` and a
  handful of others in CI for now; see `.github/workflows/build-rust.yml`).
- **Shell:** `shellcheck` is enforced for any hook or build script.
- **Markdown:** 100-col soft wrap, no hard line breaks in paragraphs.

## Commit message format

We follow a simplified Conventional Commits:

```
<type>(<scope>): <subject>

<body>
```

`<type>` is one of:
- `feat` — a new feature
- `fix` — a bug fix
- `docs` — documentation only
- `build` — build system, CI, packaging
- `refactor` — code restructure without behavior change
- `test` — tests
- `chore` — misc

`<scope>` is the component: `greeter`, `panel`, `launcher`, `store`,
`installer`, `settings`, `welcome`, `iso`, `ci`, `docs`.

Example:

```
feat(greeter): add face image support for users

Reads /var/lib/AccountsService/icons/<user>.png and falls back to a
default. Matches gdm3 behavior so users migrating from GNOME keep
their avatars.
```

## Pull request process

1. Open an issue first if your change is non-trivial. A 30-second
   "is this wanted?" saves hours of wasted work.
2. Branch from `main`. Keep PRs small (≤400 LOC) where possible.
3. Make sure CI passes locally:
   ```bash
   cd rust-components
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
4. Update the changelog if your change is user-visible.
5. Squash-merge on approval. Use the PR title as the commit message.

## CI behavior

Our CI is self-healing:

- **Transient failures** (network blips, registry 5xx, apt hash
  mismatches) are auto-retried up to 3 times with exponential backoff.
- **Real failures** open a GitHub issue automatically with the relevant
  log excerpt. A maintainer will triage.

If you see a CI failure with the `ci-failure` label, it's an automated
report — please investigate, fix, and close.

## Adding a new component

If you're adding a new `novau-*` component:

1. Create `rust-components/novau-<name>/` with `Cargo.toml` and `src/main.rs`.
2. Add it to the workspace `members` in `rust-components/Cargo.toml`.
3. Add a systemd unit file in
   `iso-build/config/hooks/normal/02-novau-systemd-units.hook.chroot`.
4. If the component has binaries that need to be on PATH, the
   `iso-build/build.sh` script's binary-staging loop already copies
   every `novau-*` crate — just add the name there.
5. Update `docs/ARCHITECTURE.md` to describe the new component.

## License

By contributing, you agree that your contributions are licensed under
the same terms as NovauOS (GPLv3-or-later for distribution code,
MIT OR Apache-2.0 for individual Rust crates).
