# 9. Type Conversions

**What this replaces:** Python's implicit numeric coercion (`1 + 1.0`
just works) and Java's implicit widening (`int` → `long` → `double`
automatically) — Rust does **none** of this. Every conversion between
different types, even "obviously safe" ones like `i32` → `i64`, must be
written explicitly. This is the same "no silent surprises" philosophy as
ownership and exhaustive `match` — it trades a little typing for
eliminating a whole class of "wait, when did that become a float" bugs.

## `as` — explicit, potentially-lossy casting

```rust
let a: i32 = 10;
let b: i64 = a as i64;      // widening — always safe
let c: f64 = a as f64;      // int to float — always safe

let big: i64 = 300;
let small: u8 = big as u8;   // 44 — SILENTLY TRUNCATES, no error, no panic
```

`as` is the blunt instrument — it always "succeeds," even when the
result is nonsense (a value too large for the target type wraps around,
same as in C). Reach for `as` for conversions you're certain are safe
(widening a small int to a bigger one, int → float for display); avoid
it for narrowing conversions where the input could plausibly be out of
range — use `TryFrom` instead.

## `From` / `Into` — infallible, safe conversions

```rust
let s: String = String::from("hello");   // From, called explicitly
let s: String = "hello".into();           // Into — same conversion, called on the source
```

`From`/`Into` are a matched pair: implementing `From<A> for B` gives you
`Into<A> for B`'s counterpart *automatically* — you only ever implement
`From`, never `Into` directly. Use these for conversions guaranteed to
succeed for *every* possible input — a `&str` can always become a
`String`, so that's `From`/`Into`; a `String` can *not* always become an
`i32` (`"hello".parse::<i32>()` — see below), so that's not.

```rust
struct Celsius(f64);
struct Fahrenheit(f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

let f: Fahrenheit = Celsius(100.0).into();
```

This is also exactly what powers `?`'s auto-conversion mentioned in
chapter 6 — if your error enum implements `From<io::Error>`, then
`std::fs::read_to_string(path)?` inside a function returning
`Result<T, YourError>` converts the `io::Error` into `YourError`
automatically at the `?`.

## `TryFrom` / `TryInto` — fallible conversions, return a `Result`

```rust
use std::convert::TryFrom;

let big: i64 = 300;
let small: Result<u8, _> = u8::try_from(big);   // Err — 300 doesn't fit in a u8
assert!(small.is_err());

let big: i64 = 100;
let small: u8 = u8::try_from(big).unwrap();      // Ok(100)
```

The safe alternative to `as` for narrowing conversions — instead of
silently wrapping/truncating, you get a `Result` you're forced to
handle (same "no silent surprises" theme as chapter 6). Prefer this over
`as` whenever the input value isn't provably in range ahead of time.

## Parsing strings — `.parse()`

```rust
let n: i32 = "42".parse().unwrap();          // turbofish form: "42".parse::<i32>().unwrap()
let n: Result<i32, _> = "not a number".parse();   // Err — parse always returns a Result
```

The Rust equivalent of Python's `int("42")`/Java's `Integer.parseInt`,
except it never throws — `.parse()` returns `Result<T, ParseIntError>`
(or the equivalent for whatever `T` is), so a malformed string is just
another `Err` to handle with everything from chapter 6, not an exception
to catch.
