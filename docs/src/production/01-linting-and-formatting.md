# 1. Linting & Formatting

**Python/Java equivalent:** `black`/`ruff` + `flake8` (Python),
`checkstyle`/`spotless` (Java) — except Rust's versions ship in the
toolchain, so there's no "which formatter does the team pick" debate.

## `cargo fmt` — one canonical style, no debate

```sh
cargo fmt              # reformat the whole crate in place
cargo fmt -- --check   # fail (exit nonzero) if anything isn't formatted — use in CI
```

Rust's formatting is close to a solved problem community-wide: almost
every public crate uses default `rustfmt` settings, so there's very
little of the "tabs vs spaces" bikeshedding Python/JS projects have.
Run it before every commit; run `--check` in CI so a PR can't merge
unformatted code (see [CI/CD](./07-ci-cd.md)).

## `cargo clippy` — a linter that actually knows Rust idioms

```sh
cargo clippy                          # lint the whole crate
cargo clippy -- -D warnings           # treat every warning as an error — use in CI
```

Clippy catches things the compiler considers valid but that are almost
always mistakes or non-idiomatic: needless `.clone()` calls, `if x ==
true` instead of `if x`, manually reimplementing something `Iterator`
already provides, `.unwrap()` where a `?` would be clearer. It's
substantially more opinionated than `flake8`/`checkstyle` — closer to
`ruff`'s more aggressive rule set, or a very well-tuned Java IDE
inspection pass, except it runs from the command line and in CI, not
just inside an editor.

```rust
// clippy: "redundant clone"
let s = String::from("hi");
foo(s.clone());
drop(s); // if this weren't here, clippy would flag the .clone() as unnecessary

// clippy: "this `.map().unwrap_or()` can be `.map_or()`"
opt.map(|x| x + 1).unwrap_or(0);   // clippy suggests: opt.map_or(0, |x| x + 1)
```

Clippy lints have levels — `warn` (default, shown but doesn't fail the
build), `deny` (fails the build), `allow` (silenced). Silence a
specific lint inline when you've deliberately chosen the flagged
pattern, with a comment explaining why (so it reads as a decision, not
an oversight):

```rust
#[allow(clippy::too_many_arguments)] // deliberately wide constructor, see ADR-3
fn build(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) { }
```

## Wiring both in as a pre-commit habit

Neither tool runs automatically — you have to actually invoke them.
Two common ways to make that non-optional:

```sh
# a git pre-commit hook (.git/hooks/pre-commit, or via a tool like `pre-commit`)
cargo fmt -- --check && cargo clippy -- -D warnings
```

Or, more reliably (since local hooks can be skipped/forgotten), enforce
both as required CI checks — see [CI/CD](./07-ci-cd.md) — so a PR
literally cannot merge without passing both. Treat a red `clippy -D
warnings` run the same way you'd treat a failing test: something to fix
before moving on, not something to silence by reflex.
