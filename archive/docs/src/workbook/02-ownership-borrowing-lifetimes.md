# 2. Ownership, Borrowing & Lifetimes

No Python/Java equivalent — this is the concept most likely to have faded
between sessions, so budget real time here. 14 rules, 4 small groups. Scan
the table, then read only the cards for rules you're shaky on. Each card
is self-contained: **rule → code → why** — you don't need the others open
to understand one.

## The mental model

Python/Java: a garbage collector frees memory whenever nothing points to
it anymore — you don't track this yourself.

```python
a = [1, 2, 3]
b = a            # both names point at the same list
b.append(4)      # a sees this too
```

Rust: no GC. Instead, every value has exactly one **owner**, tracked at
*compile time*, and it's freed the instant that owner goes out of scope.
The 14 rules below are just this one idea, worked out into every corner
it touches.

## Quick reference

| # | Ownership | | # | Borrowing |
|---|---|---|---|---|
| 1 | Every value has exactly one owner | | 6 | `&T`/`&mut T` use a value without owning it |
| 2 | Owner out of scope → value dropped | | 7 | Any number of `&T`, **or** one `&mut T` — never both |
| 3 | Moving a non-`Copy` value invalidates the old binding | | 8 | A borrow ends at last use, not `{ }` end (NLL) |
| 4 | `Copy` types duplicate instead of moving | | 9 | Slices (`&[T]`, `&str`) borrow *part* of a collection |
| 5 | Moving one field = partial move; other fields still usable | | | |

| # | Lifetimes | | # | Escape valve |
|---|---|---|---|---|
| 10 | A reference can't outlive its value | | 14 | Rules 1 & 7 too strict? `Rc`/`Arc` (shared owner), `RefCell`/`Cell` (check moves to runtime) |
| 11 | Lifetimes are compile-time labels, mostly inferred | | | |
| 12 | A struct holding a reference needs a lifetime param | | | |
| 13 | `'static` = valid for whole program | | | |

---

## Ownership (1–5)

### Rules 1–2: one owner, freed on scope exit

**Assigning a non-`Copy` value moves ownership; the old binding dies.**

```rust
let a = String::from("hello");
let b = a;          // ownership MOVES a → b
println!("{a}");    // COMPILE ERROR: value borrowed after move
```

**Why:** unlike Python's `b = a`, this isn't a second reference to the
same object — `a` is gone. When `b` later goes out of scope, Rust frees
the string automatically. No GC, no `free()`.

### Rule 3: passing into a function moves too

```rust
fn print_it(s: String) { println!("{s}"); }

let a = String::from("hello");
print_it(a);   // moves in
print_it(a);   // COMPILE ERROR: a was already moved
```

### Rule 4: `Copy` types duplicate instead

```rust
let x = 5;
let y = x;
println!("{x}");   // fine — i32 is Copy
```

**Why:** plain stack data (`i32`, `f64`, `bool`, `char`, tuples of these)
has no heap pointer to worry about sharing, so it's cheap to duplicate
implicitly. `String` owns a heap allocation, so it isn't `Copy` —
`.clone()` (`Clone` trait) is the *explicit* opt-in version for types too
expensive to duplicate silently.
→ [Stack vs Heap](../foundation/stack-vs-heap.md), [Traits](../foundation/traits.md)

### Rule 5: partial moves

```rust
struct Pair { a: String, b: String }

let p = Pair { a: String::from("x"), b: String::from("y") };
let a = p.a;              // moves just `a` out
println!("{}", p.b);      // fine — b untouched
println!("{}", p.a);      // COMPILE ERROR — p.a already moved
```

