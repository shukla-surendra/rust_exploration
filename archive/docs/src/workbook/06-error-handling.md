# 6. Error Handling

**What this replaces:** Python's `try`/`except` and Java's checked/unchecked
exceptions. Rust has **no exceptions at all** — every fallible operation
returns a value (`Result<T, E>`) that you're statically forced to
acknowledge, rather than an error that silently propagates up the call
stack until something happens to `catch` it.

This is covered in full depth already — this chapter is a fast recap;
follow the links for the real explanations:

- [Option, Result & unwrap_or_else](../foundation/option-result.md) —
  `Option`/`Result`, the whole `unwrap`/`unwrap_or`/`unwrap_or_else`
  family, `Some`/`.ok_or()`, `println!` vs `eprintln!`
- [Error Handling in main.rs](../foundation/error-handling.md) — a full
  walkthrough of real error-handling code, `match`/`if let`/combinator
  style side by side, and `todo!()`/`panic!()`
- [CLI Implementation](../foundation/cli.md) — how this plays out
  end-to-end in a real program (`rgrep`)

## The one-paragraph version

`Option<T>` = maybe a value (`Some`/`None`, no reason attached).
`Result<T, E>` = maybe a value, with a reason on failure (`Ok`/`Err`).
Neither can be used without acknowledging the "bad" case — there's no
way to accidentally treat a `None`/`Err` as if it were the value you
wanted, the way a Python `None` or a Java `null` can silently propagate
until something crashes three functions later.

## `panic!` vs `Result` — pick based on "is this a bug?"

| | Use for | Python/Java analog |
|---|---|---|
| `panic!()` / `.unwrap()` / `.expect()` | states that should be *impossible* in correct code — a real bug if reached | an uncaught exception crashing the program |
| `Result<T, E>` | expected, routine failure — bad input, missing file, network error | a caught, handled exception |

The difference from exceptions: Rust makes this choice **visible in the
type signature**. A function returning `Result<T, E>` is advertising "I
can fail, and here's what you get if I do" right in its signature — no
need to read the implementation (or hunt through a `throws` clause, or
worse, undocumented Python behavior) to know it's fallible.

## `?` — propagate an error without a `try`/`catch`

```rust
fn read_and_parse(path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;   // Err? return it immediately.
    let n: i32 = contents.trim().parse()?;             // Ok? unwrap and keep going.
    Ok(n)
}
```

`expr?` means: if `expr` is `Ok(v)`, evaluate to `v` and continue; if
it's `Err(e)`, **return `Err(e)` from the enclosing function immediately**
(running any error-type conversion via `From` along the way, so a
`std::io::Error` can become your own error type automatically if you've
implemented that conversion). This replaces both Python's implicit
exception propagation *and* the explicit re-`raise` you'd write to add
context — `?` is that same "let it bubble up" behavior, just visible at
every call site instead of invisible until a `try`/`except` catches it
somewhere far away.

`?` can only be used inside a function that itself returns
`Result`/`Option` — this is why `main` in
[Error Handling in main.rs](../foundation/error-handling.md) uses
`unwrap_or_else`/`if let` instead of `?`: `main` is where propagation
ends, not a link in the chain (unless `main` itself is declared
`fn main() -> Result<(), E>`, in which case `?` works there too).
