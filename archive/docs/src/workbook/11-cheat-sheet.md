# 11. Cheat Sheet — Python / Java / Rust, side by side

The "keep open in a second window" page. Terse on purpose — every row
links back to the chapter with the actual explanation if it doesn't
click immediately.

## Variables & types

| | Python | Java | Rust |
|---|---|---|---|
| Declare | `x = 5` | `int x = 5;` | `let x = 5;` |
| Mutable | always | unless `final` | `let mut x = 5;` — immutable by default ([ch.1](./01-basics-and-control-flow.md)) |
| Reassign type | yes | no | no (shadowing aside) |
| Type check | runtime | compile time | compile time, usually inferred |
| `null`/absence | `None` | `null` | `Option<T>` — `Some`/`None` ([ch.6](./06-error-handling.md)) |

## Functions

| | Python | Java | Rust |
|---|---|---|---|
| Define | `def f(x): return x` | `int f(int x) { return x; }` | `fn f(x: i32) -> i32 { x }` |
| Return | explicit `return` | explicit `return` | last expression, **no `;`** ([ch.1](./01-basics-and-control-flow.md)) |
| Overloading | no (default args instead) | yes | no (traits/generics instead) |
| Lambda | `lambda x: x*2` | `x -> x*2` | `\|x\| x*2` ([ch.5](./05-collections-closures-iterators.md)) |

## Control flow

| | Python | Java | Rust |
|---|---|---|---|
| Conditional | `if/elif/else` | `if/else if/else` | `if/else if/else` — **is an expression** |
| Ternary | `a if c else b` | `c ? a : b` | `if c { a } else { b }` (no separate `?:`) |
| Switch | `match` (3.10+) | `switch` | `match` — exhaustive, no fallthrough ([ch.3](./03-structs-enums-pattern-matching.md)) |
| For-each | `for x in xs:` | `for (T x : xs)` | `for x in &xs { }` |
| Range loop | `for i in range(5):` | `for (int i=0; i<5; i++)` | `for i in 0..5 { }` |
| Infinite loop | `while True:` | `while (true)` | `loop { }` — can `break value` |

## Data structures

| | Python | Java | Rust |
|---|---|---|---|
| Growable array | `list` | `ArrayList<T>` | `Vec<T>` ([ch.5](./05-collections-closures-iterators.md)) |
| Fixed array | (no real equivalent) | `T[]` | `[T; N]` |
| Borrowed view | slicing `lst[1:3]` (copies) | (no equivalent) | `&[T]` (no copy) |
| Hash map | `dict` | `HashMap<K,V>` | `HashMap<K,V>` |
| Hash set | `set` | `HashSet<T>` | `HashSet<T>` |
| Tuple | `(1, 2)` | (no built-in) | `(1, 2)` |
| Struct/record | `@dataclass class` | `class`/`record` | `struct` ([ch.3](./03-structs-enums-pattern-matching.md)) |
| Tagged union | `Union[...]` (unenforced) | (no equivalent) | `enum` with data per variant |

## Strings

| | Python | Java | Rust |
|---|---|---|---|
| Type | `str` | `String` | `String` (owned) / `&str` (borrowed view) — [full page](../foundation/strings.md) |
| Concatenate | `a + b` | `a + b` | `format!("{a}{b}")` or `a.push_str(b)` |
| Interpolate | f-strings `f"{x}"` | `String.format` / `"%s".formatted` | `format!("{x}")` / `println!("{x}")` |

## Object-orientation / behavior

