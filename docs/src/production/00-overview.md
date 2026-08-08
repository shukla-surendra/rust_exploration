# Productionizing Rust: what's left after the code compiles

The [workbook](../workbook/00-how-to-use.md) and
[foundation](../foundation/crates-and-modules.md) sections cover writing
correct Rust. This section covers everything else a piece of Rust code
needs before it's a *production* program someone can depend on — the
same gap that exists in Python (writing a script vs. shipping a
service with logging, config, CI, and monitoring) or Java (a class that
compiles vs. a deployable artifact with a build pipeline), just with
Rust-specific tools filling each role.

## The checklist

| Concern | Chapter | Python/Java rough equivalent |
|---|---|---|
| Consistent style, catching bugs before runtime | [Linting & Formatting](./01-linting-and-formatting.md) | `black`/`flake8`, `checkstyle` |
| More than "does it compile" | [Testing Strategy](./02-testing-strategy.md) | `pytest` layers, JUnit + Mockito |
| Failing safely, not just failing | [Error Handling in Production](./03-error-handling-in-production.md) | structured exception hierarchies |
| Knowing what happened after deployment | [Logging & Observability](./04-logging-and-observability.md) | `logging`/`structlog`, SLF4J |
| Not hardcoding secrets and environments | [Configuration & Secrets](./05-configuration-and-secrets.md) | `python-dotenv`, Spring `application.yml` |
| Trusting your dependency tree | [Dependency Management & Security](./06-dependency-management-and-security.md) | `pip-audit`, `npm audit`, OWASP dependency-check |
| Catching regressions automatically | [CI/CD](./07-ci-cd.md) | GitHub Actions for any language |
| Making the binary actually fast/small | [Build Profiles & Performance](./08-build-profiles-and-performance.md) | JVM flags, PyPy/Cython |
| Getting the binary to where it runs | [Packaging & Deployment](./09-packaging-and-deployment.md) | Docker images, `.jar`/wheel builds |
| Other people (including future you) using this code | [Documentation & Publishing](./10-documentation-and-publishing.md) | docstrings/Sphinx, Javadoc, PyPI/Maven Central |

## Why this matters more in Rust than it might in Python

Two things in Rust's culture push this further than a typical Python
project:

- **The ecosystem expects it.** `cargo fmt`, `cargo clippy`, `cargo
  test`, `cargo doc`, and `cargo audit` are either built into the
  toolchain or a one-command install away — there's much less "which
  tool do I even pick" friction than the Python/JS tooling landscape,
  which means there's also less excuse to skip them.
- **Rust's compile-time guarantees cover correctness, not operations.**
  The borrow checker (see [Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md))
  guarantees your code won't have data races or use-after-free bugs. It
  says nothing about whether your program logs enough to debug a
  production incident, whether a dependency has a known CVE, or whether
  your Docker image is 1.5GB because you forgot a multi-stage build.
  Those are separate disciplines this section covers.

Read this section once end to end when you're about to take a personal
project ([`rgrep`](../../../use_cases/rgrep) is used as the running
example throughout) from "it works on my machine" to "I'd trust this
running unattended."
