# Rust Exploration

Welcome! This is a hands-on Rust tutorial built from working notes and
runnable example projects. It starts with the fundamentals and goes all the
way down to assembly, bare-metal, and operating-system development.

## How this book is organized

| Section | What you'll learn |
|---|---|
| **Rust Refresher Workbook** | Syntax, ownership, traits, collections, errors, concurrency, and a cheat sheet. Start here. |
| **Foundation (Deep Dives)** | The memory model, strings, collections, and error handling, explained by building a small `grep` clone. |
| **Async Rust** | Futures, runtimes, `async`/`.await`, concurrency patterns, and common pitfalls. |
| **Productionizing Rust** | Linting, testing, logging, configuration, CI/CD, build profiles, and deployment. |
| **Systems From First Principles** | How the machine underneath your Rust program actually works. |
| **OxideOS, Assembly & Embedded** | Writing an OS kernel, assembly from zero to hero, and Rust on microcontrollers. |

## Suggested paths

- **New to Rust?** Work through the *Refresher Workbook* in order, then the
  *Foundation* deep dives.
- **Already know Rust?** Go straight to *Async Rust* or *Productionizing Rust*.
- **Curious about low-level work?** Read *Systems From First Principles*, then
  *Assembly* and *OxideOS*.

## Running the examples

The source code is on
[GitHub](https://github.com/shukla-surendra/rust_exploration). Each top-level
folder (`foundation/`, `struct_traits/`, `collections_practice/`,
`use_cases/rgrep/`, …) is a standalone Cargo project:

```sh
git clone https://github.com/shukla-surendra/rust_exploration
cd rust_exploration/foundation
cargo run
```

Many code blocks in this book also have a ▶ button that runs them on the
Rust Playground.
