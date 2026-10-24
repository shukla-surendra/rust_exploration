// Simple threads: start 10 threads, wait for all of them to finish.
//
// Key syntax: `move || { ... }` is a CLOSURE (an anonymous function written inline).
//
//   ||          -> the parameter list, here empty. Like `()` in `fn name()`.
//                  With parameters:  |x| x + 1      |a, b| a * b
//   { ... }     -> the body: the code that runs when the closure is called.
//   move        -> the closure takes OWNERSHIP of the variables it uses from
//                  outside (here `i`), instead of borrowing them.
//
// Closure vs fn:
//   fn add_one(x: i32) -> i32 { x + 1 }      // named function
//   let add_one = |x: i32| x + 1;            // closure stored in a variable
//   add_one(5);                              // both called the same way -> 6
// The difference: a closure can use ("capture") variables from the code around it.
//
// Why `move` is needed here:
//   The new thread may keep running after this loop iteration (or even main) ends,
//   when `i` no longer exists. A borrow of `i` could point to nothing, so Rust
//   refuses it. Without `move` you get:
//     error[E0373]: closure may outlive the current function, but it borrows `i`
//   With `move`, each thread gets its OWN `i`. i32 is Copy, so it's a copy: thread 0
//   gets 0, thread 1 gets 1, and so on.

use std::thread;
use std::time::Duration;

fn main() {
    // Will hold one JoinHandle per thread so we can wait for them later.
    let mut handles = vec![];

    for i in 0..10 {
        // thread::spawn takes a closure and runs it on a new thread.
        // It returns immediately (it doesn't wait for the thread) with a JoinHandle.
        let handle = thread::spawn(move || {
            // This body runs on the new thread, using its own copy of `i`.
            println!("Thread {} started!", i);
            // Pause this thread only (not the others) for 500 milliseconds.
            thread::sleep(Duration::from_millis(500));
            println!("Thread {} finished!", i);
        });

        handles.push(handle);
    }

    // All 10 threads are running at the same time now. Their "started" and
    // "finished" lines print in a different order each run: the OS decides
    // which thread runs when. Because they sleep in parallel, the whole program
    // takes ~0.5 s, not 10 × 0.5 s = 5 s.

    // Wait for all threads to finish.
    // join() blocks until that thread ends; it returns Result (Err if the thread
    // panicked), and unwrap() turns that into a panic here in main.
    // Without this loop, main could reach the end and exit, killing the threads
    // before they print "finished".
    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads finished!");
}
