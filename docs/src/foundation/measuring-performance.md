# Measuring wall-clock time and memory, without touching the code

> **Coming from Python/Java:** this replaces `timeit`/`cProfile`
> (Python) or a profiler/JMH setup (Java) — but notice everything below
> needs **zero code changes**, not even an import. That's only possible
> because a release-build Rust binary has no interpreter startup and no
> JIT warm-up to account for (unlike Python, and unlike a Java JVM's
> first few seconds) — the OS's own process-accounting tools (`time`)
> are already measuring something meaningful the instant the binary
> starts, so there's no need for in-process instrumentation just to get
> a fair number.

`cargo run`/`cargo build` doesn't report either by itself, but the OS
already tracks both for every process it runs — external tools just read
that out. No `Instant::now()`, no extra dependencies, no code edits.
Examples below run against [`rgrep`](../../../use_cases/rgrep), but every
command works unmodified against any Rust binary in this repo.

## Setup: always benchmark a release build

```sh
cd use_cases/rgrep
cargo build --release
```

`cargo run` (no flags) uses the **debug** profile - unoptimized, with
overflow checks and debug assertions on. It can be 10-100x slower than
release, so timing/memory numbers from a debug build don't reflect real
performance. Build `--release` first, then run the binary directly out of
`target/release/` (not via `cargo run --release`, which adds a small
amount of its own startup overhead on top of the binary).

## macOS (BSD `time`)

```sh
/usr/bin/time -l ./target/release/rgrep error sample_logs/app.log
```

(Note the explicit `/usr/bin/time` - the plain `time` you'd normally type
is your shell's *built-in* `time`, which only reports wall/user/sys, not
memory. `-l` on the real `/usr/bin/time` binary adds the memory/resource
section.)

Example output shape:

```
$ /usr/bin/time -l ./target/release/rgrep error sample_logs/app.log
sample_logs/app.log: ERROR failed to connect to cache: timeout after 5s
sample_logs/app.log: ERROR unhandled exception in request handler /api/users
sample_logs/app.log: ERROR database query failed: connection reset
        0.01 real         0.00 user         0.00 sys
             1310720  maximum resident set size
                   0  average shared memory size
                   0  average unshared data size
                   0  average unshared stack size
                 210  page reclaims
                   0  page faults
                   0  swaps
                   0  block input operations
                   0  block output operations
                   0  messages sent
                   0  messages received
                   0  signals received
                   0  voluntary context switches
                   4  involuntary context switches
             8213410  instructions retired
             5120044  cycles elapsed
              884736  peak memory footprint
```

What to actually read:

- **`real`** - wall-clock time (what you asked for). `user`/`sys` split
  that into CPU time spent in your code vs. in the kernel on its behalf -
  for a single-threaded, I/O-light program like `rgrep`, `real` and
  `user` should be close.
- **`maximum resident set size`** - peak physical memory (RAM) the process
  held at any point, in **bytes** on macOS (this line reads as ~1.3 MB,
  almost entirely binary/runtime overhead for a tiny input file).
- **`peak memory footprint`** - macOS-specific, a slightly different
  accounting of peak memory that also includes reclaimable pages; close
  to `maximum resident set size` for most purposes.
- Everything else (page faults, context switches, instructions retired)
  is low-level profiling detail, not needed for a basic "how much time and
  memory did this use" check.

For something with actually-visible memory growth - e.g. running `rgrep`
against a multi-gigabyte log file instead of the small `sample_logs/`
fixtures - the `real` time and `maximum resident set size` numbers will
move accordingly, which is the point of running this against different
inputs (or different implementations of `search`) for comparison.

## Linux (GNU `time`)

Same idea, different flag and units:

```sh
/usr/bin/time -v ./target/release/rgrep error sample_logs/app.log
```

Look for `Elapsed (wall clock) time` and `Maximum resident set size
(kbytes)` (Linux reports memory in **KB**, not bytes). GNU `time` isn't
installed on macOS by default; `brew install gnu-time` (or `coreutils`)
gets you `gtime -v ...` there if you want the more detailed GNU-style
output instead of BSD `time -l`.

## Comparing multiple runs / reducing noise

A single `time` run can be noisy (OS scheduling, cold caches, thermal
throttling). For anything where the difference matters (e.g. comparing
two different implementations of `search`, or the same binary against a
small vs. large log file), run it a handful of times and eyeball the
spread, or use a proper benchmarking tool:

```sh
brew install hyperfine
hyperfine './target/release/rgrep error sample_logs/app.log'
```

`hyperfine` runs the binary repeatedly, warms up first, and reports
mean/stddev/min/max wall-clock time - still zero code changes, just a
statistically sturdier version of `time`.
