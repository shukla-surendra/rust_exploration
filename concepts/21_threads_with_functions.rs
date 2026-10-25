// Threads that run normal named functions, with input passed in.
//
// In simple_threads.rs all the thread's code was written inside the closure.
// Here the work lives in separate functions (greet, square, count_letters), and
// each thread just calls one of them with its input.
//
// Three ways to start a thread with a function:
//   1. No input:    thread::spawn(say_hello)              // pass the function itself
//   2. With input:  thread::spawn(move || greet(name))     // closure calls it with args
//   3. With output: let h = thread::spawn(move || square(n));
//                   let result = h.join().unwrap();       // get the return value back
//
// Input can also come from the command line:
//   ./target/threads_with_functions Alice Bob Carol
// (with no arguments, default names are used)

use std::env;
use std::thread;

// ---- Functions that will run on threads ----

// No input, no output.
fn say_hello() {
    println!("[say_hello] Hello from a thread!");
}

// Takes input. `name: String` is owned, so the thread owns its own copy of the text.
fn greet(name: String) {
    println!("[greet] Hi, {}!", name);
}

// Takes input and returns a value. The return value comes back through join().
fn square(n: i32) -> i32 {
    n * n
}

// Takes input and returns a value.
fn count_letters(word: String) -> usize {
    word.len()
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: A function with no input
    // ------------------------------------------------------------------
    println!("--- Example 1: no input ---");
    // spawn needs "something callable with no arguments". A function with no
    // parameters already is that, so we pass its name directly (no () after it:
    // say_hello() would CALL it here; say_hello hands it to the thread).
    let handle = thread::spawn(say_hello);
    handle.join().unwrap();

    // ------------------------------------------------------------------
    // Example 2: Functions with input (from the command line)
    // ------------------------------------------------------------------
    println!("\n--- Example 2: input from command line ---");
    // env::args() gives the program name first, then the arguments.
    // skip(1) drops the program name; collect() puts the rest in a Vec<String>.
    let mut names: Vec<String> = env::args().skip(1).collect();
    if names.is_empty() {
        names = vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()];
    }

    let mut handles = vec![];
    for name in names {
        // spawn can't pass arguments itself, so we wrap the call in a closure:
        //   move || greet(name)
        // `move` moves this `name` (a String) into the closure, so the thread owns
        // it. Then the closure calls greet(name) on the new thread.
        let handle = thread::spawn(move || greet(name));
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    // ------------------------------------------------------------------
    // Example 3: Getting a result back from a thread
    // ------------------------------------------------------------------
    println!("\n--- Example 3: return values ---");
    let numbers = vec![2, 3, 4];

    let mut handles = vec![];
    for n in numbers {
        // The closure returns whatever square(n) returns, so this is a
        // JoinHandle<i32>: a handle that will give back an i32.
        handles.push(thread::spawn(move || square(n)));
    }
    for handle in handles {
        // join() waits for the thread and returns Result<i32, _>.
        // unwrap() gives the i32 the function returned.
        let result = handle.join().unwrap();
        println!("[square] {}", result);
    }

    // ------------------------------------------------------------------
    // Example 4: Two different functions running at the same time
    // ------------------------------------------------------------------
    println!("\n--- Example 4: different functions in parallel ---");
    let word = String::from("thread");

    // Both threads start right away and run in parallel.
    // word.clone() makes a separate String for the thread, so main keeps `word`.
    let letters_handle = thread::spawn({
        let word = word.clone();
        move || count_letters(word)
    });
    let square_handle = thread::spawn(|| square(12)); // no outside variables, so no move needed

    // Collect both results (waits for each thread to finish).
    let letters = letters_handle.join().unwrap(); // JoinHandle<usize>
    let squared = square_handle.join().unwrap();  // JoinHandle<i32>
    println!("[count_letters] '{}' has {} letters", word, letters);
    println!("[square] 12 * 12 = {}", squared);
}
