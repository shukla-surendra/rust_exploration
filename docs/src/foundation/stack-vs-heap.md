# Stack vs Heap: why Rust defaults to the stack, and Python doesn't

## The two memory regions, quickly

- **Stack** — a fixed-size, LIFO region. Allocating is just moving a
  pointer; freeing is moving it back. Extremely fast, but every value on
  it must have a size known at compile time, and it disappears when its
  function frame returns.
- **Heap** — a general-purpose pool. Allocating means asking an allocator
  to find free space (slower, and needs bookkeeping); the data survives
  until something explicitly frees it. Values can grow, shrink, or outlive
  the function that created them.

Every language uses both. The difference between Rust and Python is
**which one is the default for an ordinary variable**, and that comes
down to what each language needs to know about a value.

## Rust: stack by default

Rust's compiler tracks, for every value, an exact size and an exact
owner. That's what makes stack allocation the default:

- **Sizes are known at compile time.** `let x: i32 = 5;` — the compiler
  knows `i32` is 4 bytes, full stop. It can carve out that space on the
  stack frame right there, no allocator involved.
- **Ownership has a single, statically-tracked owner.** Rust doesn't need
  a garbage collector or reference counting to know when to free memory —
  it knows at compile time exactly which line of code causes a value to
  go out of scope, and it inserts the cleanup (`drop`) there. For
  stack values that's free (the frame just pops); for heap values owned
  through something like `Box`, it's a deterministic `free()` call, not
  a GC sweep.
- **No indirection tax.** Because the compiler already knows types and
  sizes, it doesn't need to wrap every value in a pointer-plus-metadata
  box just to work with it generically.

You only pay for the heap when you explicitly ask for it — `Box::new`,
`Vec`, `String`, `Rc`/`Arc` — typically because a size isn't known until
runtime, or because a value needs to be shared or outlive its creating
scope. See [`heap_allocation.rs`](../../../variables/src/bin/heap_allocation.rs)
for a minimal example of reaching for `Box` on purpose, and "Choosing
between them" in [String vs str / &str](./strings.md) for the
owned-vs-borrowed side of the same tradeoff. For the specific scenarios
where "size known at compile time" breaks down, see
[Sized vs Unsized](./sized-vs-unsized.md).

## Python: heap by (almost) default

In CPython, **every value is an object** (a `PyObject`), even an `int`.
`x = 5` doesn't put `5` directly in a variable slot — `x` holds a
reference to a heap-allocated `PyObject` that wraps the integer, plus a
type tag and a reference count. This isn't a style choice; it falls out
of two things Python guarantees that Rust doesn't:

- **Dynamic typing.** A variable can hold an `int` today and a `str`
  tomorrow. The variable itself can't be a fixed-size slot for "whatever
  this type is" because the type — and therefore the size — isn't known
  until runtime. The only fixed-size thing every variable *can* hold is
  a pointer to a heap object that knows its own type and size.
- **Shared, GC'd ownership.** Multiple names can reference the same
  object (`y = x`) and Python doesn't statically track when the last
  reference disappears — it counts references at runtime (plus a cycle
  collector for reference cycles) and frees the object when the count
  hits zero. Reference counting only makes sense against heap objects
  with a stable address; you can't reference-count a stack slot that's
  about to be popped by an unrelated function return.

So in Python, the heap isn't the "expensive escape hatch" the way it is
in Rust — it's the only place a dynamically-typed, reference-counted
object can live. The stack still exists (it holds the *pointers* — each
frame's local variables — and manages call/return), but the data those
pointers point to is heap-allocated PyObjects almost without exception.

## Side by side

| | Rust | Python |
|---|---|---|
| Default location for a value | stack | heap (as a `PyObject`) |
| Why | size + owner known at compile time | type/size only known at runtime; refs are shared and GC'd |
| Deallocation | deterministic, compiler-inserted (`drop`) | reference counting + cycle-collecting GC |
| Cost of an "ordinary" variable | one stack slot, no allocator call | one heap allocation + refcount per object |
| How you opt into the heap | explicitly: `Box`, `Vec`, `String`, `Rc`/`Arc` | implicitly: it's where everything already lives |

## The takeaway

Rust can default to the stack because the compiler does the bookkeeping
(sizes, lifetimes, ownership) ahead of time and enforces it at compile
time. Python defaults to the heap because it defers that same
bookkeeping (types, lifetimes, sharing) to runtime, and only heap objects
can carry the metadata needed to resolve it there. `Box` in Rust is you
manually opting into the model Python uses for every single value.
