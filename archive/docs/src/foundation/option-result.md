# `Option`, `Result`, and how you get values out of them

> **Coming from Python/Java:** if you've used Java 8+'s `Optional<T>`,
> you already know `Option<T>` almost exactly — same idea (`Optional.of`
> ≈ `Some`, `Optional.empty()` ≈ `None`), except Rust's version is load-
> bearing everywhere (a `HashMap` lookup, a `Vec` index) rather than an
> opt-in wrapper you have to remember to reach for. `Result<T, E>` has no
> real Java/Python equivalent — it's what replaces exceptions entirely,
> not just `null`. The habit to unlearn from Python: `None` in Rust can
> never silently flow through code that forgot to check for it the way a
> Python `None` can (`'NoneType' object has no attribute ...`, three
> function calls away from where it actually went wrong) — the compiler
> won't let you touch the inside of an `Option`/`Result` without
> acknowledging both cases first.
>
> **Practical payoff:** this page's `unwrap`/`unwrap_or_else`/`.ok_or()`
> family is what you'll type dozens of times a day in real Rust code —
> worth having the decision table below memorized, or at least bookmarked.

Rust has no `null` and no exceptions. Both "a value that might not exist"
and "an operation that might fail" are ordinary enum values instead —
`Option<T>` and `Result<T, E>` — and the compiler forces you to deal with
both cases before you can use what's inside. This page covers what they
are, the family of methods for getting a value back out (`unwrap`,
`unwrap_or_else`, and friends), and `println!`/`eprintln!`, since where
you print an error is part of the same "handling the failure case
correctly" story.

## `Option<T>` — a value that might not be there

```rust
enum Option<T> {
    Some(T),
    None,
}
```

Anywhere another language would use `null`/`nil`/`None`-as-a-sentinel,
Rust uses `Option<T>`. `HashMap::get` is the canonical example: looking
up a key that isn't there can't return "nothing" silently — it returns
`None`, and looking up a key that *is* there returns `Some(&value)`. The
type system then refuses to let you use the inner value without first
handling both cases — there's no way to accidentally dereference a
"null" `Option`, because there's no way to get at the `T` without
acknowledging it might not be there.

## `Some` — constructing and matching the "value is present" case

`Some` isn't a keyword — it's a regular enum variant, and also doubles as
a function: `Some(5)` is literally calling `Some` as a tuple-struct-style
constructor that wraps `5` into an `Option<i32>`. You'll meet it in three
shapes:

**As a value you construct**, when a function needs to signal "found
something":

```rust
fn find_first_error(lines: &[&str]) -> Option<&str> {
    for line in lines {
        if line.contains("ERROR") {
            return Some(line);   // wrap the found value
        }
    }
    None                          // nothing found — no error to wrap
}
```

**As a pattern you match against**, to get the wrapped value back out:

```rust
match find_first_error(&lines) {
    Some(line) => println!("first error: {line}"),
    None => println!("no errors"),
}
```

`Some(line)` here is a *pattern*, not a call — it says "if this is the
`Some` variant, bind whatever's inside to `line`." This is the same
mechanism as `Ok`/`Err` patterns in a `match` on a `Result`. It also
composes with `&`: `Some(&cached)` in a pattern strips off one layer of
reference while unwrapping, binding `cached` to an owned copy rather
than a `&reference` — handy when matching against something like
`HashMap::get`'s `Option<&V>` return type and you need an owned value
out the other end (works cheaply for `Copy` types like `u64`; for
non-`Copy` types reach for `.cloned()`/`.copied()` on the `Option`
instead).

**As a condition**, when you only care about the `Some` case and want to
skip the ceremony of a full `match`:

```rust
if let Some(line) = find_first_error(&lines) {
    println!("first error: {line}");
}
// nothing printed if it was None — no explicit else needed

// or, to loop while values keep coming (e.g. draining an iterator):
while let Some(x) = stack.pop() {
    println!("{x}");
}
```

