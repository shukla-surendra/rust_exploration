// Two threads running at the same time, using named functions
//
// thread::spawn(counter) starts a new thread that runs counter(). We pass the
// function itself (no parentheses): it takes no input, so no closure is needed.
// (With input you'd write thread::spawn(move || f(x)), see 21_threads_with_functions.rs.)
//
// Both threads run at once, so their output INTERLEAVES:
//   Counting to 10...
//   Counting to 10...
//   1
//   2
//   2
//   4
//   ...   (the exact order can change between runs)
//
// Each thread sleeps 500 ms per step, 10 steps = 5 seconds. Because they run in
// parallel, the whole program takes about 5 seconds, not 10.
use std::thread;
fn main(){
// spawn returns a JoinHandle: our way to wait for that thread later.
let handle = thread::spawn(counter);
let handle2 = thread::spawn(counter_twice);
// without join, the main thread may finish before the spawned threads complete, leading to premature termination of the program. 
// By calling join on each handle, we ensure that the main thread waits for both spawned threads to finish their execution before exiting. 
// This is crucial for long-running threads to complete their tasks properly.
// join() returns a Result: Err if the thread panicked. unwrap() crashes main in
// that case (see 14_unwrap_usage.rs).
handle.join().unwrap();
handle2.join().unwrap();
}

fn counter(){
    println!("Counting to 10...");
    // 1..=10 = 1 to 10 INCLUDING 10. (1..10 would stop at 9.)
    for i in 1..=10 {
        println!("{i}");
        // Pause this thread only; the other thread keeps running.
        thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn counter_twice(){
    println!("Counting to 10...");
    for i in 1..=10 {
        println!("{}", i*2);
        thread::sleep(std::time::Duration::from_millis(500));
    }
}
