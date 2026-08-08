# Rust Refresher Workbook — how to use this

You already know Python well, and know Java/C at a basic level. You've
touched Rust on and off for about a year and a half, but it doesn't
stick between sessions. This workbook exists for exactly that: a
start-to-finish pass through the language, written *against* what you
already know, so you're learning the diff, not the whole thing from
scratch each time.

## How it's organized

Eleven short chapters, each independently skimmable, roughly in the
order concepts build on each other:

1. [Syntax & Control Flow](./01-basics-and-control-flow.md) — ~15 min, mostly familiar
2. [Ownership, Borrowing & Lifetimes](./02-ownership-borrowing-lifetimes.md) — ~40 min, **the one with no Python/Java equivalent — budget real time here**
3. [Structs, Enums & Pattern Matching](./03-structs-enums-pattern-matching.md) — ~25 min
4. [Traits & Generics](./04-traits-and-generics.md) — ~20 min
5. [Collections, Closures & Iterators](./05-collections-closures-iterators.md) — ~30 min
6. [Error Handling](./06-error-handling.md) — ~15 min
7. [Memory & Smart Pointers](./07-memory-and-smart-pointers.md) — ~25 min
8. [Concurrency](./08-concurrency.md) — ~20 min
9. [Type Conversions](./09-type-conversions.md) — ~10 min
10. [Cargo, Modules, Testing & Macros](./10-cargo-modules-testing-macros.md) — ~15 min
11. [Cheat Sheet](./11-cheat-sheet.md) — a single dense reference page, skim last or keep open while coding

That's roughly 3.5–4 hours read straight through, or comfortably a day
if you're typing the examples out as you go (recommended — the
ownership chapter especially won't stick from reading alone).

## How each chapter is written

Every chapter leads with **"what this replaces in Python/Java"** — the
mental model you already have that either transfers directly or has to
be partly unlearned. Where a topic already has a full deep-dive page
under [`foundation/`](../foundation/crates-and-modules.md) (more
examples, more "why," tied to the `rgrep` project you built), the
chapter here is deliberately short and links out — come back to this
workbook for the fast pass, go to `foundation/` when you're stuck on
specifics or want to re-derive the reasoning.

## Suggested use for a comeback session

- **Have an hour?** Read chapter 2 (ownership/borrowing/lifetimes) only
  — it's the thing that actually causes compile errors when you're
  rusty, and everything else is closer to "syntax you'll recognize on
  sight."
- **Have half a day?** Read start to finish, chapter 11 last as a
  recap/reference to keep open in a second window.
- **Mid-project and blocked on something specific?** Jump straight to
  the relevant chapter, or straight to [foundation/](../foundation/crates-and-modules.md)
  if you need the fuller explanation.
