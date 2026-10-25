// Sharing a value among threads: READ only
//
// Goal: 3 threads all read the same list of names.
//
// Only one problem this time: every thread needs the data (ownership).
// Nobody changes it, so no Mutex is needed -> Arc<T> alone is enough.
//
//   names (main)   ──┐
//   thread 0 clone ──┼──> Arc ──> ["Alice", "Bob", "Carol"]   (Arc count = 4)
//   thread 1 clone ──┤
//   thread 2 clone ──┘
//
// Compare with 42_thread_writing_in_shared_var.rs, which needs Arc<Mutex<T>> because
// the threads WRITE.

use std::sync::Arc;
use std::thread;

fn main() {
    // The data lives once, inside the Arc.
    let names = Arc::new(vec!["Alice", "Bob", "Carol"]);

    let mut handles = vec![];

    for i in 0..3 {
        // New pointer to the same Vec (no copy of the data), count goes up by 1.
        let names = Arc::clone(&names);

        // `move` gives this thread ownership of its Arc clone.
        let handle = thread::spawn(move || {
            // Reading through an Arc needs no lock: shared read-only access is
            // always safe. Writing (e.g. names.push(..)) would not compile.
            println!("Thread {} reads: {}", i, names[i]);
        }); // This thread's clone is dropped here -> count goes down by 1.

        handles.push(handle);
    }

    // Wait for all threads to finish.
    for handle in handles {
        handle.join().unwrap();
    }

    // main still owns its Arc, so it can keep using the data.
    // Only main's clone is left now, so the count is back to 1.
    println!("All names: {:?}", names);
    println!("Arc count: {}", Arc::strong_count(&names));
}
