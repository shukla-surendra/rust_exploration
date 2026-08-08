# 12. Kubernetes & Infrastructure

[Packaging & Deployment](./09-packaging-and-deployment.md) covers
building the Docker image. This chapter covers what changes once that
image runs inside Kubernetes (or similar orchestration) instead of as a
single container — mostly *not* Rust-specific, but with a few points
where Rust's characteristics (tiny binaries, fast startup, no GC)
genuinely change the calculus versus deploying a Python/Java service the
same way.

## The Deployment — nothing Rust-specific, but worth seeing once

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rgrep-service
spec:
  replicas: 3
  selector:
    matchLabels: { app: rgrep-service }
  template:
    metadata:
      labels: { app: rgrep-service }
    spec:
      containers:
        - name: rgrep-service
          image: my-registry/rgrep-service:1.2.3
          resources:
            requests: { cpu: "50m", memory: "32Mi" }
            limits: { cpu: "200m", memory: "64Mi" }
          ports:
            - containerPort: 8080
```

The `resources` block is the first place Rust's profile genuinely
differs from a Python/Java equivalent: a Rust service with no GC and no
interpreter typically runs comfortably in tens of megabytes of memory
where the equivalent Python (interpreter + libraries loaded) or Java
(JVM heap + metaspace) service might request ten times that. Worth
setting these *from measurement* (see
[Measuring Performance](../foundation/measuring-performance.md)), not
guessing, but the ceiling is genuinely lower — a real cost/density
advantage in a cluster running many replicas.

`cpu: "50m"` is **millicpu** — 50m means 1/20th of one CPU core, not a
whole core running slower. See
[Cores, Threads, vCPUs & Fractional CPU](../cpu-division.md) for
exactly what that number controls (Linux cgroups CPU quota/period —
the same mechanism `docker run --cpus=0.5` and AWS Fargate's fractional
vCPU billing both use underneath) and why it's time-slicing, not a
smaller unit of silicon.

## Health checks — wiring up what `/health`/`/ready` need from your code

```yaml
          livenessProbe:
            httpGet: { path: /healthz, port: 8080 }
            initialDelaySeconds: 2      # Rust services start fast — no JVM warm-up to wait out
            periodSeconds: 10
          readinessProbe:
            httpGet: { path: /readyz, port: 8080 }
```

`livenessProbe` asks "is this process still functioning" (restart the
pod if not); `readinessProbe` asks "should traffic be routed here right
now" (pull out of rotation without restarting — useful during a slow
dependency-connection retry, for instance). These endpoints are just
ordinary handlers in your service, returning 200/503 based on real
internal state — nothing Kubernetes-specific in the Rust code itself,
but they're exactly the kind of thing to log through
[Logging & Observability](./04-logging-and-observability.md) when they
flip, since a flapping readiness probe with no log trail is a common
"why did this pod keep restarting" mystery.

**`initialDelaySeconds` can be set aggressively low for a Rust service**
compared to defaults copied from a Java/Python deployment template — no
JVM class-loading or Python module-import startup cost to wait out, so
a Rust binary is typically ready to serve traffic within milliseconds of
starting, not seconds.

## Config & secrets — the Kubernetes side of chapter 5

```yaml
          envFrom:
            - configMapRef: { name: rgrep-config }
            - secretRef: { name: rgrep-secrets }
```

A `ConfigMap`/`Secret` injected as environment variables is exactly
what [Configuration & Secrets](./05-configuration-and-secrets.md)'s
`std::env::var` calls read on the Rust side — no special Kubernetes
crate needed for the common case, since env vars are env vars regardless
of who set them. Reach for the `kube` crate specifically only if your
Rust program needs to *talk to the Kubernetes API itself* (a controller
or operator — see below), not just read the config it was handed.

## Graceful shutdown — SIGTERM handling

Kubernetes sends `SIGTERM` before killing a pod (giving it a grace
period to finish in-flight work) — a Rust service should actually listen
for it:

```rust
use tokio::signal;

#[tokio::main]
async fn main() {
    // ... start server ...
    signal::ctrl_c().await.ok();   // or tokio::signal::unix::signal(SIGTERM) on Linux
    println!("shutting down gracefully");
    // stop accepting new work, finish in-flight requests, then exit
}
```

Skipping this isn't fatal (Kubernetes force-kills after the grace period
regardless), but it's the difference between requests in flight during a
rolling deploy completing cleanly versus being cut off mid-response —
the same concern a Java service handles via a shutdown hook
(`Runtime.getRuntime().addShutdownHook`) or a Python service via signal
handlers in `asyncio`.

## Building an actual Kubernetes controller/operator in Rust

If the Rust program needs to *watch and react to* Kubernetes resources
(not just run as an ordinary workload) — the **`kube`** crate is the
Rust equivalent of Python's `kubernetes` client library:

```rust
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Pod;

let client = Client::try_default().await?;
let pods: Api<Pod> = Api::namespaced(client, "default");
for p in pods.list(&Default::default()).await? {
    println!("{}", p.metadata.name.unwrap_or_default());
}
```

The [`kube-rs`](https://kube.rs) project (`kube` + `kube-runtime`) is
mature enough to be a real alternative to writing a controller in Go
with `client-go` (the ecosystem's usual default) — worth knowing this
exists if a project ever needs a custom operator, though it's a
meaningfully bigger undertaking than deploying a Rust service *as* an
ordinary workload, which is by far the more common need.

## Other infrastructure tooling worth knowing about

Rust isn't usually the language you write your *infrastructure
definitions* in (Terraform's HCL, or Pulumi in Python/TypeScript, remain
the standard choices — there's no reason to introduce Rust there just
because the application is Rust), but a few places Rust shows up in the
broader infra tooling landscape are worth recognizing by name if you
encounter them:

| Tool | What it is | Why it's Rust |
|---|---|---|
| `cross` | cross-compilation via Docker (ch. 9) | build tooling, not infra-as-code |
| `nextest` | a faster `cargo test` runner, common in CI | speeds up ch. 7's test step directly |
| `bottlerocket` | AWS's container-optimized Linux, written in Rust | the *host OS* your containers might run on, not something you write |
| Deno/some edge-compute platforms | run WASM built from Rust at the edge | a deployment target, if you compile to `wasm32` instead of a native binary |

None of these are things you need to reach for to deploy `rgrep` or a
typical service — they're here so the names are recognizable if they
come up, not as a checklist to work through.

## Rule of thumb for a project this size

- **A CLI tool** (`rgrep`): none of this chapter applies — it's not a
  long-running service, so there's nothing to health-check or
  gracefully shut down.
- **A service deployed to k8s**: the `Deployment` + health-check +
  config/secret wiring above is the real minimum; graceful shutdown is
  cheap to add and worth doing from the start rather than retrofitting
  after a rolling deploy drops requests.
- **A Kubernetes controller/operator**: a distinct, larger undertaking —
  reach for `kube-rs`'s own documentation once you're actually there,
  this section is only meant to tell you the option exists.
