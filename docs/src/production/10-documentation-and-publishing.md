# 10. Documentation & Publishing

**Python/Java equivalent:** docstrings + Sphinx + PyPI (Python), Javadoc
+ Maven Central (Java). Rust's version (`rustdoc` + crates.io) is more
tightly integrated than either — doc comments are checked by the
compiler (see doc tests in
[Testing Strategy](./02-testing-strategy.md)), and publishing is a
single `cargo` subcommand.

## Doc comments — `///` and `//!`

```rust
/// Finds every line in `contents` containing `pattern`.
///
/// # Examples
///
/// ```
/// let matches = rgrep::search::search("error", "ok\nan error here");
/// assert_eq!(matches, vec!["an error here"]);
/// ```
pub fn search<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    // ...
}
```

`///` documents the item immediately below it (a function, struct,
enum, module) — this is what `cargo doc` renders and what shows up on
[docs.rs](https://docs.rs) for a published crate. `//!` (note the `!`)
documents the *enclosing* item instead — conventionally used at the top
of `lib.rs`/a module file to describe the module/crate as a whole:

```rust
//! `rgrep` is a minimal grep clone for searching log files by pattern.
//!
//! See [`search`] for the core matching logic and [`cli`] for argument
//! parsing.
```

`[`search`]` inside a doc comment is an **intra-doc link** — `rustdoc`
resolves it to the actual `search` function's generated page
automatically, and errors at doc-build time if the reference is stale
(renamed/removed item) — the same "can't silently rot" property doc
tests give your *examples*, applied to your *cross-references* instead.

## Generating and viewing docs locally

```sh
cargo doc --open --no-deps
```

`--no-deps` renders only your own crate's docs, not every dependency's
too (much faster, and what you actually want while iterating).
Structurally the same output shape as Javadoc's generated HTML or
Sphinx's — navigable by module/type/function, with search.

## Conventional sections inside a doc comment

```rust
/// Reads `path` and returns its contents as a `String`.
///
/// # Errors
///
/// Returns [`RgrepError::Io`] if the file doesn't exist or can't be read.
///
/// # Panics
///
/// Panics if `path` contains invalid UTF-8 in its... (only if genuinely true — omit otherwise)
///
/// # Examples
///
/// ```
/// # use rgrep::read_file;
/// let contents = read_file("sample_logs/app.log")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

`# Errors` and `# Panics` are convention, not compiler-enforced, but
followed widely enough across the ecosystem (`clippy` even has a lint,
`missing_errors_doc`, nudging you toward writing them) that omitting
them on a public API reads as incomplete documentation to anyone
familiar with Rust crates.

## `cargo publish` — shipping a library crate

```sh
cargo login                 # one-time, using a crates.io API token
cargo publish --dry-run     # verify without actually publishing
cargo publish
```

The direct equivalent of `twine upload` (PyPI) or `mvn deploy` (Maven
Central) — except crates.io publishes are **immutable**: once a version
is published, it cannot be changed or deleted (only "yanked," which
prevents *new* projects from picking it up while leaving existing users
unaffected). This makes `--dry-run` and a solid `Cargo.toml` (license,
description, repository link — all required or strongly expected
metadata) worth double-checking before the real publish, since there's
no fixing a typo in-place afterward the way you might force-overwrite a
mistaken PyPI upload.

## Versioning — semver, and Rust's specific enforcement of it

```toml
[package]
version = "1.2.3"   # MAJOR.MINOR.PATCH
```

Same semver rules as npm/PyPI conventionally follow: breaking change →
bump MAJOR, new backward-compatible feature → bump MINOR, bug fix →
bump PATCH. Rust leans on this more heavily than most ecosystems because
`Cargo.toml` dependency ranges (`"1.2"` means "`>=1.2.0, <2.0.0`") are
*trusted* to hold — the tooling assumes any `1.x` release won't break
you, which is why `cargo-semver-checks` exists specifically to catch
accidental breaking changes before a publish that claims to be a mere
MINOR/PATCH bump.

## Workspaces — multiple crates sharing one `Cargo.lock`

```toml
# Cargo.toml at the repo root
[workspace]
members = ["core", "cli", "server"]
```

Once a project splits into multiple crates that depend on each other
(a shared `core` library, a `cli` binary, a `server` binary — the same
shape as `rgrep`'s `lib.rs` + `main.rs`, scaled up to fully separate
crates), a workspace ties them together: one shared `Cargo.lock`, one
`target/` directory, `cargo build`/`cargo test` at the root runs across
all members. Closest analog: a monorepo with a single shared dependency
lockfile, the same organizational choice a large Python monorepo makes
with a single `poetry.lock`/`uv.lock` at the root instead of one per
service.