This is the exact `if let`/`while let` family used elsewhere in this repo
— see [Error Handling in main.rs](./error-handling.md#if-let-errerr--rgreprunconfig---)
for the `if let Err(...)` sibling pattern used on `Result`.

### Turning a `Some`/`None` into a `Result`: `.ok_or()`

`Option` and `Result` aren't just similar — you convert directly between
them, and this is precisely where "a value might not be there" becomes
"an error to report." `.ok_or(err)` turns `Some(v)` into `Ok(v)` and
`None` into `Err(err)`:

```rust
pub fn build(args: &[String]) -> Result<Config, RgrepError> {
    let pattern = args.get(1)
        .ok_or(RgrepError::MissingPattern)?   // Option<&String> -> Result<&String, RgrepError>
        .clone();
    // ...
}
```

`args.get(1)` returns `Option<&String>` — `Some(&args[1])` if that index
exists, `None` if the caller didn't pass enough arguments. But
`Config::build`'s return type is a `Result`, not an `Option` — "missing
pattern" needs to become an actual `RgrepError` the caller can report,
not a bare `None`. `.ok_or(RgrepError::MissingPattern)` is exactly that
bridge, and the `?` right after propagates the `Err` immediately if the
argument was missing (see [CLI Implementation](./cli.md) for `Config::build`'s
full context in `rgrep`). `.ok_or_else(|| ...)` is the lazy counterpart —
same relationship to `.ok_or()` as `.unwrap_or_else()` has to
`.unwrap_or()` below: use it when building the error value costs
something and shouldn't happen on the success path.

## `Result<T, E>` — an operation that might fail

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Same shape, different meaning: `Ok(value)` for success, `Err(error)` for
failure, where `E` carries *why* it failed (unlike `Option`, where
`None` carries no explanation). This is what `Config::build` and
`rgrep::run` return in [`rgrep`](../../../use_cases/rgrep) — see
[Error Handling in main.rs](./error-handling.md) — and what
`std::fs::read_to_string` returns (`Result<String, io::Error>`): reading
a file can fail for reasons worth knowing (not found, permission
denied), so the failure carries an `io::Error`, not just an "it didn't
work."

## Getting the value out: the `unwrap` family

Both types share the same family of methods for extracting the value
(or reacting to its absence). They only differ in *what happens on the
"bad" case* (`None` / `Err`):

| Method | On success | On `None`/`Err` |
|---|---|---|
| `.unwrap()` | returns the value | **panics** — crashes the program, printing a debug message |
| `.expect("msg")` | returns the value | **panics** with your custom message prepended — same crash, better diagnostics |
| `.unwrap_or(fallback)` | returns the value | returns `fallback` — a plain value, computed eagerly *even on success* (it still has to exist to be passed in) |
| `.unwrap_or_else(\|err\| ...)` | returns the value | **runs the closure** and returns whatever it produces — computed lazily, only when actually needed |
| `.unwrap_or_default()` | returns the value | returns `T::default()` (needs `T: Default`) |

The last three all avoid panicking — that's the important split. `unwrap`
and `expect` are for situations you consider a bug if they ever trigger
("this file I just wrote a line ago should obviously still be openable —
if it's not, something is deeply wrong, crash loudly"). Everything else
is for situations that are *expected* to fail sometimes — bad user
input, a missing file, a network hiccup — where crashing the whole
program would be the wrong response.

### `unwrap_or` vs `unwrap_or_else`

The difference that trips people up: `unwrap_or` takes an already-built
*value*; `unwrap_or_else` takes a *function* (closure) that builds the
value on demand.

```rust
// unwrap_or: `expensive_default()` runs every time, even when config_str is Ok
let a = config_str.unwrap_or(expensive_default());

// unwrap_or_else: the closure only runs if config_str is actually Err
let b = config_str.unwrap_or_else(|_| expensive_default());
```

`unwrap_or_else` is also the *only* option of the two when the fallback
needs to **do something** rather than just **be something** — print a
message, exit the process, look at the specific error that occurred.
That's exactly the case in `rgrep`'s `main.rs`:

```rust
let config = Config::build(&args).unwrap_or_else(|err| {
    eprintln!("Problem parsing arguments: {err}");
    process::exit(1);
});
```

You couldn't write this with `unwrap_or` — there's no plain `Config`
*value* that means "print an error and quit." The closure form is
required because the fallback is a sequence of actions, and it needs
`err` (the actual parse failure) to describe what went wrong.

`process::exit` never returns (its type is `!`, the "never" type), which
is what lets this closure satisfy `unwrap_or_else`'s signature at all —
see [Error Handling in main.rs](./error-handling.md#configbuildargsunwrap_or_elseerr---) for the full walkthrough of why that
type-checks.

## Three ways to handle the same `Result`, side by side

```rust
// 1. match — most explicit, handles both arms by hand
let config = match Config::build(&args) {
    Ok(c) => c,
    Err(err) => {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    }
};

// 2. if let — good when you only care about one arm
if let Err(err) = rgrep::run(config) {
    eprintln!("Application error: {err}");
    process::exit(1);
}

// 3. unwrap_or_else — a combinator, good when you need the success value
//    and the failure path is a short, self-contained action
let config = Config::build(&args).unwrap_or_else(|err| {
    eprintln!("Problem parsing arguments: {err}");
    process::exit(1);
});
```

All three are idiomatic; `rgrep`'s `main.rs` actually uses #2 and #3 side
by side (see [Error Handling in main.rs](./error-handling.md)) — the
choice comes down to whether the success case produces a value you need
(`unwrap_or_else`/`match`) or not (`if let`).

## Other common combinators worth knowing

Beyond the `unwrap` family, both types support chaining without ever
"opening" the value by hand:

| Method | Does |
|---|---|
| `.map(f)` | if `Some`/`Ok`, transform the inner value with `f`; otherwise pass `None`/`Err` through unchanged |
| `.and_then(f)` | like `.map`, but `f` itself returns an `Option`/`Result` — avoids nested `Option<Option<T>>` |
| `.is_some()` / `.is_none()` / `.is_ok()` / `.is_err()` | just check which variant it is, no extraction |
| `.ok()` | `Result<T, E> → Option<T>`, discarding the error |
| `.ok_or(err)` / `.ok_or_else(\|\| err)` | `Option<T> → Result<T, E>` — turns `None` into `Err(err)`, `Some(v)` into `Ok(v)` |
| `?` | inside a function that itself returns `Result`, propagate an `Err` straight to the caller instead of handling it locally |

`.map`/`.and_then` are what a chain like
`some_option.as_ref().map(|v| do_something(v)).unwrap_or(false)` is built
from: `.as_ref()` borrows the contents instead of moving them out,
`.map(...)` transforms the inner value only if it's `Some`, and
`.unwrap_or(false)` supplies the fallback for the `None` case — three
combinators standing in for what would otherwise be a multi-arm `match`.

## `println!` vs `eprintln!` — where the value goes once you have it

Extracting a value or an error is only half the story for a CLI — where
you print it matters too. A process has two separate output channels,
even though a terminal displays both in the same place:

- **stdout** — the program's actual output/product. `println!` writes
  here.
- **stderr** — diagnostics, errors, progress messages. `eprintln!`
  writes here.

They look identical in a terminal, which is why the distinction is easy
to miss — until output gets redirected or piped:

```sh
rgrep error app.log > matches.txt   # only stdout goes to the file
rgrep error app.log | wc -l         # only stdout is counted
```

If error messages went through `println!`, they'd land in `matches.txt`
mixed in with real matches, or get counted by `wc -l` as if they were
results. Because `rgrep`'s `main.rs` uses `eprintln!` for both error
sites, errors still show up on-screen in either case, while stdout stays
exactly the "real" output. Rule of thumb: **`println!` for what your
program produces, `eprintln!` for what your program has to say about
itself.** This is covered in more depth in
[CLI Implementation](./cli.md#reporting-errors-stderr-not-stdout).

`{err}` inside `eprintln!("Application error: {err}")` works because the
error type (`RgrepError`) implements the `Display` trait — see
[Traits](./traits.md) — which is what decides the actual wording shown.

## Summary

- `Option<T>` = maybe a value (`Some`/`None`); `Result<T, E>` = maybe a
  value, with a reason attached on failure (`Ok`/`Err`).
- `Some(v)` is both a constructor (wrap a found value) and a pattern
  (unwrap it back out in `match`/`if let`/`while let`).
- `.ok_or(err)` is the bridge from `Option` to `Result` — the exact move
  that turns "this argument wasn't there" into an actual, reportable
  error in `rgrep::cli::Config::build`.
- `.unwrap()`/`.expect()` — crash on failure; use only for "this should
  be impossible" cases.
- `.unwrap_or(v)` — plain fallback value, built eagerly.
- `.unwrap_or_else(|e| ...)` — fallback *computed on demand*, the only
  option when the fallback needs to run code (print, exit, inspect `e`)
  rather than just exist.
- `match`/`if let`/combinators (`.map`, `.and_then`, `?`) are different
  syntaxes for the same underlying decision — which one reads best
  depends on whether you need the success value and how many arms you
  actually care about.
- `println!` is your program's output; `eprintln!` is its commentary —
  keeping them separate is what makes redirection and piping work
  correctly.
