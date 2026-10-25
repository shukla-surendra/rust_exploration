// Rust syntax: the shape of a Rust program
//
// Rust is a compiled language: you turn the .rs file into a program first,
// then run that program. There is no interpreter.
//
// Two ways to build and run:
// | Tool  | Command                                    | Use for                            |
// |-------|--------------------------------------------|------------------------------------|
// | rustc | rustc 00_syntax.rs && ./00_syntax          | one file, no outside libraries     |
// | cargo | cargo new app && cd app && cargo run       | real projects (see ../web-server)  |
// In this folder:  ./rs.sh 00_syntax.rs  (compiles into target/ and runs it).
//
// Install: https://rustup.rs  (gives you rustc, cargo, rustfmt, and the docs)
// Check:   rustc --version  /  cargo --version
//
// The rules this file shows:
// | Rule                                   | Example                          |
// |----------------------------------------|----------------------------------|
// | The program starts at main             | fn main() { ... }                |
// | Code blocks go inside { }              | if x > 1 { ... }                 |
// | Statements end with ;                  | let x = 5;                       |
// | A name ending in ! is a macro          | println!(...)  vec![...]         |
// | Text in double quotes is a string      | "Hello"                          |
// | Indentation is style, not syntax       | 4 spaces by convention           |
//
// Naming conventions (the compiler warns if you break them):
// | Kind                        | Style                 | Example        |
// |-----------------------------|-----------------------|----------------|
// | variables, functions        | snake_case            | user_name      |
// | types (struct, enum, trait) | UpperCamelCase        | PlayerScore    |
// | constants                   | SCREAMING_SNAKE_CASE  | MAX_PLAYERS    |
//
// Formatting: `rustfmt 00_syntax.rs` (or `cargo fmt` in a project) rewrites the
// file in the standard style, so you never argue about spaces.

// Items (functions, structs, constants) can be written in any order in the
// file: main can use add() and PlayerScore even though they come later.
const MAX_PLAYERS: u32 = 4;

// `fn` starts a function. main takes nothing and returns nothing.
fn main() {
    // A statement: does something, ends with ;
    println!("Hello, World!");

    // `let` creates a variable (03_variable.rs).
    let x = 5;
    println!("x = {}", x);

    // Statement vs expression:
    //   statement  -> does something, gives no value       (let y = 6;)
    //   expression -> produces a value                     (5 + 1, add(2, 3), { ... })
    // A block { } is an expression: its value is its LAST line, written WITHOUT ;
    let y = {
        let a = 2;
        a * 3 // no semicolon: this is the block's value (6)
    };
    println!("y = {}", y);

    // Calling a function defined below main.
    let sum = add(2, 3);
    println!("2 + 3 = {}", sum);

    // Using a type (struct) defined below main (29_structs.rs).
    let score = PlayerScore { points: 42 };
    println!("score = {} (max players: {})", score.points, MAX_PLAYERS);

    // Whitespace doesn't matter to the compiler: this works, but is hard to read.
    let     z    =    7   ;    println!("z = {}", z);
}

// Parameters have types (a: i32), and -> gives the return type.
fn add(a: i32, b: i32) -> i32 {
    a + b // last expression, no ;, is returned (16_functions.rs)
}

struct PlayerScore {
    points: u32,
}
