use std::fs::File;
use std::io::Read;
use std::fs;
use std::io::Write;
use std::fs::OpenOptions;

fn main() {

    // # openning a file 
    let file = File::open("file.txt");
    
    match file {
        Ok(file) => {
            println!("File opened successfully");
        }
        Err(e) => println!("Got Error: {}", e),
    }
    // # reading a file
    let mut file = File::open("hello.txt").expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read file");
    println!("File content:\n{}", contents);
    // Easier way of reading a file
    let contents_easy = fs::read_to_string("hello.txt").expect("Failed to read file");
    println!("File content:\n{}", contents_easy);
    // # writing to a file
    let mut file = File::create("hello_write.txt").expect("Failed to create file");
    file.write_all(b"Hello, world!").expect("Failed to write to file");
    println!("File written successfully");

    // writing to a file easiery way
    fs::write("output.txt", "Hello, Rust!").expect("Failed to write file");

    let mut file = OpenOptions::new()
    .append(true)  // enable appending
    .open("output.txt")
    .expect("Failed to open file");
    writeln!(file, "\nAdding a new line").expect("Failed to write");


}
