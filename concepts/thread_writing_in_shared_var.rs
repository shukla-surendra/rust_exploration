// Sharing a value among threads
//
// Goal: 3 threads each add 1 to the same counter; the final value should be 3.
//
// Two problems, two tools:
// 1. Who owns the counter? Each thread needs it, but a value has one owner, and a
//    thread may outlive main's local variables. -> Arc<T> (Atomically Reference
//    Counted): shared ownership. Each clone is another owner of the SAME value; the
//    value is freed when the last clone is dropped. "Atomic" = the count is safe to
//    update from many threads (Rc<T> is the single-thread version and won't compile here).
// 2. Who can change it? Arc only gives shared (read-only) access. -> Mutex<T>
//    (mutual exclusion): lock() lets only one thread at a time get mutable access.
//
// Arc<Mutex<T>> = "many owners, one writer at a time".
//
//   counter (main) ──┐
//   thread 1 clone ──┼──> Arc ──> Mutex ──> 0      (Arc count = 4)
//   thread 2 clone ──┤             └─ lock(): one thread at a time
//   thread 3 clone ──┘
//
// What breaks without each one:
// | You use         | Result                                                             |
// |-----------------|--------------------------------------------------------------------|
// | Only Mutex      | Compile error: first thread moves the Mutex in -> "use of moved    |
// |                 | value" on the next loop iteration. (One owner only.)               |
// | Only Arc        | Compile error: "cannot assign to data in an `Arc`". (Read-only.)   |
// | Arc<Mutex<T>>   | Works: Arc gets it into every thread, Mutex serializes the writes. |
//
// They are NOT always used together:
// - Arc alone:   many threads only READ shared data, e.g. Arc<Vec<String>> config.
// - Mutex alone: no shared ownership needed, e.g. threads in thread::scope can
//                borrow a local Mutex (they're guaranteed to finish before it's dropped).
// - Neither:     a plain counter can use an atomic instead, Arc<AtomicUsize>, which
//                updates the number safely without a lock.

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // counter --> Arc --> Mutex --> 0
    let counter = Arc::new(Mutex::new(0));

    // Holds a JoinHandle for each thread so we can wait for them to finish.
    let mut handles = vec![];

    for _ in 0..3 {
        // Arc::clone does NOT copy the Mutex or the 0: it makes a new pointer to the same
        // value and bumps the reference count (1 -> 2 -> 3 -> 4). Shadowing gives each
        // thread its own `counter` handle to move in, while main keeps the original.
        let counter = Arc::clone(&counter);

        // thread::spawn runs the closure on a new thread and returns a JoinHandle.
        // `move` makes the closure take ownership of what it uses (this clone of
        // `counter`). Required: the thread could outlive this loop iteration, so it
        // can't just borrow a local variable.
        let handle = thread::spawn(move || {
            // lock() waits until no other thread holds the lock, then returns
            // Result<MutexGuard, PoisonError>.
            // unwrap(): take the MutexGuard on Ok, or panic on Err. Err happens only
            // if another thread panicked while holding the lock ("poisoned" mutex).
            // num is a MutexGuard: it acts like &mut i32 to the value inside.
            let mut num = counter.lock().unwrap();

            // Deref the guard to change the value inside the Mutex.
            *num += 1;

            // Order of these lines can vary between runs: the OS decides which
            // thread runs first. The values are still 1, 2, 3 because of the lock.
            println!("Counter: {}", *num);
        }); // num (the guard) is dropped here -> the lock is released automatically.
            // The thread's Arc clone is dropped too -> reference count goes down by 1.

        handles.push(handle);
    }

    for handle in handles {
        // join() blocks until that thread finishes. It returns Result: Err if the
        // thread panicked. unwrap() re-raises that panic here in main.
        // Without joining, main could print (and exit) before the threads are done.
        handle.join().unwrap();
    }

    // All threads are done, so this always prints 3.
    // lock().unwrap() gets the guard; *guard reads the value. The guard is a
    // temporary, so the lock is released at the end of this statement.
    println!("Final counter: {}", *counter.lock().unwrap());
}
