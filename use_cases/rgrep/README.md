# rgrep — build your own `grep`

```
rgrep "error" app.log
rgrep "error" sample_logs/*.log
```

This is a scaffold, not a solution. Every `todo!()` in `src/` is a spot
where you write the real logic. `cargo build` will compile as-is;
`cargo run` will panic at the first `todo!()` it reaches until you fill
it in — that's expected and is your progress indicator.

## Layout

```
src/
  main.rs    — wired up already: reads env::args, calls cli::Config::build,
               calls rgrep::run, handles errors with an exit code. Read it
               first — it's the map of how the pieces fit together.
  cli.rs     — Config struct + Config::build(&args) -> Result<Config, RgrepError>
  search.rs  — search(pattern, contents) -> Vec<&str>
  error.rs   — RgrepError enum + Display impl
  lib.rs     — run(config) -> Result<(), RgrepError>, ties file I/O +
               search + printing together
sample_logs/
  app.log, worker.log — fixtures with a few ERROR/WARN/INFO lines each,
  for manual testing and for the *.log multi-file milestone
```

## Milestones

Work top to bottom — each one only needs the `todo!()`s from that step
and earlier.

### M1 — search a string in memory

Fill in `search::search`. Run `cargo test search` until the three tests
in `src/search.rs` pass. No file I/O, no CLI parsing yet — just the
matching logic against a hardcoded `&str`.

### M2 — parse CLI arguments

Fill in `cli::Config::build`. Run `cargo test cli` until the four tests
in `src/cli.rs` pass. Handle the "no pattern given" / "no paths given"
cases by returning `RgrepError` variants — you'll define those variants
yourself in `error.rs` (there's no test forcing a specific variant name,
so pick names that make sense to you).

### M3 — wire it together with real file I/O

Fill in `error::RgrepError`'s `Display` impl, then `lib::run`. At this
point:

```sh
cargo run -- error sample_logs/app.log
```

should print the three ERROR lines from `app.log`.

### M4 — proper errors

Try it against a file that doesn't exist:

```sh
cargo run -- error sample_logs/does-not-exist.log
```

It should print a clear message to stderr (via your `RgrepError::Io`
variant's `Display` impl) and exit with a non-zero status — not panic,
not print a raw Rust `Debug` dump. Check `echo $?` after running it.

### M5 — multiple files

```sh
cargo run -- error sample_logs/*.log
```

Note: the shell (not your program) expands `*.log` into two separate
arguments before `rgrep` ever sees them — that's why `Config.paths` is
already a `Vec<String>`, and why M2's tests already cover multiple
paths. Your job here is just making `run` loop over all of them. Real
`grep` prefixes each line with its filename when searching more than one
file (`app.log:... ERROR ...`) — match that.

### Stretch goals (optional, in roughly increasing difficulty)

- `-i` flag for case-insensitive search (extend `Config` and `search`)
- `-n` flag to prefix matches with their line number
- Read from stdin when no file paths are given (`cat app.log | rgrep error`)
- Colorize the matched substring in the output
- Swap the literal-substring `search` for a real regex using the `regex`
  crate, behind a `-E`/`--regex` flag, keeping literal search as default

## Concepts this project exercises

| Concept | Where |
|---|---|
| `String` vs `&str` | `Config.pattern: String` (owned) vs `search`'s `&str` params (borrowed) |
| `&` references / borrowing | `search(pattern: &str, contents: &'a str)` — you never copy the file contents to search it |
| Ownership | deciding whether `Config::build` clones pieces of `args` or takes ownership of them |
| `Vec<T>` | `Config.paths`, `search`'s return type |
| Iterators | `contents.lines()`, `.filter()`/`.collect()` in `search` |
| `Result<T, E>` and `?` | every fallible function in this crate returns one |
| `match` | `RgrepError`'s `Display` impl, parsing args in `Config::build` |
| Structs | `Config`, `RgrepError` |
| Modules | `cli`, `error`, `search` as separate files wired together in `lib.rs` |
| File I/O | `std::fs::read_to_string` in `run` |

If you want the conceptual background for any of these before or while
you write the code, see `../../docs/src/foundation/` — particularly
[`strings.md`](../../docs/src/foundation/strings.md),
[`dereferencing.md`](../../docs/src/foundation/dereferencing.md), and
[`traits.md`](../../docs/src/foundation/traits.md) (relevant once you
implement `Display` for `RgrepError`).

## Running tests as you go

```sh
cargo test              # everything
cargo test search       # just search.rs
cargo test cli          # just cli.rs
```
