// unwrap(): where you see it and what it does
//
// Many Rust functions don't return the value directly. They return a "box" that
// may or may not have the value inside:
//
//   Option<T>     = Some(value)  or  None        (value might be missing)
//   Result<T, E>  = Ok(value)    or  Err(error)  (operation might fail)
//
// unwrap() opens the box:
//   Some(v) / Ok(v)  -> gives you v
//   None    / Err(e) -> PANICS (the program crashes with an error message)
//
// So unwrap() means: "I'm sure the value is there; crash if I'm wrong."
// Fine for examples, tests and quick scripts. In real code, prefer the safer
// alternatives shown in Example 7.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // ------------------------------------------------------------------
    // Example 1: What unwrap() really is (written out by hand)
    // ------------------------------------------------------------------
    println!("--- Example 1: unwrap() written by hand ---");
    let maybe: Option<i32> = Some(5);
    // This match is exactly what maybe.unwrap() does:
    let value = match maybe {
        Some(v) => v,
        // panic! is a built-in macro (the ! means macro, like println!), no import
        // needed. It stops the current thread with this message. See Example 8.
        None => panic!("called `Option::unwrap()` on a `None` value"),
    };
    println!("By hand: {}", value);
    println!("unwrap(): {}", maybe.unwrap());

    // ------------------------------------------------------------------
    // Example 2: Parsing text into a number (Result)
    // ------------------------------------------------------------------
    println!("\n--- Example 2: parse() ---");
    // "42".parse() returns Result<i32, ParseIntError>: the text might not be a number.
    let n: i32 = "42".parse().unwrap(); // Ok(42) -> 42
    println!("Parsed: {}", n);
    // "abc".parse::<i32>().unwrap() would panic: Err(ParseIntError).

    // ------------------------------------------------------------------
    // Example 3: Getting an item from a Vec (Option)
    // ------------------------------------------------------------------
    println!("\n--- Example 3: Vec get() / first() ---");
    let numbers = vec![10, 20, 30];
    // get(i) returns Option<&i32>: the index might be out of range.
    let second = numbers.get(1).unwrap(); // Some(&20) -> &20
    println!("Second: {}", second);
    // first() returns Option<&i32>: the Vec might be empty.
    println!("First: {}", numbers.first().unwrap());
    // numbers.get(10).unwrap() would panic: None.

    // ------------------------------------------------------------------
    // Example 4: Looking up a key in a HashMap (Option)
    // ------------------------------------------------------------------
    println!("\n--- Example 4: HashMap get() ---");
    let mut ages = HashMap::new();
    ages.insert("Alice", 30);
    // get(key) returns Option<&V>: the key might not exist.
    println!("Alice is {}", ages.get("Alice").unwrap()); // Some(&30) -> &30
    // ages.get("Bob").unwrap() would panic: None.

    // ------------------------------------------------------------------
    // Example 5: Characters of a string (Option)
    // ------------------------------------------------------------------
    println!("\n--- Example 5: chars().next() ---");
    // next() returns Option<char>: the string might be empty.
    let first_letter = "Rust".chars().next().unwrap(); // Some('R') -> 'R'
    println!("First letter: {}", first_letter);

    // ------------------------------------------------------------------
    // Example 6: Threads, Mutex (Result) — the ones from the thread files
    // ------------------------------------------------------------------
    println!("\n--- Example 6: lock() and join() ---");
    let counter = Arc::new(Mutex::new(0));
    let c = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        // lock() returns Result: Err only if another thread panicked while holding
        // the lock (a "poisoned" mutex).
        *c.lock().unwrap() += 1;
    });
    // join() returns Result: Err if the thread panicked.
    handle.join().unwrap();
    println!("Counter: {}", counter.lock().unwrap());

    // ------------------------------------------------------------------
    // Example 7: Safer alternatives to unwrap()
    // ------------------------------------------------------------------
    println!("\n--- Example 7: Alternatives ---");
    let missing: Option<i32> = None;

    // expect("msg"): same as unwrap(), but panics with YOUR message. Easier to debug.
    let n: i32 = "7".parse().expect("text should be a number");
    println!("expect: {}", n);

    // unwrap_or(default): give a fallback value instead of panicking.
    println!("unwrap_or: {}", missing.unwrap_or(0));

    // unwrap_or_default(): fallback is the type's default (0 for i32, "" for String).
    println!("unwrap_or_default: {}", missing.unwrap_or_default());

    // unwrap_or_else(|| ...): compute the fallback only when needed.
    println!("unwrap_or_else: {}", missing.unwrap_or_else(|| 100 + 1));

    // match / if let: handle both cases yourself.
    match "abc".parse::<i32>() {
        Ok(v) => println!("match: parsed {}", v),
        Err(e) => println!("match: could not parse: {}", e),
    }
    if let Some(age) = ages.get("Bob") {
        println!("if let: Bob is {}", age);
    } else {
        println!("if let: Bob not found");
    }

    // ? operator: inside a function that returns Result/Option, pass the error up
    // to the caller instead of panicking. See add_strings() below.
    println!("?: {:?}", add_strings("2", "3"));
    println!("?: {:?}", add_strings("2", "x"));

    // ------------------------------------------------------------------
    // Example 8: panic! — what happens when unwrap() fails
    // ------------------------------------------------------------------
    // panic! means "something went wrong that the program can't recover from".
    // Takes a message like println!:  panic!("bad value: {}", x);
    //
    // When a panic happens:
    // 1. An error message with file:line is printed to stderr.
    // 2. Unwinding: Rust leaves each function on the way up, dropping (freeing)
    //    their local variables. Nothing after the panic runs in that thread.
    // 3. The thread stops:
    //    - main thread    -> the whole program exits with exit code 101.
    //    - spawned thread -> only that thread dies; its join() returns Err.
    //
    // Things that panic for you (you rarely write panic! yourself):
    //   None.unwrap() / Err(e).unwrap() / expect("msg")
    //   v[10] on a 3-item Vec      (index out of bounds)
    //   10 / 0 with integers       (divide by zero)
    //   255u8 + 1 in debug builds  (integer overflow)
    // Macros that panic on purpose:
    //   assert!(cond), assert_eq!(a, b)  -> panic if the check fails
    //   todo!()                          -> "not written yet"
    //   unreachable!()                   -> "this should never run"
    //
    // panic vs Result:
    //   panic!  -> bugs, "this should never happen". Crashes.
    //   Result  -> expected failures (bad input, missing file). Caller decides.
    //
    // Tip: RUST_BACKTRACE=1 ./target/unwrap_usage shows the chain of calls.
    println!("\n--- Example 8: panic! ---");

    // Demo: panic inside a spawned thread so main survives and we can watch it.
    let handle = thread::spawn(|| {
        let bad: Option<i32> = None;
        println!("Thread: about to unwrap None...");
        bad.unwrap(); // panics here: message is printed to stderr
        println!("Thread: this line never runs");
    });
    // The thread panicked, so join() returns Err instead of Ok.
    match handle.join() {
        Ok(_) => println!("Main: thread finished normally"),
        Err(_) => println!("Main: thread panicked, but main is still running"),
    }

    // In main itself, a panic would end the whole program (uncomment to try):
    // let bad: Option<i32> = None;
    // bad.unwrap();
    // Output:
    // thread 'main' panicked at unwrap_usage.rs:...:
    // called `Option::unwrap()` on a `None` value
    // (program exits with code 101; nothing after this line runs)
    println!("Main: end of program");
}

// Returns Result instead of panicking. Each `?` means:
//   Ok(v)  -> continue with v
//   Err(e) -> return Err(e) from this function right now
fn add_strings(a: &str, b: &str) -> Result<i32, std::num::ParseIntError> {
    let x: i32 = a.parse()?;
    let y: i32 = b.parse()?;
    Ok(x + y)
}
