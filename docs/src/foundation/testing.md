# Rust's built-in test framework

> **Coming from Python/Java:** `#[test]` + `cargo test` is `pytest`
> (Python) or JUnit's `@Test` + a build-tool `test` goal (Java), except
> built directly into the toolchain — no separate install, no test
> discovery configuration. `assert_eq!` is `assertEqual`/`assert ==`.
> The one habit worth noting: Rust tests conventionally live in the
> *same file* as the code they test (`mod tests` at the bottom), which
> looks unusual coming from Python/Java's "tests live in a mirrored
> `tests/` directory" convention — see the last section below for why,
> and when a separate directory is still the right call.
>
> **Practical payoff:** the `rgrep` scaffold's tests aren't decoration —
> they're your progress bar. `cargo test` failing at a `todo!()` tells
> you exactly which function still needs real logic, the same way a red
> `pytest`/JUnit run points at what's broken, except here it's "not
> written yet" instead of "written wrong."

No external test runner or assertion library needed — `cargo test`,
`#[test]`, and a handful of `assert!` macros are all part of `std` and
the toolchain. This is exactly what's already sitting in
[`rgrep/src/search.rs`](../../../use_cases/rgrep/src/search.rs) and
[`rgrep/src/cli.rs`](../../../use_cases/rgrep/src/cli.rs):

```rust
pub fn search<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    todo!("find every line in `contents` containing `pattern`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_matching_lines() {
        let contents = "first line\nsecond line with error";
        let results = search("error", contents);
        assert_eq!(results, vec!["second line with error"]);
    }
}
```

## The four pieces

- **`#[cfg(test)]`** — a *conditional compilation* attribute. This module
  is only compiled when running `cargo test`; it's entirely absent from
  a normal `cargo build`/`cargo run` binary, so test code never bloats
  or slows down the real program.
- **`mod tests { ... }`** — an ordinary inline module (see
  [Crates & Modules](./crates-and-modules.md)), just conventionally named
  `tests` and placed at the bottom of the file it's testing.
- **`use super::*;`** — imports everything from the parent module
  (`search`, and anything else defined above in the same file) into
  `tests`' scope, so the test bodies can call `search(...)` directly
  instead of spelling out `super::search(...)`.
- **`#[test]`** — marks a function as a test case. `cargo test` discovers
  every `#[test]`-annotated function in the crate, runs each one, and
  reports pass/fail based on whether it panics.

## How pass/fail actually works

A test doesn't "return" pass or fail — it **passes by returning
normally** and **fails by panicking**. That's why the assertion macros
(`assert!`, `assert_eq!`, `assert_ne!`) are just thin wrappers around
`panic!`:

```rust
assert_eq!(results, vec!["second line with error"]);
// roughly: if results != vec![...] { panic!("assertion failed: ...") }
```

This is the same panic mechanism covered in
[Error Handling in main.rs](./error-handling.md#a-related-but-different-failure-mode-todo-unimplemented-panic)
for `todo!()`/`unimplemented!()` — a test that hits an unfilled
`todo!()` fails for exactly the same reason a failed `assert_eq!` does:
both panic, and `cargo test` treats any panic during a `#[test]`
function as a failure.

## Running tests

```sh
cargo test              # every #[test] in the crate
cargo test search       # only tests whose name contains "search"
cargo test -- --nocapture   # also show println! output from passing tests
```

`cargo test`'s output tells you exactly which function failed and, for
`assert_eq!`, prints both sides of the comparison — which is what makes
the failing tests in the `rgrep` scaffold useful as a todo list: each
one names the function whose `todo!()` you haven't replaced yet.

## Why tests live in the same file as the code

Keeping `mod tests` at the bottom of `search.rs`/`cli.rs` (rather than in
a separate `tests/` directory) is Rust's convention for **unit tests** —
tests that need access to private, non-`pub` items in the module, which
`use super::*` gives them. A separate top-level `tests/` directory (one
level up from `src/`) is for **integration tests** instead — each file
there is compiled as its own crate that can only see your crate's public
API, closer to how an external user of the crate would call into it.
`rgrep`'s scaffold only needs the former, since `search` and
`Config::build` are both `pub` anyway and small enough to test directly
alongside their implementation.
