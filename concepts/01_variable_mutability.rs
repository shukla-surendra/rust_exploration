// Mutability: can a variable change?
//
// By default a variable is IMMUTABLE: after `let y = 56;`, y stays 56.
// Add `mut` to allow changes: `let mut x = 5;` and then `x = 10;` works.
//
// | Declaration     | `x = 10;` later?        |
// |-----------------|-------------------------|
// | let x = 5;      | No: compile error       |
// | let mut x = 5;  | Yes                     |
//
// Why immutable by default? A value that can't change can't be changed by
// surprise somewhere else. You opt in with `mut`, so changes stand out.
//
// NOTE: this file does NOT compile, on purpose. The compiler stops with:
//   error[E0384]: cannot assign twice to immutable variable `y`
// Delete the `y = 100;` line (or write `let mut y`) and it builds.
//
// Error Rust Code Index https://doc.rust-lang.org/error_codes/error-index.html
// (look up any code, like E0384, for an explanation and examples)
fn main(){
    let mut x = 5; //mutable varable can be changed
    println!("x = {}", x);
    x = 10;
    println!("x = {}", x);

    let y = 56; //immutable variable cannot be changed
    println!("y = {}", y);
    y = 100; //this will give an error because y is immutable
}