**Why:** the compiler tracks moves per field, but only while you hold the
owned value directly — through `&self` you don't get this, use
`Some(ref mut n)` instead.
→ [Structs & Methods](../foundation/structs-and-methods.md#self-referential-structs-optionboxself)

## Borrowing (6–9)

### Rule 6: borrow instead of move

```rust
fn print_it(s: &String) { println!("{s}"); }  // borrows

let a = String::from("hello");
print_it(&a);   // lend
print_it(&a);   // a still valid — lend again
```

**Why:** this is why most Rust signatures take `&String`/`&str`/`&[T]`
rather than owned types. → [Dereferencing](../foundation/dereferencing.md)

### Rule 7: many readers, XOR one writer

```rust
let mut s = String::from("hello");
let r1 = &s;          // ok
let r2 = &s;          // ok — many immutable borrows fine
println!("{r1} {r2}");
let r3 = &mut s;       // COMPILE ERROR while r1/r2 are still live
```

**Why:** this rule *is* "the borrow checker" — not a separate tool, just
the compiler proving this invariant statically. Its errors name the
conflicting borrow and point at where each one starts; the fix is almost
always "shorten one borrow's lifetime."

### Rule 8: a borrow ends at last use (NLL)

```rust
let mut s = String::from("hello");
let r1 = &s;
println!("{r1}");      // last use — r1's borrow ends HERE
let r3 = &mut s;        // fine — r1 is already done
```

**Why:** the compiler tracks *actual last use*, not the enclosing `{ }`
block — this is why the example above compiles.

### Rule 9: slices borrow part of a collection

```rust
let v = vec![1, 2, 3, 4, 5];
let middle: &[i32] = &v[1..3];   // borrows elements 1..3, no copy
```

**Why:** `&str`/`&[T]` are "fat pointers" (pointer + length) — same idea
as Rule 6, applied to a range instead of a whole value.
→ [CLI Implementation](../foundation/cli.md), [Strings](../foundation/strings.md)

## Lifetimes (10–13)

### Rule 10: a reference can't outlive its value

The obvious one — the alternative is a dangling pointer. The compiler
enforces it without you writing anything, most of the time.

### Rule 11: lifetimes are compile-time labels, mostly inferred

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**Why:** `'a` isn't a runtime value — it's a label meaning "the returned
reference lives no longer than the shorter of `x`, `y`." **Elision**
handles the common case for free: `fn first_word(s: &str) -> &str` needs
no annotation, because "one input ref, one output ref → assume they're
tied" is already a rule the compiler applies. You write `'a` explicitly
only when there are multiple input refs and the compiler can't guess
which one the output relates to, as above.

### Rule 12: a struct holding a reference needs a lifetime param

```rust
struct Excerpt<'a> { part: &'a str }

let novel = String::from("Call me Ishmael...");
let first_sentence = novel.split('.').next().unwrap();
let excerpt = Excerpt { part: first_sentence };
// excerpt can't outlive novel — Rule 10, one level up
```

**Why:** without `'a`, Rust can't tell how long the reference stored
inside the struct needs to stay valid, and refuses to compile.

### Rule 13: `'static` — valid for the whole program

```rust
let s: &'static str = "hello";   // string literals are 'static
```

**Why:** literals are compiled into the binary, not allocated at
runtime — nothing can free them out from under the reference. Also shows
up in trait-object bounds (`Box<dyn Error + 'static>`) and
`thread::spawn`, which requires `'static` because a spawned thread might
outlive the function that started it. → [Concurrency](./08-concurrency.md)

## The escape valve (14)

Rules 1 & 7 (one owner, one writer XOR many readers) are sometimes
genuinely too strict — a graph with shared child nodes, a cache read from
multiple places. Rust makes you opt in explicitly rather than relaxing
the rule:

- `Rc<T>` / `Arc<T>` — shared ownership
- `RefCell<T>` / `Cell<T>` — moves the borrow check from compile time to
  *runtime* (panics instead of refusing to compile)

→ full treatment + `Rc<RefCell<T>>` decision table:
[Memory & Smart Pointers](./07-memory-and-smart-pointers.md)

## Why bother

- **No GC pause** — memory frees deterministically at scope exit
  (`Drop`, → [Traits](../foundation/traits.md))
- **No data races, at compile time** — Rule 7, enforced across threads
  too via `Send`/`Sync` (→ [Concurrency](./08-concurrency.md))
- **No use-after-free / double-free / null deref** — runtime crashes in
  C, GC + `NullPointerException` band-aids in Python/Java, compile errors
  here
