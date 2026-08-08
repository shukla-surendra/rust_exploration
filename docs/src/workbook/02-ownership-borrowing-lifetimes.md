# 2. Ownership, Borrowing & Lifetimes

**What this replaces:** nothing — this is the concept with no Python or
Java equivalent, and the one most likely to have faded between Rust
sessions. Budget real time here; everything else in the language is
comparatively familiar syntax.

## The problem both languages you know solve with a GC

Python and Java both let you hand the same object to multiple variables
freely, and a garbage collector figures out later when nothing points to
it anymore and frees it:

```python
a = [1, 2, 3]
b = a            # both names point at the same list
b.append(4)      # a sees this too — same object
```

Rust has no garbage collector. Instead, it tracks **who owns each
value** at compile time, and frees memory deterministically the instant
the owner goes out of scope — no runtime GC pause, no "when exactly does
this get freed" uncertainty. The rules that make this possible are what
this chapter covers.

## Rule 1: every value has exactly one owner

```rust
let a = String::from("hello");
let b = a;          // ownership MOVES from a to b
println!("{a}");    // COMPILE ERROR: value borrowed after move
```

This is the line that surprises everyone coming from Python. `b = a`
doesn't copy the string and doesn't create a second reference to the
same data the way Python's `b = a` does — it **transfers ownership**.
After the move, `a` is no longer valid; the compiler enforces this
statically, so there's no way to accidentally use a value that's been
"given away." Only one owner can exist at any point, and when that owner
goes out of scope, the value is dropped (freed) automatically.

**Types that don't move, they copy:** simple stack-only types (`i32`,
`f64`, `bool`, `char`, tuples of these) implement `Copy` — `let b = a;`
duplicates the value instead of moving it, and `a` stays valid. This is
why `let x = 5; let y = x; println!("{x}");` compiles fine but the
`String` version above doesn't — `i32` is `Copy`, `String` isn't (it
owns a heap allocation, and heap-owning types generally can't be
implicitly duplicated — see [Stack vs Heap](../foundation/stack-vs-heap.md)).

## Rule 2: you can borrow a value without taking ownership

Moving everything everywhere would be unworkable — you need to pass
values to functions and get them back. **References** (`&`) let you
*borrow* a value temporarily without taking ownership:

```rust
fn print_it(s: &String) {   // borrows, doesn't take ownership
    println!("{s}");
}

let a = String::from("hello");
print_it(&a);       // lend a reference
print_it(&a);        // a is still valid — can lend it again
```

Compare the version *without* `&`:

```rust
fn print_it(s: String) { println!("{s}"); }

let a = String::from("hello");
print_it(a);         // ownership moves into the function...
print_it(a);          // COMPILE ERROR: a was already moved, it's gone
```

This is the mechanical reason so many Rust function signatures take
`&String`/`&str`/`&[T]` rather than owned types — see
[Dereferencing](../foundation/dereferencing.md) for the `*`/`Deref` side
of working with references, and [CLI Implementation](../foundation/cli.md)
for `&[String]` used this exact way in `rgrep`.

## Rule 3: many readers, XOR one writer — enforced at compile time

```rust
let mut s = String::from("hello");

let r1 = &s;         // ok
let r2 = &s;          // ok — multiple immutable borrows are fine
println!("{r1} {r2}");

let r3 = &mut s;      // COMPILE ERROR if r1/r2 are still in use:
                       // cannot borrow `s` as mutable because it's also borrowed as immutable
```

At any given point, a value can have **either** any number of immutable
borrows (`&T`) **or exactly one** mutable borrow (`&mut T`) — never
both, never more than one mutable borrow. This is the rule that prevents
a whole category of bugs Python and Java both allow at runtime: one part
of your code mutating a collection while another part is iterating over
it (Python: `RuntimeError: dictionary changed size during iteration`,
caught at runtime, if you're lucky enough to hit it; Rust: caught before
the program ever runs).

