# Error handling, line by line: `rgrep`'s `main.rs`

> **Coming from Python/Java:** there's no `try`/`except`/`catch` in
> Rust, full stop — every function that can fail says so in its return
> type (`Result<T, E>`), and the caller is statically forced to
> acknowledge that, the same way Java forces you to handle a *checked*
> exception, except Rust applies that discipline to *every* failure, not
> just the ones a library author remembered to declare `throws` on. The
> practical effect: you can't have a Python-style bug where an exception
> from deep inside a call stack surfaces somewhere unrelated three
> functions later — the failure is visible in every signature along the
> way.
>
> **Practical payoff:** this page is what that discipline looks like in
> a real ten-line `main` — worth reading closely once, since the pattern
> (`unwrap_or_else` here, `if let Err` there, `?` one layer down) is what
> you'll reach for in every Rust program's entry point, not just this
> one.

This walks through every line of
[`rgrep/src/main.rs`](../../../use_cases/rgrep/src/main.rs), focused on
*why* it handles errors the way it does — including why the two error
sites in this ten-line function use two different patterns.

```rust
use std::env;
use std::process;

use rgrep::cli::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(err) = rgrep::run(config) {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}
```

## The shape both fallible calls share

`Config::build` and `rgrep::run` both return `Result<T, RgrepError>` —
see [CLI Implementation](./cli.md#turning-vecstring-into-something-meaningful)
for why parsing/running a CLI is exactly the "can fail for reasons the
caller must react to" case `Result` exists for. `Result<T, E>` is an
enum with two variants, `Ok(T)` and `Err(E)`, and `main` has to decide
what to do in each case. That decision is *all* error handling is —
everything below is just different syntax for "what do I do with an
`Err`."

The reason `main` uses two different idioms for two calls that look
structurally similar is that **the two calls have different `T`s**:

| Call | Return type | On success, `main` needs... |
|---|---|---|
| `Config::build(&args)` | `Result<Config, RgrepError>` | the `Config` value, to keep going |
| `rgrep::run(config)` | `Result<(), RgrepError>` | nothing — `()` carries no information |

That single difference is why one call uses `.unwrap_or_else(...)` and
the other uses `if let Err(...)`.

## `Config::build(&args).unwrap_or_else(|err| { ... })`

```rust
let config = Config::build(&args).unwrap_or_else(|err| {
    eprintln!("Problem parsing arguments: {err}");
    process::exit(1);
});
```

`main` needs a `Config` to pass to `rgrep::run` on the next line — the
`let config = ...` binding requires *some* `Config` value to exist.
`unwrap_or_else` is a method on `Result<T, E>` with this signature
(simplified):

```rust
fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T
```

- On `Ok(config)`, it returns `config` directly — the closure never runs.
- On `Err(err)`, it calls the closure with `err` and returns *whatever
  the closure returns*, which must be a `T` (a `Config`, here) to
  satisfy the function signature.

That's the catch: the closure `|err| { eprintln!(...); process::exit(1); }`
doesn't look like it returns a `Config` — it looks like it returns
nothing. It compiles anyway because `process::exit` has return type `!`
("never" — Rust's type for "this function provably does not return,"
since it terminates the process immediately). A value of type `!` is
allowed to stand in for *any* type the compiler expects, because control
flow never reaches the point where the mismatched type would matter.
So `unwrap_or_else` is satisfied: either the closure produces a real
`Config`, or it never returns at all, and both cases type-check.

This is the pattern for: *"get me the value, or handle the failure and
stop — there is no valid `Config` to fall back to, so don't try to keep
going."*

## `if let Err(err) = rgrep::run(config) { ... }`

```rust
if let Err(err) = rgrep::run(config) {
    eprintln!("Application error: {err}");
    process::exit(1);
}
```

`rgrep::run` returns `Result<(), RgrepError>`. On success there is
nothing to extract — `()` is the empty tuple, meaning "ran fine, no
payload." Nothing after this call needs a value from it, so there's no
`let result = ...` to satisfy. `if let Err(err) = ...` is exactly what
it says: *"if this turned out to be an `Err`, bind it to `err` and run
this block; if it's `Ok(())`, skip the block and fall through"* — which
here means falling off the end of `main`, exiting with code `0`.

Using `unwrap_or_else` here instead would compile, but reads worse — you'd
be forcing a `()`-returning closure into a value-producing combinator
for a value nobody wants. `if let` is the idiomatic shape once the
success case carries no payload.

## The shared parts: `eprintln!` and `process::exit(1)`

Both branches do the same two things once they have an error, and both
choices matter for the same reason: **this is a CLI, and its behavior
under failure is part of its contract with whoever/whatever invokes it**
(a human, a shell script, another program in a pipeline).

- **`eprintln!("...: {err}")` writes to stderr, not stdout.** `{err}`
  works because `RgrepError` implements the `Display` trait (see
  [Traits](./traits.md)) — that impl is what decides the actual wording
  the user sees. Writing to stderr (not `println!`'s stdout) means a
  redirect like `rgrep error app.log > matches.txt` still shows the
  error on-screen instead of corrupting the output file, and a pipe like
  `rgrep error app.log | wc -l` still gets a clean count. This is
  covered in more depth in
  [CLI Implementation](./cli.md#reporting-errors-stderr-not-stdout).
- **`process::exit(1)` sets a non-zero exit code and terminates
  immediately.** By Unix convention, `0` means success and any nonzero
  value means failure; scripts and shells branch on this
  (`rgrep ... && echo done`, `if rgrep ...; then`). Without an explicit
  `process::exit(1)`, a program that merely *prints* an error message
  but then falls off the end of `main` normally would still exit `0` —
  silently telling every caller "this succeeded" when it didn't. The
  explicit exit call is what makes the failure real, not just visible.

Note `process::exit` skips the rest of the current scope — no further
code after it runs, and any values that would normally be dropped on
the way out of `main` are not. That's fine here since nothing owns
resources needing cleanup at that point, but it's why `eprintln!` is
written *before* `process::exit`, not after: anything you still need to
happen has to happen first.

## Why not just `?` everywhere, or `.unwrap()`?

Two idioms conspicuously *not* used here, and why not:

- **`.unwrap()` / `.expect("...")`** would panic on `Err`, printing a
  Rust-internal `Debug` message and a backtrace hint — appropriate for
  bugs you don't expect to happen, wrong for "the user passed a bad file
  path," which is routine, expected, user-facing input validation, not
  a bug in the program.
- **`?`** propagates an `Err` to the *caller* of the current function.
  `main` has no caller (in the Rust-program sense) to propagate to — it
  needs to be where the buck stops, translating a `Result` into
  process-exit behavior. (`?` *is* used one layer down, inside
  `Config::build` and `rgrep::run` themselves, to propagate failures
  from `std::fs::read_to_string` etc. up to `main` — see
  [CLI Implementation](./cli.md#reading-input-beyond-argv-files-and-stdin).
  `main` is the end of that chain, not a link in it.)

## A related but different failure mode: `todo!()`, `unimplemented!()`, `panic!()`

Everything above is about *handling* an error someone anticipated.
`todo!()` is different — it's a placeholder for code that doesn't exist
yet, used throughout the [`rgrep`](../../../use_cases/rgrep) scaffold
itself:

```rust
pub fn search<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    todo!("find every line in `contents` containing `pattern`")
}
```

`search`'s signature promises a `Vec<&'a str>`, but there's no
implementation yet. `todo!()` still compiles because its return type is
the special "never type" `!` (the same type `process::exit` has, from
earlier on this page), which Rust lets coerce into *any* expected type —
the compiler trusts that this branch never actually produces a value,
because calling it always panics. That's the whole trick: `cargo build`
succeeds on the scaffold as-is, and `cargo test` only fails once
execution actually *reaches* a `todo!()` at runtime — which is exactly
why running `cargo test` against the unfinished scaffold gives you a
precise, per-function progress signal instead of a wall of compile
errors.

- `todo!()` — "not implemented yet, but will be." Optionally takes a
  `format!`-style message: `todo!("handle the {case} branch")`.
- `unimplemented!()` — behaves identically; the convention is to use it
  for "deliberately not implementing this," as opposed to `todo!()`'s
  "coming soon."
- `panic!()` — the general macro both are built on. Any of the three
  immediately unwind/abort the current thread with a message, which is
  categorically different from returning `Err`: a `Result` is a value
  the caller can inspect and react to; a panic is the program declaring
  "this is a bug, not a recoverable condition" and stopping.

This is the same distinction covered above for `.unwrap()`/`.expect()`
vs proper `Result` handling: panicking macros are for states you don't
expect to reach in working code (missing implementations, genuine
invariant violations), never for routine failure conditions like bad
user input or a missing file — those belong in `RgrepError` and get
handled the way `main` handles them here.

## Summary

| Situation | Idiom | Why |
|---|---|---|
| Need the success value to continue, no valid fallback exists | `.unwrap_or_else(\|err\| { ...; process::exit(1) })` | closure must "return" `T`; `process::exit`'s `!` type satisfies that by never returning |
| Success carries no value (`Result<(), E>`) | `if let Err(err) = ... { ... }` | nothing to extract on `Ok`, so there's nothing to bind — just branch on failure |
| Reporting the failure | `eprintln!("...: {err}")` | stderr keeps error text out of stdout, which pipelines/redirects depend on being clean |
| Signaling failure to the OS/caller | `process::exit(1)` | falling off the end of `main` exits `0` regardless of what was printed — the exit code is the only part scripts actually check |
| Deeper in the call stack (`Config::build`, `run`) | `?` | propagates the error upward instead of handling it — `main` is where handling finally happens |
