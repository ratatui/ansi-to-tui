# Contributing

Thanks for contributing!

## CI Overview

This repo has two CI workflows:

- `build.yml` (Cargo): contributor-friendly checks using standard Rust tooling.
- `build-nix.yml` (Nix): reproducible, flake-defined checks executed via `nix build`.

The Nix workflow generates its job matrix by evaluating `.#githubActions.matrix`, so the checks it
runs in CI are the same ones you can run locally with Nix.

### Why Nix?

Using Nix adds some complexity, but it also buys a few things that are otherwise hard to keep
reliable over time:

- A pinned Rust toolchain (aligned with the crate MSRV) and pinned tool versions via `flake.lock`.
- A single source of truth for the check list (the flake), used both locally and in CI.
- Reproducible builds across machines/CI runners, which helps avoid "works on my machine" issues.

The Cargo workflow exists so contributors can run the same checks without needing to learn Nix.

## Local Development (Cargo)

These commands match what `build.yml` runs:

```bash
taplo fmt --check
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features
cargo nextest run --no-default-features
cargo doc --no-deps
```

If you don't have `cargo-nextest` installed, you can use `cargo test` instead:

```bash
cargo test --all-features
cargo test --no-default-features
```

### Dependency and License Checks

If you have these tools installed locally, you can run:

```bash
cargo deny check
cargo audit
```

## Local Development (Nix)

If you use Nix, the flake exposes the same checks CI runs. Typical invocations:

```bash
nix build -L .#checks.x86_64-linux.ansi-to-tui-fmt
nix build -L .#checks.x86_64-linux.ansi-to-tui-clippy
nix build -L .#checks.x86_64-linux.ansi-to-tui-nextest
```

To see the matrix that CI evaluates:

```bash
nix eval --json .#githubActions.matrix
```
