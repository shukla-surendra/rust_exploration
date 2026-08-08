# 7. CI/CD

**Python/Java equivalent:** GitHub Actions/Jenkins/CircleCI running
`pytest`/`mvn test` on every push — same tools, same role, just
different commands in the pipeline steps. If you already have CI set up
for a Python or Java project, this chapter is "same shape, Rust
commands."

## What a Rust CI pipeline actually needs to run

Every chapter so far maps to one CI step:

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Format check
        run: cargo fmt -- --check          # ch. 1

      - name: Lint
        run: cargo clippy -- -D warnings   # ch. 1

      - name: Test
        run: cargo test                     # ch. 2

      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit                       # ch. 6
```

Run this against `use_cases/rgrep` (or any crate in this repo) with a
`working-directory: use_cases/rgrep` step option, or a matrix if you
want CI to cover every crate in the repo at once.

## Caching — the difference between a 30-second and 5-minute CI run

Rust compiles from source every time by default — without caching,
every CI run rebuilds every dependency from scratch. `Swatinem/rust-cache`
(a community GitHub Action) caches `~/.cargo` and `target/` keyed on
`Cargo.lock`, so unchanged dependencies aren't recompiled on every push:

```yaml
      - uses: Swatinem/rust-cache@v2
```

Add this once, right after checking out the code — it's close to a
required step for any Rust CI pipeline beyond a toy project, the same
way caching `node_modules`/`.venv` is standard practice in JS/Python CI.

## Matrix builds — testing across targets

```yaml
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    runs-on: ${{ matrix.os }}
```

Worth doing once a crate is meant to run on more than one platform —
catches platform-specific bugs (path separators, case-sensitivity
assumptions) the same class of issue a Python project's CI matrix
across Python versions/OSes is meant to catch.

## Release automation

A tagged release can trigger building and publishing artifacts
automatically — the Rust-specific pieces:

```yaml
on:
  push:
    tags: ['v*']

jobs:
  release:
    steps:
      - run: cargo build --release   # ch. 8
      # then upload target/release/<binary> as a GitHub Release asset,
      # or `cargo publish` if this is a library crate (ch. 10)
```

## The minimum viable pipeline for a personal project

If the full pipeline above feels like too much to start: `cargo fmt --
--check`, `cargo clippy -- -D warnings`, and `cargo test` on every push
is a strong baseline on its own — the same three checks you'd run
locally before committing, just enforced so they can't be skipped, and
visible to anyone (including future you) looking at the repo's CI
status badge.
