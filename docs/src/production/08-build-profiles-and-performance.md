# 8. Build Profiles & Performance

**Python/Java equivalent:** JVM startup/GC flags (Java), or reaching for
PyPy/Cython/`numba` (Python) when interpreted performance isn't enough.
Rust's version of this tuning happens at *compile time*, via
`Cargo.toml` profile settings — no runtime flags needed, because
there's no runtime doing JIT compilation or interpretation to tune.

## `debug` vs `release` — the one you already need to know

Covered in [Measuring Performance](../foundation/measuring-performance.md):
`cargo build` (debug) is unoptimized with overflow checks and debug
assertions on — 10-100x slower than `cargo build --release`. Never
benchmark, and never ship, a debug build.

## Tuning the release profile further

```toml
# Cargo.toml
[profile.release]
opt-level = 3        # 0-3, or "s"/"z" for size-optimized instead of speed-optimized
lto = true            # Link-Time Optimization — slower to compile, faster/smaller binary
codegen-units = 1     # fewer parallel compilation units → more optimization opportunity, slower build
strip = true          # strip debug symbols from the final binary → smaller artifact
panic = "abort"       # skip unwinding machinery on panic → smaller/faster, but no catch_unwind
```

Every one of these trades **compile time** for **runtime performance or
binary size** — there's no free lunch, which is why they're off by
default. Turn them on when you're building a release artifact you'll
actually ship, not during day-to-day development (where fast iteration
matters more than a maximally-optimized binary).

- **`lto = true`** — lets the optimizer see across crate boundaries
  (your code + every dependency) instead of optimizing each crate in
  isolation. Meaningful win for CPU-bound code, most valuable exactly
  where you'd reach for it in Java (`-XX:+UseParallelGC`-style tuning
  has no real analog — this is closer to what a JIT does automatically
  at runtime, done once at compile time instead).
- **`panic = "abort"`** — Rust's default panic behavior unwinds the
  stack (running destructors along the way, which is what makes
  `catch_unwind` from
  [Error Handling in Production](./03-error-handling-in-production.md)
  possible). `abort` skips that machinery entirely — smaller binary,
  faster panic, but the process just dies immediately with no recovery
  option. Fine for a CLI tool; a bad choice for a server that wants to
  isolate a panicking request handler from the rest of the process.

## Custom profiles for different needs

```toml
[profile.release-with-debug]
inherits = "release"
debug = true          # keep symbols for profiling, even in an otherwise-optimized build
```

Useful when you need release-level performance *and* the ability to
attach a profiler/debugger meaningfully — `cargo build --profile
release-with-debug`.

## Profiling — finding out *why* something is slow

`/usr/bin/time`/`hyperfine` (from
[Measuring Performance](../foundation/measuring-performance.md)) tell
you *that* something is slow. For *where*:

```sh
cargo install flamegraph
cargo flamegraph --bin rgrep -- error sample_logs/app.log
```

Produces an actual flame graph (same visualization Java's async-profiler
or Python's `py-spy` produce) — a visual breakdown of where CPU time is
actually spent, function by function, rather than guessing. `perf`
(Linux) / Instruments (macOS) underneath it are the same low-level
profiling tools JVM profilers ultimately wrap too.

## Rule of thumb for a project this size

- **`rgrep` as it stands**: default `--release` is plenty — it's already
  processing log files far faster than the I/O of reading them is
  likely to be the bottleneck.
- **Reach for profile tuning** only once you've *measured* a real
  problem (see [Measuring Performance](../foundation/measuring-performance.md))
  — tuning `Cargo.toml` profile flags before profiling is guessing, the
  same premature-optimization trap in any language.