| | Python | Java | Rust |
|---|---|---|---|
| Class | `class Foo:` | `class Foo {}` | `struct Foo {}` + `impl Foo {}` ([ch.3](./03-structs-enums-pattern-matching.md)) |
| Inheritance | yes | yes | **no** — composition + traits instead |
| Interface | duck typing / `Protocol` | `interface` | `trait` ([ch.4](./04-traits-and-generics.md), [full page](../foundation/traits.md)) |
| Generics | `TypeVar` (unenforced at runtime) | `<T>` (type-erased) | `<T>` (monomorphized — real, per-type compiled copies) |
| `this`/`self` | `self` (explicit param) | `this` (implicit) | `self`/`&self`/`&mut self` (explicit, and mutability-checked) — [full page](../foundation/structs-and-methods.md) |

## Errors

| | Python | Java | Rust |
|---|---|---|---|
| Mechanism | exceptions | exceptions (checked + unchecked) | `Result<T, E>` — no exceptions at all ([ch.6](./06-error-handling.md)) |
| Handle | `try/except` | `try/catch` | `match`/`if let`/`?`/`.unwrap_or_else()` |
| Propagate | automatic (until caught) | automatic, or declared `throws` | explicit `?` at every hop |
| "Impossible" state | assert / uncaught exception | `assert` / `RuntimeException` | `panic!()` / `.unwrap()` / `todo!()` |

## Memory & references

| | Python | Java | Rust |
|---|---|---|---|
| Memory management | GC (refcount + cycle collector) | GC | none — ownership, freed deterministically on scope exit ([ch.2](./02-ownership-borrowing-lifetimes.md)) |
| Assignment `b = a` | both point to same object | both point to same object | **moves** ownership from `a` to `b` (unless `T: Copy`) |
| Passing to a function | reference (mutable objects mutate the caller's copy) | reference | moves, or borrows via `&`/`&mut` — caller's choice, enforced |
| Multiple owners | implicit (GC handles it) | implicit (GC handles it) | explicit: `Rc<T>` (single-thread) / `Arc<T>` (multi-thread) ([ch.7](./07-memory-and-smart-pointers.md)) |
| Mutate through a shared ref | always allowed | always allowed | needs `RefCell<T>`/`Mutex<T>` — "interior mutability" |

## Concurrency

| | Python | Java | Rust |
|---|---|---|---|
| Threads | `threading` (GIL-limited — not truly parallel for CPU work) | `Thread`/`ExecutorService` | `thread::spawn` — real OS threads, no GIL ([ch.8](./08-concurrency.md)) |
| Shared mutable state | just share the object | `synchronized` / `ReentrantLock` | `Arc<Mutex<T>>` — lock and data are the same object |
| Data races | possible but rare (GIL) | possible, a real hazard | **prevented at compile time** |
| Message passing | `queue.Queue` | `BlockingQueue` | `std::sync::mpsc::channel()` |

## Type conversion

| | Python | Java | Rust |
|---|---|---|---|
| Implicit numeric coercion | yes (`1 + 1.0` works) | yes (widening) | **never** — always explicit ([ch.9](./09-type-conversions.md)) |
| Explicit, possibly lossy | `int(x)` | `(int) x` | `x as i32` |
| Explicit, safe/checked | — | — | `T::try_from(x)` → `Result` |
| Explicit, guaranteed-safe | `str(x)` | `String.valueOf(x)` | `T::from(x)` / `x.into()` |
| Parse string | `int("42")` (raises) | `Integer.parseInt` (throws) | `"42".parse::<i32>()` → `Result` |

## Tooling

| | Python | Java | Rust |
|---|---|---|---|
| Package manifest | `pyproject.toml` | `pom.xml` / `build.gradle` | `Cargo.toml` ([ch.10](./10-cargo-modules-testing-macros.md)) |
| Lockfile | `poetry.lock` | (dependency-managed by Maven/Gradle) | `Cargo.lock` |
| Install a dependency | `pip install` | edit `pom.xml` + reload | `cargo add` |
| Run tests | `pytest` | `mvn test` | `cargo test` |
| Format code | `black` | (IDE/plugin) | `cargo fmt` (built in) |
| Lint | `flake8`/`ruff` | (IDE/plugin) | `cargo clippy` (built in) |
