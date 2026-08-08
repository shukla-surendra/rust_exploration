# 9. Packaging & Deployment

**Python/Java equivalent:** shipping a `.jar` + JVM, or a Docker image
with your Python + `pip install -r requirements.txt` baked in. Rust's
version tends to be simpler at the last step (a single static binary,
often with *no* runtime dependency at all) but the path there has its
own specifics.

## The advantage you start with: a single, mostly self-contained binary

```sh
cargo build --release
file target/release/rgrep
```

Unlike Python (needs a Python interpreter + your dependencies installed
wherever it runs) or Java (needs a JVM), a Rust release binary is
statically linked against almost everything by default — it doesn't
need `cargo`, `rustc`, or your dependency crates present on the machine
that runs it. What it typically *does* still dynamically link is `libc`
(and a few other system libraries) — which is where the next section
comes in.

## Fully static binaries with `musl`

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

Building against `musl` libc instead of the default `glibc` produces a
binary with **zero** dynamic library dependencies — copy the single
file to any Linux machine (or a minimal/`scratch` Docker base image) and
run it, no compatibility concerns about the target system's glibc
version. This is the closest Rust gets to "download one file, run it
anywhere" — categorically simpler than a Python or JVM deployment story
in that specific way, though `musl`'s allocator can be slightly slower
for allocation-heavy workloads, worth knowing if you profile a
difference.

## Cross-compilation — building for a target you're not running on

```sh
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

Building on your Mac for a Linux ARM server, for instance. Cross-
compilation can get genuinely fiddly once C dependencies are involved
(anything linking against a system library needs that library available
for the *target*, not just the host) — the
[`cross`](https://github.com/cross-rs/cross) tool wraps the whole thing
in Docker containers preconfigured per target, sidestepping most of that
setup by hand.

## Docker: multi-stage builds keep the final image small

```dockerfile
# --- build stage ---
FROM rust:1-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# --- runtime stage ---
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/rgrep /usr/local/bin/rgrep
ENTRYPOINT ["rgrep"]
```

The `builder` stage has the full Rust toolchain (large — gigabytes);
the final image only copies the compiled binary out of it, so the
shipped image is tens of megabytes, not gigabytes. This is the same
multi-stage pattern used for compiled Java (`maven:...` builder →
`eclipse-temurin:...-jre` runtime, shipping just the `.jar` + a slim
JRE, not the full JDK + build tools) — Rust's version is even leaner
since there's no JRE-equivalent needed in the final image at all, just
the binary (or `FROM scratch` entirely, if you built against `musl`
above and have zero dynamic dependencies left to satisfy).

## What actually needs deciding for a given project

| Question | Drives |
|---|---|
| Does this need to run anywhere without Docker? | static `musl` build |
| Does this run in a container/orchestrator? | multi-stage Dockerfile |
| Does this target more than one architecture (x86/ARM)? | cross-compilation, or CI matrix builds (ch. 7) |
| Is this a library other Rust projects depend on, not a deployed binary? | skip this chapter — see [Documentation & Publishing](./10-documentation-and-publishing.md) instead |
