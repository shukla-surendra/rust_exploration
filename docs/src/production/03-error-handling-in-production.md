# 3. Error Handling in Production

[Option, Result & unwrap_or_else](../foundation/option-result.md) and
[Error Handling in main.rs](../foundation/error-handling.md) cover the
language mechanics. This chapter covers the practices that matter once
the program is something other people run: which crates to reach for,
where `.unwrap()` becomes a liability, and what to do when a panic
happens anyway.

## `thiserror` vs `anyhow` — the two standard answers to "what's `E`?"

`rgrep`'s hand-written `RgrepError` enum (see
[Error Handling in main.rs](../foundation/error-handling.md)) is exactly
right for a small binary with a handful of failure modes. Two crates
cover the more common real-world shapes:

**`thiserror`** — for **libraries**, where callers need to `match` on
*which specific error* occurred:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum RgrepError {
    #[error("missing search pattern")]
    MissingPattern,
    #[error("no file paths given")]
    MissingPaths,
    #[error("couldn't read {path}: {source}")]
    Io { path: String, source: std::io::Error },
}
```

This is a derive macro that generates exactly the `Display` impl
`rgrep`'s `error.rs` currently asks you to hand-write — same enum shape,
same `#[error("...")]` message per variant, just without writing the
`match` yourself. Still a real `enum`, so callers can still
`match err { RgrepError::MissingPattern => ..., _ => ... }`.

**`anyhow`** — for **binaries** (application entry points), where the
caller (typically just `main`, printing the error and exiting) doesn't
need to distinguish error variants, just report them:

```rust
use anyhow::{Context, Result};

fn run(config: Config) -> Result<()> {
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {path}"))?;
    // ...
    Ok(())
}
```

`anyhow::Result<T>` is shorthand for `Result<T, anyhow::Error>` — a
single catch-all error type that can wrap *any* error (`?` converts
automatically from anything implementing `std::error::Error`), with
`.context()`/`.with_context()` to attach a human-readable breadcrumb at
each layer without defining a new enum variant for every possible
failure. Rule of thumb: **`thiserror` in library crates you publish,
`anyhow` in the binary that consumes them** — `rgrep`'s `lib.rs` would
use `thiserror`-style typed errors, `main.rs` could use `anyhow` if the
error variants stopped mattering to whoever's reading `main`'s output.

## Where `.unwrap()`/`.expect()` stop being acceptable

In a learning scaffold, `.unwrap()` is fine — you want to see the panic
and its exact line number while iterating. In production code, every
`.unwrap()`/`.expect()` is a claim: "I have proven this can never be
`None`/`Err` here." Audit each one:

```rust
// defensible: parsing a literal you wrote yourself
let re = Regex::new(r"^\d+$").unwrap();   // this literal is provably valid — a bug if it ever isn't

// NOT defensible: anything derived from user input, a file, the network, args
let n: i32 = user_input.parse().unwrap();  // panics the whole program on bad input
```

`.expect("message")` over bare `.unwrap()` at minimum turns a cryptic
panic into a diagnostic one — but the real fix for user-facing/external
input is propagating a `Result` (chapter 6 of the workbook) instead of
panicking at all. A production Rust service crashing because of
malformed input it should have rejected gracefully is the same category
of bug as an uncaught `NullPointerException` taking down a Java service.

## Panics still happen — plan for them anyway

Even with careful error handling, a genuine bug (an unreachable branch
that wasn't, an integer overflow in debug builds, a third-party
dependency panicking) can still occur. Two mechanisms matter for a
production binary:

```rust
std::panic::set_hook(Box::new(|info| {
    eprintln!("panic: {info}");
    // send to your logging/monitoring system before the process exits
}));
```

A panic hook runs *before* the program unwinds/aborts — this is where
you'd emit a structured log line (see
[Logging & Observability](./04-logging-and-observability.md)) so a panic
in production shows up in your monitoring instead of just a mysterious
process restart with no trace of why.

```rust
let result = std::panic::catch_unwind(|| {
    risky_third_party_call()
});
```

`catch_unwind` can stop a panic from crashing the *whole* process — used
sparingly, typically at a boundary like "one request handler in a
server shouldn't take the whole server down." It is **not** a
`try`/`except` substitute for normal error handling — reach for `Result`
first; `catch_unwind` is specifically for isolating panics you can't
prevent (e.g. in code you don't control) from taking down unrelated
work.

## Exit codes still matter

Following on from [Error Handling in main.rs](../foundation/error-handling.md):
a production CLI/service should exit with a code that means something
to whatever's watching it (a shell script, systemd, an orchestrator) —
`0` for success, distinct nonzero codes for distinct failure categories
if that distinction is useful downstream, not just a blanket `exit(1)`
for everything.
