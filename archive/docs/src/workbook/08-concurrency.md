# 8. Concurrency

**What this replaces:** Java's `Thread`/`synchronized`/`ExecutorService`
— conceptually close. Python's `threading` module is the wrong mental
model to reach for here: the GIL means Python threads don't actually run
Python bytecode in parallel, so real parallel data races mostly can't
happen in pure Python the way they can in Java or Rust. Rust threads are
real OS threads with no GIL, so the hazards Java code has to be
disciplined about are exactly the ones Rust prevents at compile time.

## Spawning a thread

```rust
use std::thread;

let handle = thread::spawn(|| {
    println!("running in a new thread");
});

handle.join().unwrap();   // block until the thread finishes
```

`thread::spawn` takes a closure (chapter 5) and runs it on a new OS
thread — directly analogous to Java's `new Thread(() -> {...}).start()`.
`.join()` is Java's `Thread.join()`: block the current thread until the
spawned one completes; it returns a `Result` (the thread might have
panicked), hence the `.unwrap()`.

## The rule that makes Rust concurrency different: shared state must prove it's safe

```rust
let data = vec![1, 2, 3];
let handle = thread::spawn(|| {
    println!("{:?}", data);   // COMPILE ERROR (without `move`): closure may outlive `data`
});
```

A closure passed to `thread::spawn` might run *after* the spawning
function has already returned — so it can't be allowed to merely borrow
local data, since that data might not exist anymore by the time the
thread actually runs. The fix is `move`:

```rust
let handle = thread::spawn(move || {
    println!("{:?}", data);   // closure now OWNS data — no dangling-reference risk
});
```

`move` forces the closure to take ownership of everything it captures,
same keyword, same ownership machinery from chapter 2 — concurrency
safety here is just the *existing* borrow-checker rules applied across a
thread boundary, not a separate concurrency-specific system. This is the
mechanical reason Rust famously "prevents data races at compile time":
a data race requires two threads with simultaneous mutable-or-mixed
access to the same memory, and rule 3 from chapter 2 (many readers XOR
one writer) already forbids that within a single thread — extending it
across threads via `move`/`Send`/`Sync` closes the same loophole for
concurrent code.

## Sharing state between threads: `Arc<Mutex<T>>`

Multiple threads needing to *share and mutate* the same data (the
concurrent equivalent of `Rc<RefCell<T>>` from chapter 7):

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut num = counter.lock().unwrap();   // blocks until the lock is free
        *num += 1;
    }));                                           // lock automatically released here
}

for handle in handles { handle.join().unwrap(); }
println!("{}", *counter.lock().unwrap());          // 10
```

- `Arc<T>` — shared ownership across threads (chapter 7), needed because
  each spawned thread needs its own handle to the same `Mutex`.
- `Mutex<T>` — like Java's `synchronized`/`ReentrantLock`, but the lock
  and the data it protects are *the same object* — you cannot get at the
  `i32` inside without going through `.lock()`, so there's no way to
  "forget" to lock before touching the shared data, unlike Java where
  `synchronized` and the field it protects are two separate things you
  have to remember to keep paired correctly.
- `.lock()` returns a `MutexGuard` (a smart pointer, like `Box`/`Rc`) —
  when it goes out of scope (end of the loop body above), the lock is
  released automatically via `Drop`. No `finally { lock.unlock(); }`
  needed, and no way to forget it.

## Channels — message passing instead of shared memory

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    tx.send("hello from the thread").unwrap();
});

println!("{}", rx.recv().unwrap());   // blocks until a message arrives
```

`mpsc` = "multiple producer, single consumer" — clone `tx` to have
several threads send into the same channel; `rx` only ever has one
receiver. Directly comparable to Java's `BlockingQueue`, or Python's
`queue.Queue` — "don't share memory to communicate, communicate to
share memory," Rust's version of the same message-passing pattern those
offer, just baked into `std`.

## `Send` and `Sync` — the traits that make all of this enforceable

You won't write these often, but recognizing them explains *why* certain
code refuses to compile:

- **`Send`** — a type is safe to **move** to another thread. Almost
  everything is `Send`; `Rc<T>` famously is *not* (its reference count
  isn't atomic — moving it across threads could race), which is exactly
  why the compiler rejects `Rc` where `Arc` is required.
- **`Sync`** — a type is safe to **share by reference** (`&T`) across
  threads. `Mutex<T>` is `Sync` even when `T` isn't, because the mutex
  itself guarantees only one thread touches the inside at a time.

These are marker traits (no methods, like `Eq` from chapter 4) that the
compiler derives automatically for most types — you mostly encounter
them as the *reason* in an error message ("`Rc<i32>` cannot be sent
between threads safely") rather than something you implement by hand.
