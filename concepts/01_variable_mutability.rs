// Error Rust Code Index https://doc.rust-lang.org/error_codes/error-index.html
fn main(){
    let mut x = 5; //mutable varable can be changed
    println!("x = {}", x);
    x = 10;
    println!("x = {}", x);

    let y = 56; //immutable variable cannot be changed
    println!("y = {}", y);
    y = 100; //this will give an error because y is immutable
}