```rust
let mut v = vec![1, 2, 3];
for x in &v {
    v.push(*x);   // COMPILE ERROR — can't mutate v while r (the iterator) borrows it
}
```

## What "the borrow checker" actually is

There's no separate tool — it's part of the compiler. When you hit a
message like `cannot borrow ... as mutable more than once`, that's the
compiler statically proving your code would otherwise violate rule 3.
The unfamiliar part isn't the rule itself (both Python and Java code
*try* to avoid "mutate while iterating" bugs already, by convention) —
it's that Rust **refuses to compile** rather than letting you find out
at runtime.

### Reading a borrow-checker error

```
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
  --> src/main.rs:5:14
   |
3  |     let r1 = &s;
   |              -- immutable borrow occurs here
4  |     println!("{r1}");
5  |     let r3 = &mut s;
   |              ^^^^^^ mutable borrow occurs here
```

Read bottom to top: it names the conflicting borrow, points at exactly
where each one starts, and (usually) suggests a fix. The fix is almost
always "shorten one borrow's lifetime" — e.g. move the `println!` after
the `let r3` line isn't possible here, but ending `r1`/`r2`'s usage
before line 5 (which the compiler tracks automatically based on last
use, not lexical scope — see "non-lexical lifetimes" below) resolves it.

## Lifetimes — naming *how long* a reference is valid

A reference can never outlive the value it points to (an obvious rule —
the alternative is a dangling pointer). Most of the time the compiler
infers this without you writing anything. Occasionally, usually in
function signatures returning a reference, it needs help:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a` (read "tick-a," or "lifetime a") isn't a runtime value — it's a
compile-time label saying "the reference this function returns lives no
longer than the *shorter* of `x` and `y`'s lifetimes." Without it, the
compiler can't tell which input the output reference is tied to (could
be either `x` or `y`), and can't verify the caller won't use the result
after one of the inputs is gone. You already have this in the codebase:

```rust
pub fn search<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    // ...
}
```

`search`'s `'a` says "every `&str` in the returned `Vec` borrows directly
from `contents`, and is only valid as long as `contents` is" — see
[Dereferencing](../foundation/dereferencing.md) for `search`'s full
context in `rgrep`. Note `pattern` has no lifetime annotation needed —
it isn't part of what gets returned, so there's nothing to tie it to.

**Lifetime elision** — the common cases don't need `'a` written out at
all. `fn first_word(s: &str) -> &str` compiles with no annotations
because the compiler applies a rule: "one input reference, one output
reference → assume they're tied together." You mostly only write
explicit lifetimes when there are multiple input references and the
compiler can't guess which one the output relates to (like `longest`
above).

## Non-lexical lifetimes: borrows end at last use, not end of scope

```rust
let mut s = String::from("hello");
let r1 = &s;
println!("{r1}");      // last use of r1 — its borrow effectively ends HERE
let r3 = &mut s;        // fine — r1's borrow is already over
```

Modern Rust (since the 2018 edition) tracks a borrow's actual last use,
not its enclosing `{ }` block — this is why the example above compiles
even though `r1` is lexically still "in scope" when `r3` is created.
This is a frequent source of "wait, I thought this would error" pleasant
surprises once you're used to the older, stricter mental model.

## The payoff: why bother with all this

- **No garbage collector** — no GC pause, ever, and memory is freed the
  instant an owner's scope ends, deterministically (see `Drop` in
  [Traits](../foundation/traits.md)).
- **No data races, at compile time** — the "many readers XOR one writer"
  rule is exactly what prevents two threads from mutating the same data
  simultaneously without synchronization; see
  [Concurrency](./08-concurrency.md), where the same rule is enforced
  across threads via `Send`/`Sync`, not just within one.
- **No use-after-free, no double-free, no null-pointer dereference** —
  categories of bugs that are runtime crashes (or worse, silent
  corruption) in C, and that Python/Java avoid via GC + runtime checks
  (`NullPointerException`) instead of preventing at compile time.
