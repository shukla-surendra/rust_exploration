# 4. Logging & Observability

**Python/Java equivalent:** the `logging` module / `structlog`
(Python), SLF4J + Logback (Java). `println!`/`eprintln!` (see
[Option, Result & unwrap_or_else](../foundation/option-result.md)) are
fine for a learning scaffold or a tiny CLI; anything that runs
unattended — a server, a background job, a scheduled task — needs
actual logging: leveled, filterable, and structured enough to search
later.

## Why `println!` stops being enough

- **No levels.** You can't turn off noisy debug output without editing
  code and recompiling.
- **No structure.** `println!("user {} did {}", id, action)` produces
  text a human can read but a log aggregator (Datadog, CloudWatch, an
  ELK stack) has to regex-parse instead of query directly.
- **No context propagation.** In a real service handling concurrent
  requests, you need to know *which* request a given log line belongs
  to — plain `println!` gives you no way to tag that automatically.

## `log` — the standard facade

```rust
use log::{info, warn, error, debug};

info!("server starting on port {port}");
warn!("cache miss for key {key}, falling back to db");
error!("failed to connect to db: {e}");
```

`log` defines the macros and levels (`error!`/`warn!`/`info!`/`debug!`/
`trace!`) but does **not** decide where log lines go — that's a separate
"logger implementation" crate you plug in once, at the very start of
`main` (e.g. `env_logger`, which reads a `RUST_LOG=debug` environment
variable to control verbosity without recompiling — directly analogous
to Python's `logging.basicConfig(level=...)`).

```rust
fn main() {
    env_logger::init();   // reads RUST_LOG env var
    log::info!("started");
}
```

## `tracing` — structured, and built for concurrent/async code

`tracing` is `log`'s more capable successor, standard in most modern
Rust services (especially anything async — see it paired with `tokio`
in real-world servers):

```rust
use tracing::{info, instrument};

#[instrument]                          // auto-logs entry/exit + arguments
fn search(pattern: &str, path: &str) -> usize {
    info!(pattern, path, "starting search");
    let count = 42;
    info!(count, "search complete");
    count
}
```

Two things `log` doesn't give you:

- **Structured fields** (`info!(pattern, path, "...")`) — logged as
  actual key-value data, not just interpolated into a string, so a log
  backend can filter/aggregate on `pattern` directly instead of
  regex-extracting it from prose.
- **Spans** — a `#[instrument]`'d function automatically tags every log
  line inside it (and inside anything it calls) with which invocation
  it belongs to. This is the piece that matters once you have concurrent
  work happening (multiple requests, multiple threads/tasks) — it's how
  you answer "show me every log line for *this specific* request,"
  the same problem a request-ID/correlation-ID convention solves by
  hand in Python/Java logging setups.

## Metrics — counters and gauges, not just text

Logging answers "what happened." Metrics answer "how much/how often,"
cheaply enough to record for every single request rather than just
notable events. The `metrics` crate (with a backend like
`metrics-exporter-prometheus`) is the common pairing:

```rust
metrics::counter!("requests_total").increment(1);
metrics::histogram!("request_duration_seconds").record(elapsed.as_secs_f64());
```

Directly analogous to a Prometheus client library in Python/Java (both
ecosystems have one) — same idea, same eventual destination (a
Prometheus-compatible `/metrics` endpoint a monitoring system scrapes).

## Rule of thumb for a project this size

- **Personal CLI tool** (like `rgrep` as it stands): `eprintln!` for
  errors is genuinely fine — see
  [CLI Implementation](../foundation/cli.md#reporting-errors-stderr-not-stdout).
- **Anything you'd run unattended or hand to someone else**: reach for
  `log` + `env_logger` at minimum — cheap to add, immediately gives you
  levels and an environment-variable-controlled verbosity knob.
- **A concurrent service**: `tracing`, so you can actually correlate log
  lines across simultaneous requests — plain `log` starts to feel
  inadequate the moment two things can be happening at once.
