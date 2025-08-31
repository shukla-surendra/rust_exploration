// src/bin/scope.rs
//
// This program demonstrates:
// 1. Variables
// 2. Scope
// 3. Immutability
// 4. Shadowing
//
// Each example is in its own function, called from main().

fn main() {
    println!("--- Example 1: Variables ---");
    variables_example();

    println!("\n--- Example 2: Scope ---");
    scope_example();

    println!("\n--- Example 3: Immutability ---");
    immutability_example();

    println!("\n--- Example 4: Shadowing ---");
    shadowing_example();
}

// ---------------- Example 1 ----------------
// Variables: mutable vs immutable
fn variables_example() {
    // Immutable variable (default in Rust)
    let x = 5;
    println!("Immutable variable x = {}", x);

    // Mutable variable
    let mut y = 10;
    println!("Mutable variable y (before) = {}", y);

    y = 20; // ✅ Allowed because 'y' is mutable
    println!("Mutable variable y (after) = {}", y);
}

// ---------------- Example 2 ----------------
// Scope: inner and outer blocks
fn scope_example() {
    let outer = 100;
    println!("Outer variable: {}", outer);

    {
        let inner = 200;
        println!("Inner variable: {}", inner);

        // Inner scope can access outer scope
        println!("Access outer inside inner scope: {}", outer);
    }

    // The inner variable is dropped here
    // println!("Trying to use inner: {}", inner); // ❌ Error if uncommented
}

// ---------------- Example 3 ----------------
// Immutability: variables are immutable by default
fn immutability_example() {
    let a = 42;
    println!("Immutable variable a = {}", a);

    // a = 50; // ❌ Error: cannot assign twice to immutable variable

    println!("Immutability prevents accidental changes.");
}

// ---------------- Example 4 ----------------
// Shadowing: redeclaring with the same name
fn shadowing_example() {
    // Start with a mutable variable
    let mut x = 5;
    println!("Original mutable x = {}", x);

    x = 10; // ✅ Works because it's mutable
    println!("x after mutation = {}", x);

    // Now shadow x with a new variable (immutable by default)
    let x = x + 1;
    println!("Shadowed immutable x = {}", x);

    // Try changing it again
    // x = 20; // ❌ ERROR: cannot assign twice to immutable variable

    // But you can shadow again with 'mut' explicitly
    let mut x = x;
    x = 30; // ✅ Now works because new binding is mutable
    println!("Shadowed mutable x = {}", x);
}
