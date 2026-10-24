# 7. Memory & Smart Pointers

**What this replaces:** Python and Java both put nearly everything on
the heap and garbage-collect it — see
[Stack vs Heap](../foundation/stack-vs-heap.md) for exactly why. This
chapter recaps the stack/heap/`Box` material fast, then covers `Rc`,
`Arc`, and `RefCell`/`Cell` — the tools for the cases ownership's "one
owner" rule (chapter 2) is genuinely too strict for, which don't come up
until you leave beginner territory.

## Recap: `Box<T>` — already covered in depth

- [Stack vs Heap](../foundation/stack-vs-heap.md) — why Rust defaults to
  the stack at all
- [Sized vs Unsized](../foundation/sized-vs-unsized.md) — the concrete
  scenarios (`dyn Trait`, recursive types, `str`/`[T]`) that force you
  onto the heap
- [Dereferencing](../foundation/dereferencing.md) — how `*box_val` and
  deref coercion work

One line: `Box<T>` is a single heap allocation with a single owner —
same ownership rules as any other value, just backed by heap memory
instead of the stack. Reach for it for recursive types, trait objects,
or moving large values cheaply.

## The gap `Box` doesn't cover: shared ownership

Ownership's rule 1 (chapter 2) says *exactly one* owner. Sometimes that's
genuinely too strict — a graph where multiple nodes need to reference a
shared node, a cache multiple parts of a program need read access to,
a tree where a child needs to know about its parent. Python/Java don't
have this problem (the GC just keeps anything with a live reference
alive, however many references exist) — Rust needs an explicit type for
"more than one owner, freed when the *last* one goes away."

### `Rc<T>` — reference-counted shared ownership (single-threaded)

```rust
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);        // NOT a deep copy — bumps a reference count
let c = a.clone();             // same thing, `.clone()` method form

println!("{}", Rc::strong_count(&a));   // 3
// value is only actually freed once all of a, b, c are dropped
```

`Rc::clone` is cheap — it increments an internal counter, doesn't copy
the underlying data. This is conceptually exactly what Python does for
*every* object (reference counting is literally how CPython manages
memory) — `Rc<T>` is you opting into that specific model for one value,
rather than it being true of everything by default the way it is in
Python.

`Rc<T>` only gives you **shared, immutable** access — `Rc<T>` alone
doesn't let you mutate the inner value (multiple owners + mutation is
exactly what borrowing's rule 3 forbids). For that, pair it with:

### `RefCell<T>` — move the borrow check to runtime

```rust
use std::cell::RefCell;

let cell = RefCell::new(5);
*cell.borrow_mut() += 1;             // runtime-checked mutable borrow
println!("{}", cell.borrow());        // runtime-checked immutable borrow
```

Normally, "one writer XOR many readers" (chapter 2) is enforced by the
compiler at compile time. `RefCell<T>` moves that same rule to **runtime**
instead — `.borrow()`/`.borrow_mut()` panic if you violate it (e.g. two
active mutable borrows at once), rather than refusing to compile. This
is "interior mutability": a way to mutate something through a shared
(`&`) reference, which normally isn't allowed. `Cell<T>` is `RefCell`'s
simpler sibling for `Copy` types — no borrow-tracking overhead, just
`.get()`/`.set()`, since there's never a reference to the inside to
worry about.

**The classic combo:** `Rc<RefCell<T>>` — shared ownership (`Rc`) of
something mutable (`RefCell`) — is how you build things like a
doubly-linked structure or shared mutable state in single-threaded Rust,
where Python/Java would just pass the same object reference around and
mutate it freely, no ceremony needed.

## `Arc<T>` — `Rc`'s thread-safe sibling

Identical API to `Rc<T>`, but the reference count is updated atomically,
making it safe to share across threads. `Rc<T>` is not thread-safe (and
the compiler enforces this — see [Concurrency](./08-concurrency.md) for
`Send`/`Sync`) and will refuse to compile if you try to share one across
a `thread::spawn` boundary. Rule of thumb: single-threaded → `Rc`,
multi-threaded → `Arc`. `Arc<Mutex<T>>` is the multi-threaded analog of
`Rc<RefCell<T>>` — covered in the next chapter.

## Quick decision table

| Need | Reach for |
|---|---|
| One owner, heap-allocated | `Box<T>` |
| Multiple owners, single-threaded, read-only sharing | `Rc<T>` |
| Multiple owners, single-threaded, need mutation too | `Rc<RefCell<T>>` |
| Multiple owners, across threads | `Arc<T>` |
| Multiple owners, across threads, need mutation too | `Arc<Mutex<T>>` |
