// `Box<T>` is a smart pointer that stores its value on the heap instead of
// the stack. The `Box` itself (a pointer, on the stack) owns the heap data,
// so when the `Box` goes out of scope, Rust automatically frees that memory
// (no garbage collector, no manual free). Use it when a value is too large
// to move around cheaply, when you need a recursive type, or when you need
// a trait object (`Box<dyn Trait>`).
fn main(){
    // Allocates an i32 on the heap and returns a Box<i32> pointing to it.
    let first: Box<i32> = Box::new(5);
    let second: Box<i32> = Box::new(50);

    // `Box<T>` implements `Deref`, so `*first` follows the pointer to read
    // the heap value. `+` isn't defined for `Box<i32>` itself, only for `i32`,
    // so both sides must be dereferenced before adding.
    let sum: i32 = *first + *second;
    println!("sum = {}", sum);

    // `first` and `second` are dropped here, freeing their heap allocations.
}