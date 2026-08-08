# How a CLI is implemented in Rust

> **Coming from Python/Java:** `std::env::args()` is `sys.argv` (Python)
> or the `String[] args` parameter of `main` (Java) — same information,
> same "index 0 is the program name" convention. Where Rust diverges is
> error handling: Python's `argparse` and Java's usual approach both let
> you throw/raise on bad input and let it propagate; Rust has no
> exceptions, so "bad arguments" has to become an explicit `Result`
> your `main` decides what to do with (see
> [Error Handling in main.rs](./error-handling.md)) rather than
> something that can silently bubble past a function that forgot to
> handle it.
>
> **Practical payoff:** everything below is the manual version of what
> `argparse` (Python) or a CLI library gives you for free — worth
> building by hand once (as `rgrep` does) so that reaching for `clap`
> later feels like automating something you understand, not magic.

This walks through what actually happens between typing
`rgrep "error" app.log` in a shell and your Rust program running —
using the [`rgrep`](../../../use_cases/rgrep) scaffold in this repo as
the running example.

## What the OS hands your program

When the shell runs `rgrep "error" app.log`, it doesn't call any special
"CLI" API — it starts your compiled binary as a process and gives it
three things:

1. **Argument list** — the words on the command line, already split by
   the shell (respecting quotes). For `rgrep "error" app.log` that's
   `["rgrep", "error", "app.log"]` — note argument `0` is always the
   program's own name/path, a Unix convention Rust preserves rather than
   hides from you.
2. **Three open file-like streams** — `stdin` (input), `stdout` (normal
   output), `stderr` (error/diagnostic output). These exist whether or
   not you use them; a terminal connects all three to your screen/keyboard
   by default, but a shell can redirect or pipe them independently
   (`rgrep error app.log > matches.txt` redirects only stdout).
3. **An exit code** — a single byte (0–255) your process returns when it
   finishes. `0` conventionally means success; anything else means
   failure. Shells and scripts branch on this (`if rgrep ...; then`,
   `rgrep ... && echo done`).

Everything else — flag parsing, help text, `--version` — is a library or
hand-written concern, not a language feature. Rust gives you raw access
to these three things via `std::env` and `std::io`, and you build up
from there.

## Reading arguments: `std::env::args()`

```rust
use std::env;

let args: Vec<String> = env::args().collect();
// args[0] == "rgrep" (or whatever path the binary was invoked as)
// args[1] == "error"
// args[2] == "app.log"
```

`env::args()` returns an iterator of `String`s (already-owned, valid
UTF-8 — if you need raw OS strings that might not be valid UTF-8, there's
a separate `env::args_os()`). Collecting into a `Vec<String>` is the
standard first step because indexing and slicing a `Vec` is far easier
than working with an iterator you can only consume once.

This is exactly what `rgrep`'s `main.rs` does:

```rust
let args: Vec<String> = env::args().collect();
let config = Config::build(&args).unwrap_or_else(|err| { /* ... */ });
```

## Turning `Vec<String>` into something meaningful

Raw args are just strings in a list — your program's job is to give them
structure. The common pattern (and what `rgrep::cli::Config::build`
does) is:

```rust
pub struct Config {
    pub pattern: String,
    pub paths: Vec<String>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, RgrepError> {
        let pattern = args.get(1).ok_or(RgrepError::MissingPattern)?.clone();
        let paths = args[2..].to_vec();
        if paths.is_empty() {
            return Err(RgrepError::MissingPaths);
        }
        Ok(Config { pattern, paths })
    }
}
```

A few things worth noticing here, because they're the concepts a CLI
forces you to confront:

- **`&[String]` not `Vec<String>`** — `build` only needs to *read* the
  args, not own them, so it takes a slice. This is the same
  borrow-vs-own tradeoff covered in
  [Dereferencing](./dereferencing.md) and [Stack vs Heap](./stack-vs-heap.md):
  no need to hand over ownership (or pay for a clone of the whole `Vec`)
  just to look at a few elements.
- **`args.get(1)` over `args[1]`** — indexing (`args[1]`) panics if the
  index is out of bounds; `.get(1)` returns `Option<&String>`, letting
  you convert a missing argument into a proper `Result::Err` instead of
  crashing. A CLI's entire "bad input" surface is user-controlled, so
  this distinction matters far more here than in typical internal code.
