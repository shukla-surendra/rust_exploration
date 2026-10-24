# Profiling Rust programs

How to measure and profile CPU and memory usage of a Rust program, both from the
outside (tools that watch a running binary) and from the inside (crates you add to
the code).

- [1. Build for profiling first](#1-build-for-profiling-first)
- [2. Quick numbers: time and memory of one run](#2-quick-numbers-time-and-memory-of-one-run)
- [3. Watching a running process](#3-watching-a-running-process)
- [4. CPU profilers (which function is slow?)](#4-cpu-profilers-which-function-is-slow)
- [5. Memory profilers (who allocates, what leaks?)](#5-memory-profilers-who-allocates-what-leaks)
- [6. In-code profiling: crates and std](#6-in-code-profiling-crates-and-std)
- [7. Which tool when](#7-which-tool-when)

---

## 1. Build for profiling first

Debug builds (plain `rustc file.rs`, `cargo build`, and `concepts/rs.sh`) are not
optimised and add extra checks, so they can be 10–50× slower. Numbers from them are
misleading. Profile an **optimised build that still has debug symbols**, so profilers
can show function names and line numbers.

**With rustc** (single files like `concepts/`):

```bash
rustc -O -g --edition=2024 my_prog.rs -o target/my_prog
#      │  └─ -g: debug symbols (function names in profiles)
#      └─ -O: optimise (like --release)
```

**With Cargo**, add a profile to `Cargo.toml`:

```toml
[profile.profiling]
inherits = "release"   # same optimisations as --release
debug = true           # plus debug symbols
```

```bash
cargo build --profile profiling     # binary in target/profiling/
```

**Better call stacks (optional):** some sampling profilers walk the stack using frame
pointers. Keeping them makes stacks more complete, at a small speed cost:

```bash
RUSTFLAGS="-C force-frame-pointers=yes" cargo build --profile profiling
```

---

## 2. Quick numbers: time and memory of one run

### `/usr/bin/time` (built in)

```bash
# macOS
/usr/bin/time -l ./target/my_prog

# Linux
/usr/bin/time -v ./target/my_prog
```

Use the full path. Plain `time` is the shell built-in and only shows timing.

Example output on macOS:

```
        0.00 real         0.00 user         0.00 sys
             1736704  maximum resident set size
            17472914  instructions retired
             1098040  peak memory footprint
```

| Field | Meaning |
|---|---|
| `real` | Wall-clock time from start to exit |
| `user` | CPU time spent running your code |
| `sys` | CPU time the OS spent on your behalf (I/O, memory, threads) |
| `maximum resident set size` | Peak RAM actually used, in bytes (KB on Linux) |
| `peak memory footprint` | macOS's view of peak memory, closer to what Activity Monitor shows |

If `user + sys` is larger than `real`, the program used more than one CPU core
(threads).

### `hyperfine`: compare runs reliably

One run is noisy. `hyperfine` runs the program many times and reports mean and spread.

```bash
brew install hyperfine        # or: cargo install hyperfine
hyperfine './target/my_prog'
hyperfine './target/debug_build' './target/release_build'   # compare two
```

---

## 3. Watching a running process

For programs that keep running (servers, long jobs). First find the process ID:

```bash
pgrep my_prog            # prints the PID
```

| Tool | Command | Shows |
|---|---|---|
| `top` | `top -pid <PID>` (macOS) / `top -p <PID>` (Linux) | Live CPU % and memory |
| `ps` | `ps -o pid,%cpu,%mem,rss,command -p <PID>` | One snapshot (`rss` in KB) |
| `htop` / `btop` | `htop` (install with brew/apt) | Nicer live view, per-thread |
| Activity Monitor | macOS GUI app | CPU, memory, threads; "Sample Process" button |

### macOS built-ins for a running process

These need no install and attach to a process that is already running:

```bash
sample <PID> 5           # sample the call stacks for 5 seconds: quick CPU profile
vmmap --summary <PID>    # memory map: heap, stack, libraries, totals
footprint <PID>          # memory footprint broken down by category
heap <PID>               # what is on the heap, grouped by size
leaks <PID>              # look for leaked memory
```

`sample` is the fastest way to answer "what is this stuck/busy process doing right
now?". It prints a call tree; the functions with the highest counts are the hot ones.

---

## 4. CPU profilers (which function is slow?)

A **sampling profiler** pauses the program many times per second and records which
function is running. Functions that show up in many samples are where time goes.
Results are usually shown as a **flame graph**: each bar is a function, its width is
how much time was spent in it (including the functions it called).

### samply (macOS and Linux, easiest)

```bash
cargo install --locked samply

samply record ./target/profiling/my_prog      # launch and profile
samply record -p <PID>                        # attach to a running process
```

It opens the Firefox Profiler in your browser with a flame graph, call tree and
timeline per thread. On macOS, attaching to a running process needs a one-time
`samply setup`.

### cargo-flamegraph

```bash
cargo install flamegraph

cargo flamegraph --profile profiling --bin my_prog   # in a Cargo project
flamegraph -- ./target/my_prog                       # any binary
flamegraph --pid <PID>                               # attach to running process
```

Writes `flamegraph.svg`; open it in a browser. Uses `dtrace` on macOS (needs `sudo`)
and `perf` on Linux.

### Instruments (macOS, comes with Xcode)

GUI: open Instruments, pick **Time Profiler**, choose the binary or a running process.

Command line:

```bash
xcrun xctrace record --template 'Time Profiler' --launch -- ./target/my_prog
xcrun xctrace record --template 'Time Profiler' --attach <PID>
open *.trace
```

Or through Cargo:

```bash
cargo install cargo-instruments
cargo instruments -t time --profile profiling     # CPU
cargo instruments -t alloc --profile profiling    # allocations
```

### perf (Linux)

```bash
perf record -g ./target/my_prog     # record with call stacks
perf report                         # interactive report
perf top -p <PID>                   # live view of a running process
perf stat ./target/my_prog          # counters: cycles, instructions, cache misses
```

---

## 5. Memory profilers (who allocates, what leaks?)

`/usr/bin/time` tells you the peak; these tell you **where** the memory comes from.

| Tool | Platform | Use |
|---|---|---|
| Instruments → **Allocations** | macOS | Every allocation with its call stack, over time |
| Instruments → **Leaks** | macOS | Memory that is never freed |
| `leaks --atExit -- ./target/my_prog` | macOS | Leak report when the program exits |
| `heaptrack ./target/my_prog` | Linux | Allocations over time, by call stack; GUI with `heaptrack_gui` |
| `valgrind --tool=massif ./target/my_prog` | Linux | Heap usage snapshots; view with `ms_print` |
| `dhat` crate | Any | In-code heap profiling, see [section 6](#dhat-heap-profiling) |

Safe Rust rarely leaks, but it can: `Rc`/`Arc` cycles, `Box::leak`, `mem::forget`, or
collections that only grow.

---

## 6. In-code profiling: crates and std

These go **inside** the program. The crates need a Cargo project (`cargo new`,
`cargo add <crate>`); they can't be used with plain `rustc` like the files in
`concepts/`.

### `std::time::Instant`: time a block (no crate)

```rust
use std::time::Instant;

let start = Instant::now();
// ... work ...
println!("took {:?}", start.elapsed());   // e.g. "took 12.3ms"
```

The simplest tool. Good for "how long does this one step take?".

### Benchmarking: `criterion` and `divan`

For measuring small functions precisely. They run the code many times, handle warm-up,
and report statistics.

**criterion** (most common):

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_bench"
harness = false
```

```rust
// benches/my_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn fib(n: u64) -> u64 { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }

fn bench(c: &mut Criterion) {
    // black_box stops the compiler from optimising the call away.
    c.bench_function("fib 20", |b| b.iter(|| fib(black_box(20))));
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

```bash
cargo bench      # HTML report in target/criterion/
```

**divan** is a newer alternative with simpler syntax (`#[divan::bench]` on a function)
and built-in allocation counting.

### `dhat`: heap profiling

Records every heap allocation: how many, how big, and from which line.

```toml
[dependencies]
dhat = "0.3"
```

```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;   // route all allocations through dhat

fn main() {
    let _profiler = dhat::Profiler::new_heap();   // profiling runs until this is dropped
    // ... program ...
}   // writes dhat-heap.json here
```

Prints a summary (total bytes, peak, number of allocations) and writes
`dhat-heap.json`. Open it in the online DHAT viewer to see allocations by call stack.
Slows the program down; use it in a separate build or behind a feature flag.

### `pprof`: CPU flame graph from inside the program

```toml
[dependencies]
pprof = { version = "0.14", features = ["flamegraph"] }
```

```rust
let guard = pprof::ProfilerGuard::new(100).unwrap();   // sample 100 times/second
// ... work to profile ...
if let Ok(report) = guard.report().build() {
    let file = std::fs::File::create("flamegraph.svg").unwrap();
    report.flamegraph(file).unwrap();
}
```

Useful when you can't run an external profiler (e.g. inside a container) or want to
profile only one part of the program. Works on Linux and macOS.

### `tracing` + a viewer: timeline of what happened

`tracing` records **spans** (named sections with a start and end) so you can see a
timeline of where time went, per thread.

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-chrome = "0.7"
```

```rust
use tracing::instrument;

#[instrument]                 // records a span every time this function runs
fn load_data(n: u32) { /* ... */ }

fn main() {
    use tracing_subscriber::prelude::*;
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    load_data(10);
}   // _guard dropped -> writes trace-<timestamp>.json
```

Open the JSON in [Perfetto](https://ui.perfetto.dev) or `chrome://tracing`.

Other viewers for the same `tracing` spans:
- **Tracy** (`tracing-tracy` crate): real-time profiler GUI, very detailed.
- **puffin**: lightweight in-app profiler, popular in games and GUI apps
  (`puffin::profile_function!()` macro, viewer `puffin_viewer`).
- **tokio-console**: for async programs using Tokio; shows each task, how long it
  ran, and how long it waited.

### Reading memory/CPU numbers from inside the program

| Crate | What it gives you |
|---|---|
| `memory-stats` | Current physical and virtual memory of this process: `memory_stats::memory_stats()` |
| `sysinfo` | CPU %, memory, and more for this process or any process on the system |
| `stats_alloc` | Wraps the allocator and counts allocations, frees and bytes between two points |

```rust
// memory-stats example
if let Some(usage) = memory_stats::memory_stats() {
    println!("RAM: {} bytes", usage.physical_mem);
}
```

Useful for logging memory over time, or exposing it as a metric in a long-running
service.

### Alternative allocator with profiling: jemalloc

`tikv-jemallocator` replaces the default allocator with jemalloc, which is often
faster for multi-threaded programs and has built-in heap profiling (enabled with its
`profiling` feature and the `MALLOC_CONF` environment variable). Mostly used on Linux
servers.

---

## 7. Which tool when

| Question | Start with |
|---|---|
| How long does it take and how much RAM does it use? | `/usr/bin/time -l` (macOS) / `-v` (Linux) |
| Is version A faster than version B? | `hyperfine` |
| How long does this one function take? | `Instant::now()`, then `criterion`/`divan` |
| Which function uses the CPU? | `samply`, `cargo flamegraph`, Instruments Time Profiler |
| What is this running process doing right now? | `sample <PID>` (macOS), `perf top -p <PID>` (Linux) |
| Where does memory get allocated? | `dhat`, Instruments Allocations, `heaptrack` |
| Is memory leaking? | `leaks` (macOS), Instruments Leaks, `heaptrack` |
| What happened when, across threads/tasks? | `tracing` + Perfetto or Tracy, `tokio-console` |
| Log memory usage from the app itself | `memory-stats`, `sysinfo` |

Rule of thumb: **measure first** (time / hyperfine), **then profile** (samply / dhat)
to find the cause, **then benchmark** (criterion) the fix.
