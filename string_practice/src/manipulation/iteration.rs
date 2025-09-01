pub fn main_test() {
    //# indexing 0, 1, 2 etc does not work in rust
    let s = "नमस्ते Rust"; // Hindi + English (UTF-8)

    // Iterate over characters
    for c in s.chars() {
        println!("{}", c);
    }

    // Iterate over words
    for word in s.split_whitespace() {
        println!("{}", word);
    }
}
