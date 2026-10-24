use std::thread;
use std::time::Duration;

fn main() {
    let mut handles = vec![];

    for i in 0..10 {
        // spawn a thread
        let handle = thread::spawn(move || {
            println!("Thread {} started!", i);
            thread::sleep(Duration::from_millis(500));
            println!("Thread {} finished!", i);
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads finished!");
}
