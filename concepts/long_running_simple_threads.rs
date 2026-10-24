use std::thread;
fn main(){
let handle = thread::spawn(counter);
let handle2 = thread::spawn(counter_twice);
// without join, the main thread may finish before the spawned threads complete, leading to premature termination of the program. 
// By calling join on each handle, we ensure that the main thread waits for both spawned threads to finish their execution before exiting. 
// This is crucial for long-running threads to complete their tasks properly.
handle.join().unwrap();
handle2.join().unwrap();
}

fn counter(){
    println!("Counting to 10...");
    for i in 1..=10 {
        println!("{i}");
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
