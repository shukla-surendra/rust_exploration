// Passing values to functions
//
// This file shows what happens to a value when you pass it to a function.
// There are three ways:
//
// 1. Borrow (&T or &mut T): the function gets a reference; the caller keeps ownership.
//    &T  = read-only;  &mut T = can change the caller's value (one at a time).
// 2. Move (T, non-Copy): ownership goes to the function; the caller can't use it after.
// 3. Copy (T, Copy type): the function gets its own copy; the caller's value is untouched.
//
// | Way    | Ownership moves? | Caller can use it after? | Function can change caller's value? |
// |--------|------------------|--------------------------|-------------------------------------|
// | Borrow | No               | Yes                      | Only with &mut                      |
// | Move   | Yes              | No                       | n/a (it now owns it)                |
// | Copy   | No (new copy)    | Yes                      | No                                  |

// Rust: Copy vs. Move
//
// | Copy types                                | Move types (by default)          |
// |--------------------------------------------|----------------------------------|
// | Integers: i32, u64                         | String                           |
// | Floating point: f32, f64                   | Vec<T>                           |
// | Boolean: bool                              | Box<T>                           |
// | Characters: char                           | Most custom structs              |
// | Tuples of Copy types, e.g. (i32, bool)     | Tuples containing non-Copy types |
//
// Rule of thumb:
// - Copy: Types that implement the Copy trait.
// - Move: Types that don't implement Copy, when passed by value.
// Note: String, Vec<T>, and Box<T> can be moved, but they can also be borrowed using & or &mut.

fn immutable_borrow(x: &i32) {
    // x is a shared (immutable) reference to 'a' in main: we can read it, but not
    // change it, and we don't take ownership, so 'a' is still usable after the call.
    // x stores the address of 'a', not the value itself:  x --> a (10)
    // *x dereferences: it follows the reference and reads the value it points to (10).
    println!("Value: {}", *x);
    // This also prints 10, not an address: println!'s {} (Display) is implemented
    // for references by forwarding to the value behind them, so the deref is automatic.
    // To see the address itself, use {:p}:  println!("{:p}", x);
    println!("Value: {}", x);
}


fn main() {
    // ------------------------------------------------------------------
    // Example 1: Immutable borrow (&T) — read without taking ownership
    // ------------------------------------------------------------------
    println!("--- Example 1: Immutable borrow ---");
    let a = 10;
    // Pass a reference to 'a' (&a): the function borrows 'a' instead of taking it.
    immutable_borrow(&a);
    // The borrow ended when the function returned, so 'a' is still ours to use.
    println!("Original value: {}", a);

    // ------------------------------------------------------------------
    // Example 2: Mutable borrow (&mut T) — change the caller's value
    // ------------------------------------------------------------------
    println!("\n--- Example 2: Mutable borrow ---");
    // Shadowing: this `let` makes a new variable that is also named 'a'. It's declared
    // `mut` because we need a mutable variable to take a mutable reference to it.
    let mut a = 10;
    // &mut a lends 'a' exclusively: while the borrow lasts, no other reference to 'a'
    // can exist, and the function may change 'a' through it.
    mutable_borrow(&mut a);
    println!("Updated value: {}", a); // 15

    // ------------------------------------------------------------------
    // Example 3: Move (non-Copy type by value) — ownership transfers
    // ------------------------------------------------------------------
    println!("\n--- Example 3: Move ---");
    // String owns heap data and does not implement Copy, so passing it by value
    // *moves* ownership into the function.
    let a = String::from("Hello Rust");
    // Move ownership of 'a' into move_ownership(); 'a' in main is no longer valid.
    move_ownership(a);
    // This would cause a compile-time error ("borrow of moved value: `a`") because
    // ownership of 'a' has moved to move_ownership().
    // println!("{}", a);

    // ------------------------------------------------------------------
    // Example 4: Copy (Copy type by value) — the function gets its own copy
    // ------------------------------------------------------------------
    println!("\n--- Example 4: Copy ---");
    // i32 implements Copy, so passing it by value copies the bits: the function gets
    // its own independent copy, and main keeps the original. No ownership is moved.
    let a = 10;
    copy_ownership(a);
    println!("Outside function: {}", a); // Works

}

fn mutable_borrow(x: &mut i32) {
    // x is a mutable (exclusive) reference to 'a' in main: we can read and change
    // 'a' through it, without taking ownership.
    // x stores the address of 'a':  x --> a (10)
    // *x dereferences: it follows the reference and reads the value it points to (10).
    println!("Value: {}", *x);
    // Prints 10 as well: Display for &mut i32 also forwards to the value behind it.
    println!("Value: {}", x);
    // Writing through the reference needs the explicit *: this changes 'a' in main.
    // (x += 5 would not compile, because x is a reference, not an i32.)
    *x += 5;
}

fn move_ownership(x: String) {
    // x now owns the String. When x goes out of scope at the end of this function,
    // the String is dropped and its heap memory is freed.
    println!("Value: {}", x);
}

fn copy_ownership(x: i32) {
    // x is a copy of main's 'a'; changing x here would not affect main's 'a'.
    println!("Inside function: {}", x);
}
