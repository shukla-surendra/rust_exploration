# 10. Cargo, Modules, Testing & Macros

**What this replaces:** `pip`/`venv`/`requirements.txt`/`setup.py`
(Python) or Maven/Gradle + `pom.xml`/`build.gradle` (Java), all rolled
into one tool. Modules replace Python's file-is-a-module/package-is-a-
directory convention and Java's `package`. Full depth on modules and
testing already exists — this chapter recaps fast and adds the pieces
that don't: Cargo.toml anatomy, attributes, and macros.

- [Crates & Modules](../foundation/crates-and-modules.md) — full depth
- [Testing](../foundation/testing.md) — full depth, tied to `rgrep`'s
  actual `#[cfg(test)] mod tests` blocks

## Cargo — build tool, package manager, test runner, all one command

| Command | Does | Nearest Python/Java equivalent |
|---|---|---|
| `cargo new foo` | scaffold a new project | `django-admin startproject` / a Maven archetype |
| `cargo build` | compile (debug profile) | `mvn compile` |
| `cargo build --release` | compile, optimized | `mvn package` (roughly) |
| `cargo run` | build + run | `python foo.py` |
| `cargo test` | run tests | `pytest` / `mvn test` |
| `cargo add foo` | add a dependency | `pip install foo` (but writes it to `Cargo.toml` too) |
| `cargo check` | type-check without producing a binary — much faster than `build` | closest to a Python linter/type-checker pass |

## `Cargo.toml` anatomy

```toml
[package]
name = "rgrep"
version = "0.1.0"
edition = "2024"

[dependencies]
regex = "1.10"          # from crates.io, semver-compatible with 1.10.x
serde = { version = "1", features = ["derive"] }   # with optional features enabled
```

Directly analogous to `pyproject.toml`/`package.json`/`pom.xml`:
metadata + a dependency list. `edition` (`2015`/`2018`/`2021`/`2024`) is
Rust-specific — it opts into a set of language changes without breaking
old code; unlike a language *version*, all editions are compiled by the
same compiler and can even be mixed across dependencies in one build.
`Cargo.lock` (analogous to `package-lock.json`/`poetry.lock`) pins exact
resolved versions for reproducible builds — commit it for binaries,
same convention as those ecosystems.

## Modules — organizing code within a crate

```rust
mod cli {              // inline module
    pub struct Config { /* ... */ }
}

// or, in a separate file src/cli.rs:
mod cli;                // declares "this module's code lives in cli.rs"
```

Closest Python analogy: a `mod` is like a Python module (a `.py` file),
except you have to explicitly declare `mod cli;` in a parent file to
"wire it in" — Rust doesn't auto-discover files the way Python's import
system walks a package directory. Everything is **private by default**
(unlike Python, where nothing is truly private, and unlike Java's
package-private default, which is one specific privacy level rather
than the strictest one) — `pub` opts a specific item out into visibility
from outside its module. `rgrep`'s own `lib.rs` (`pub mod cli;`) is a
live example of this wiring.

## Testing — the fast recap

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
```

`cargo test` discovers every `#[test]` function; a test passes by
returning normally and fails by panicking (same panic mechanism as
`todo!()`/`.unwrap()` from chapter 6). Full explanation, including why
tests conventionally live in the same file as the code they test, in
[Testing](../foundation/testing.md).

## Attributes — `#[...]`, metadata the compiler acts on

You've already used several: `#[derive(Debug)]` (chapter 3),
`#[cfg(test)]` (above), `#[test]`. The general shape is
`#[attribute_name(args)]` attached to whatever follows it (a function,
struct, module, or in the `#![...]` form, an entire file/crate).
Frequent ones:

| Attribute | Does |
|---|---|
| `#[derive(Trait1, Trait2, ...)]` | auto-generate trait impls — see [Traits](../foundation/traits.md) |
| `#[cfg(...)]` | conditional compilation — `#[cfg(test)]`, `#[cfg(target_os = "linux")]`, etc. |
| `#[allow(...)]` / `#[warn(...)]` / `#[deny(...)]` | tune compiler lint behavior for the item below |
| `#[test]` | mark a test function |

## Macros — code that generates code

`println!`, `vec!`, `assert_eq!`, `#[derive(...)]` are all macros, not
functions — that's what the `!` signals for the call-like ones. The
practical difference from a function: a macro runs *at compile time* and
can accept things a function can't (a variable number of arguments of
different types, as `println!("{} {}", a, b)` does; arbitrary syntax, as
`vec![1, 2, 3]`'s `[...]` shows). You'll write plenty of code *using*
macros; writing your own (`macro_rules! my_macro { ... }`, or a
procedural/derive macro) is a rare, advanced need — recognizing that
`!` means "this is a macro, not a function, and can do things a function
signature couldn't express" is the useful takeaway for a refresher pass.
