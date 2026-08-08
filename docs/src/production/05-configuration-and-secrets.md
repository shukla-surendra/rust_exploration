# 5. Configuration & Secrets

**Python/Java equivalent:** `python-dotenv` + `os.environ`, or Spring's
`application.yml` (Java). Same underlying problem in Rust: a production
program needs settings that differ per environment (dev/staging/prod)
and secrets that must never end up in source control — neither belongs
hardcoded the way `rgrep`'s `sample_logs/*.log` paths currently are for
a learning scaffold.

## Environment variables — the baseline

```rust
let port: u16 = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse()
    .expect("PORT must be a valid number");
```

`std::env::var(name)` returns `Result<String, VarError>` — absence is
just another `Err` to handle with everything from
[Option, Result & unwrap_or_else](../foundation/option-result.md), same
as any other fallible lookup. This is the universal baseline every
deployment platform (Docker, systemd, Kubernetes, Heroku-likes)
supports without any extra tooling — a sensible default for
"configuration that's simple enough not to need a whole file."

## `.env` files for local development — `dotenvy`

```rust
fn main() {
    dotenvy::dotenv().ok();   // loads .env into the process's environment, if present
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
}
```

Directly the same role as Python's `python-dotenv`: a `.env` file (never
committed — see below) holds local secrets/config, loaded into
`std::env` at startup so the rest of the code just calls
`std::env::var` uniformly whether running locally or in a real
deployment (where the platform sets real environment variables and
there's no `.env` file at all).

**Always add `.env` to `.gitignore`** — this repo's root
[`.gitignore`](../../../.gitignore) doesn't currently exclude it, which
is fine while there's no `.env` file in use yet, but add the line the
moment one shows up in any `use_cases/*` crate.

## Structured config files — the `config` crate

Once settings outgrow a handful of env vars (nested sections, different
files per environment), the `config` crate layers multiple sources
(defaults → file → environment overrides) into one typed struct:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    port: u16,
    log_level: String,
}

let settings: Settings = config::Config::builder()
    .add_source(config::File::with_name("config/default"))
    .add_source(config::Environment::default())   // env vars override the file
    .build()?
    .try_deserialize()?;
```

This pairs `serde` (Rust's near-universal
serialize/deserialize framework — the same crate behind JSON, YAML, and
TOML handling generally) with layered sources, closest to what Spring's
`application.yml` + profile overrides gives Java, or what a Python
project might build from `pydantic-settings`.

## Secrets specifically: never hardcode, never commit

- **Never commit real secrets** — not even in a "private" repo, since
  git history keeps everything forever unless you explicitly scrub it.
  Use `.env` (gitignored) locally, and your deployment platform's secret
  store in production (environment variables injected by the
  orchestrator, a dedicated secrets manager like Vault/AWS Secrets
  Manager/Doppler).
- **`.gitignore` first, always.** Before a secret-bearing file is ever
  written to disk in a git-tracked directory, confirm the ignore rule is
  already in place — not after, since "after" means it's already in
  history.
- **For genuinely sensitive in-memory values** (API keys, tokens held
  for the process's lifetime), the `secrecy` crate wraps a value so it's
  redacted from `Debug`/logs by default and zeroed from memory on drop —
  cheap insurance against a secret accidentally ending up in a log line
  via a stray `{:?}`.

## Rule of thumb for a project this size

- **A CLI tool** (`rgrep`): plain `std::env::var` for anything that
  genuinely needs to vary by environment — most CLI config is just
  argv (see [CLI Implementation](../foundation/cli.md)), not
  environment-based, so you may not need this chapter's tools at all
  yet.
- **A long-running service**: `dotenvy` for local dev, real environment
  variables (or your platform's secret store) in production, and the
  `config` crate once you have more than ~5-6 independent settings to
  track.
