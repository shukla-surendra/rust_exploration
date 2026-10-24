# 2. Testing Strategy

[Testing](../foundation/testing.md) covers the mechanics — `#[test]`,
`#[cfg(test)]`, `cargo test`. This chapter covers the strategy: what
*kinds* of tests a production Rust crate has, beyond the unit tests
already sitting in `rgrep`'s `src/cli.rs`/`src/search.rs`.

## The four kinds

| Kind | Lives in | Tests | Python/Java analog |
|---|---|---|---|
| Unit tests | `#[cfg(test)] mod tests` inside each `src/*.rs` file | one function/module, including private items | a `pytest` file next to the module, or a JUnit test class |
| Integration tests | top-level `tests/*.rs` | your crate's public API only, compiled as an external user would see it | end-to-end API tests |
| Doc tests | code blocks inside `///` doc comments | that your documented examples actually still compile and run | Python's `doctest` module, almost exactly |
| Benchmarks | `benches/*.rs` (needs the `criterion` crate, or nightly `#[bench]`) | performance regressions | JMH (Java), `pytest-benchmark` |

## Integration tests: testing the crate from the outside

```
rgrep/
  src/
    lib.rs
  tests/
    cli_integration.rs
```

```rust
// tests/cli_integration.rs
use rgrep::cli::Config;

#[test]
fn build_rejects_missing_pattern() {
    let args = vec!["rgrep".to_string()];
    assert!(Config::build(&args).is_err());
}
```

Each file under `tests/` is compiled as its **own separate crate** that
depends on yours like any external user would — it can only see `pub`
items, exactly the boundary a real caller of your library would face.
This is the test that would catch "I accidentally made something `pub`
that leaks an implementation detail" or "my public API doesn't actually
work the way I documented it," which a unit test (with full access via
`use super::*`) can't catch by construction.

## Doc tests: examples that can't go stale

```rust
/// Finds every line containing `pattern`.
///
/// ```
/// let contents = "hello\nworld with error";
/// let matches = rgrep::search::search("error", contents);
/// assert_eq!(matches, vec!["world with error"]);
/// ```
pub fn search<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    // ...
}
```

`cargo test` compiles and runs every code block inside a `///` doc
comment as its own tiny test. This solves a specific, chronic
documentation problem: an example in a docstring/Javadoc can silently
rot the moment the function's behavior changes, because nothing checks
it. A Rust doc-comment example *cannot* rot silently — if it stops
matching reality, `cargo test` fails. This is also why crate
documentation on [docs.rs](https://docs.rs) tends to be more trustworthy
than a typical README's example snippets.

## Property-based testing: generate inputs instead of hand-writing them

Regular tests assert a *specific* input produces a *specific* output.
Property tests (via the `proptest` or `quickcheck` crates — not in
`std`) assert a rule holds for *many randomly generated* inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn search_never_returns_lines_missing_the_pattern(pattern in "\\PC+", contents in ".*") {
        for line in rgrep::search::search(&pattern, &contents) {
            prop_assert!(line.contains(&pattern));
        }
    }
}
```

The closest Python equivalent is `hypothesis`; the idea transfers
directly. Reach for this when a function's *correctness condition* is
easier to state as an invariant ("every returned line actually contains
the pattern") than as a finite list of example inputs — property tests
are especially good at finding edge cases (empty strings, unicode,
extremely long input) a human wouldn't think to write by hand.

## What to actually cover, pragmatically

- **Unit tests**: the logic-heavy functions — parsing, transforming,
  anything with branches or edge cases (empty input, boundary values).
  `rgrep`'s `search`/`Config::build` scaffold already does this.
- **Integration tests**: the "does the public API actually do what it
  promises" check, especially before publishing a library crate anyone
  else will depend on.
- **Doc tests**: any public function whose usage isn't obvious from its
  signature alone — free correctness-checked documentation.
- **Benchmarks**: only once you have a performance requirement to
  protect — premature benchmarking is as wasteful as premature
  optimization (see [Measuring Performance](../foundation/measuring-performance.md)
  for measuring without a formal benchmark suite first).
