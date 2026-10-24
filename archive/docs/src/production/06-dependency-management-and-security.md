# 6. Dependency Management & Security

**Python/Java equivalent:** `pip-audit`/`safety` (Python), OWASP
dependency-check / Snyk (Java). Every `cargo add` pulls in a dependency
tree you didn't personally audit — the same supply-chain risk as `pip
install` or a Maven dependency, and it needs the same discipline.

## `cargo audit` — known-vulnerability scanning

```sh
cargo install cargo-audit
cargo audit
```

Checks every dependency in your `Cargo.lock` against the
[RustSec Advisory Database](https://rustsec.org) — the Rust ecosystem's
equivalent of the CVE database `pip-audit`/`npm audit` check against.
Run it in CI (see [CI/CD](./07-ci-cd.md)) so a newly-disclosed
vulnerability in a dependency fails the build instead of shipping
silently.

## `cargo deny` — policy enforcement beyond just CVEs

```sh
cargo install cargo-deny
cargo deny check
```

Broader than `cargo audit` — a `deny.toml` file can enforce:

- **License compliance** — reject dependencies under a license your
  project can't legally use (e.g. GPL in a closed-source product).
- **Banned crates** — block a specific crate/version outright (a known-
  bad release, an unmaintained fork you don't want creeping in
  transitively).
- **Duplicate versions** — flag when your dependency tree pulls in two
  different versions of the same crate (bloats the binary, sometimes
  signals a real problem).

```toml
# deny.toml
[licenses]
allow = ["MIT", "Apache-2.0"]

[bans]
deny = [{ name = "openssl" }]   # e.g. standardizing on rustls instead
```

## `Cargo.lock`: commit it for binaries, think twice for libraries

- **Binary crates / applications** (like `rgrep`) — **commit
  `Cargo.lock`**. It pins the exact resolved version of every
  dependency, so `cargo build` produces the same binary on every
  machine and in CI — the same reasoning as committing
  `package-lock.json`/`poetry.lock`, and why `rgrep`'s `Cargo.lock`
  already exists in this repo.
- **Library crates you publish** (something meant to be depended on by
  other projects) — conventionally **not** committed, so downstream
  consumers resolve dependency versions against *their own* project's
  constraints rather than inheriting yours. `Cargo.lock` is still
  generated and used locally for reproducible dev builds; it's a
  `.gitignore` entry only in the publish-a-library case, not the
  binary case.

## Keeping dependencies current

```sh
cargo install cargo-outdated
cargo outdated                # what's behind, and by how much
cargo update                  # bump within Cargo.toml's existing semver constraints
```

`cargo update` only moves within what your `Cargo.toml` version
requirements already allow (see semver ranges in
[Cargo, Modules, Testing & Macros](../workbook/10-cargo-modules-testing-macros.md#cargotoml-anatomy));
bumping a `Cargo.toml` version requirement itself (e.g. `"1"` → `"2"`)
is a manual, deliberate decision — Rust's semver conventions mean a
major bump can legitimately break your code, the same caution you'd
apply to a Python major-version pin bump.

## Minimizing the attack surface directly

- **Fewer dependencies is a security property, not just a build-time
  one** — every crate you add is code you didn't write running with the
  same privileges as your own. Before `cargo add`-ing something for one
  small utility function, check whether `std` or a dependency you
  already have covers it.
- **`unsafe` code, in your code or a dependency's**, deserves specific
  scrutiny — `cargo geiger` reports how much `unsafe` a dependency tree
  actually contains, which most crates keep near zero but a few
  (FFI-heavy ones especially) don't.
- **Pin exact versions for anything security-sensitive** (crypto,
  auth) rather than trusting a broad semver range to always resolve to
  something safe — the same instinct as pinning a critical Python
  package exactly rather than `>=`.