- **`Result<Config, RgrepError>`** — parsing args is the textbook
  "this can fail for reasons the caller needs to know about and react
  to" case that `Result<T, E>` exists for, as opposed to `panic!`, which
  is for bugs, not bad user input.

## Reporting errors: `stderr`, not `stdout`

```rust
eprintln!("Problem parsing arguments: {err}");
```

`eprintln!` writes to `stderr` where `println!` writes to `stdout`. This
isn't cosmetic — it's what lets `rgrep error app.log > matches.txt` put
*only* the matches in the file while error messages still show up on
your terminal, and it's what lets other programs pipe your stdout
(`rgrep error app.log | wc -l`) without your error chatter corrupting
the count. Well-behaved CLIs keep this separation strictly: stdout is
the *product* of the program, stderr is *commentary about* the program.

`{err}` in that format string works because `RgrepError` implements
`Display` — see [Traits](./traits.md#predefined-traits-you-already-use-constantly)
for why implementing one trait is what makes a custom error type
"printable" everywhere `std` and your own code expect one.

## Exiting with the right code

```rust
use std::process;

if let Err(err) = rgrep::run(config) {
    eprintln!("Application error: {err}");
    process::exit(1);
}
```

`main` returning normally exits with code `0`. `process::exit(1)` forces
a non-zero exit immediately (skipping any remaining cleanup/drops after
that call — worth knowing, since it means you should print/flush
whatever you need *before* calling it). This is the mechanism that lets
`rgrep`'s caller detect failure programmatically instead of having to
scrape output text.

`main` can alternatively return `Result<(), E>` itself (`fn main() ->
Result<(), Box<dyn Error>>`), and the runtime will print the error via
`Debug` and exit with code `1` automatically on `Err` — a shortcut for
simple programs, though it gives you less control over the exact exit
code and message format than handling it explicitly like above.

## Reading input beyond argv: files and stdin

Once paths are parsed, reading their contents is regular
[File I/O](https://doc.rust-lang.org/std/fs/fn.read_to_string.html):

```rust
let contents = std::fs::read_to_string(path)
    .map_err(|source| RgrepError::Io { path: path.to_string(), source })?;
```

Real Unix CLIs (grep included) also conventionally read from `stdin`
when no file arguments are given, so they compose in pipelines:
`cat app.log | rgrep error`. That's `std::io::stdin()`, whose `.lines()`
gives you the same iterator-of-lines shape as reading a file, just
sourced from the pipe instead — one reason it's worth writing your
search logic (`search(pattern, contents)`) generic over "some string I
already have," rather than baking file-reading into it directly.

## Scaling up: when hand-parsing isn't enough

The manual approach above is exactly right for `rgrep`'s scope — one
positional pattern, a list of positional paths, maybe a flag or two. It
gets unwieldy once a CLI needs: multiple flags with short/long forms
(`-i`/`--ignore-case`), auto-generated `--help` text, subcommands
(`git commit`, `git push`), or validation with good error messages. At
that point the standard tool is the [`clap`](https://docs.rs/clap)
crate, typically via its derive macro:

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    pattern: String,
    paths: Vec<String>,
    #[arg(short = 'i', long)]
    ignore_case: bool,
}

fn main() {
    let cli = Cli::parse();   // handles --help, errors, usage text for you
}
```

This is the same `Config` idea from `rgrep::cli`, just with the parsing
body generated by a derive macro (see
[Traits](./traits.md#most-of-these-can-be-auto-implemented-with-derive-)
for how `#[derive(...)]` generates trait impls from struct fields) instead
of hand-written `if`/`match` logic. It's worth building the manual
version first, the way `rgrep` does — `clap` is solving exactly the
problems you'll have just finished feeling by hand.

## Summary

| Piece | Rust mechanism |
|---|---|
| Get the raw arguments | `std::env::args()` → `Vec<String>` |
| Give them structure | a `Config`/`Cli` struct you define, built via a `Result`-returning constructor |
| Reject bad input without crashing | `Option`/`Result`, `.get()` over indexing, custom error enum |
| Report errors correctly | `eprintln!` (stderr), not `println!` (stdout) |
| Signal success/failure to the caller | process exit code — `0` implicitly, or `std::process::exit(n)` |
| Read file or piped input | `std::fs::read_to_string`, `std::io::stdin()` |
| Scale past a few flags | the `clap` crate, often via `#[derive(Parser)]` |
