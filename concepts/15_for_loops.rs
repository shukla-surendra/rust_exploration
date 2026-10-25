// for: once for each item
//
//   for item in something_iterable {
//       ...
//   }
//
// `for` works with anything that can produce items one at a time (an ITERATOR):
// ranges, arrays, Vecs, HashMaps, the chars of a string, lines of a file...
// It's the most used loop: no index to manage, no way to go out of bounds.
//
// Ranges:
// | Range              | Gives          |
// |--------------------|----------------|
// | 0..5               | 0 1 2 3 4      | (end excluded)
// | 0..=5              | 0 1 2 3 4 5    | (end included)
// | (0..5).rev()       | 4 3 2 1 0      |
// | (0..10).step_by(3) | 0 3 6 9        |
//
// Going through a collection: three ways, and they matter (19_ownership.rs):
// | Write             | item is   | Afterwards                       |
// |-------------------|-----------|----------------------------------|
// | for x in &v       | &T        | v unchanged, still usable        |
// | for x in &mut v   | &mut T    | can change items; v still usable |
// | for x in v        | T         | v is MOVED (used up), gone       |

use std::collections::HashMap;

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Ranges
    // ------------------------------------------------------------------
    println!("--- Example 1: ranges ---");
    for i in 0..5 {
        print!("{i} ");
    }
    println!(" <- 0..5");
    for i in 1..=5 {
        print!("{i} ");
    }
    println!(" <- 1..=5");
    for i in (1..=5).rev() {
        print!("{i} ");
    }
    println!(" <- reversed");
    for i in (0..=20).step_by(5) {
        print!("{i} ");
    }
    println!(" <- step_by(5)");
    // Don't need the number? Use _ (no unused-variable warning).
    for _ in 0..3 {
        print!("hi ");
    }
    println!();

    // ------------------------------------------------------------------
    // Example 2: Arrays and Vecs: borrow, mutate, consume
    // ------------------------------------------------------------------
    println!("\n--- Example 2: collections ---");
    let fruits = vec![String::from("apple"), String::from("banana"), String::from("cherry")];
    for f in &fruits {
        // f is &String: we only borrow
        print!("{f} ");
    }
    println!("| still have {} fruits", fruits.len());

    let mut prices = vec![10, 20, 30];
    for p in &mut prices {
        *p += 5; // * = change the item the reference points to
    }
    println!("prices after +5: {:?}", prices);

    for f in fruits {
        // f is String: each one MOVES out of the Vec
        print!("{} ", f.to_uppercase());
    }
    println!();
    // println!("{:?}", fruits);  // error[E0382]: borrow of moved value: `fruits`

    // ------------------------------------------------------------------
    // Example 3: enumerate: get the index too
    // ------------------------------------------------------------------
    println!("\n--- Example 3: enumerate ---");
    let players = ["Ann", "Ben", "Cara"];
    // enumerate gives (index, item) pairs, unpacked into two variables.
    for (i, name) in players.iter().enumerate() {
        println!("{}. {name}", i + 1);
    }
    // Avoid  for i in 0..players.len() { players[i] }  : enumerate is clearer
    // and can't index out of bounds.

    // ------------------------------------------------------------------
    // Example 4: Two collections side by side: zip
    // ------------------------------------------------------------------
    println!("\n--- Example 4: zip ---");
    let scores = [88, 92, 79];
    for (name, score) in players.iter().zip(scores.iter()) {
        println!("{name:<5} {score}");
    }

    // ------------------------------------------------------------------
    // Example 5: Strings and HashMaps
    // ------------------------------------------------------------------
    println!("\n--- Example 5: chars, words, maps ---");
    for c in "Rust".chars() {
        print!("[{c}] ");
    }
    println!();
    for word in "the quick brown fox".split_whitespace() {
        print!("<{word}> ");
    }
    println!();
    let mut stock = HashMap::new();
    stock.insert("pens", 12);
    stock.insert("cups", 4);
    // A HashMap gives (key, value) pairs, in no particular order (26_collections_hashmap.rs).
    for (item, qty) in &stock {
        println!("{item}: {qty}");
    }

    // ------------------------------------------------------------------
    // Example 6: Nested loops, break, continue, labels
    // ------------------------------------------------------------------
    println!("\n--- Example 6: nested ---");
    // Multiplication table
    for row in 1..=4 {
        for col in 1..=4 {
            print!("{:>4}", row * col);
        }
        println!();
    }
    // Skip and stop:
    for n in 1..=10 {
        if n % 3 == 0 {
            continue; // skip multiples of 3
        }
        if n > 8 {
            break;
        }
        print!("{n} ");
    }
    println!();
    // Labeled break out of both loops (13_loop.rs):
    'rows: for r in 0..5 {
        for c in 0..5 {
            if r + c == 6 {
                println!("first cell with r + c = 6: ({r}, {c})");
                break 'rows;
            }
        }
    }

    // ------------------------------------------------------------------
    // Example 7: Accumulating results
    // ------------------------------------------------------------------
    println!("\n--- Example 7: totals ---");
    let cart = [(2, 3.50), (1, 10.00), (4, 0.75)]; // (quantity, unit price)
    let mut total = 0.0;
    for (qty, price) in cart {
        total += qty as f64 * price;
    }
    println!("cart total = {total:.2}");
    // Find the biggest by hand:
    let temps = [18, 25, 21, 30, 19];
    let mut hottest = temps[0];
    for &t in &temps {
        // &t in the pattern copies the i32 out of the reference
        if t > hottest {
            hottest = t;
        }
    }
    println!("hottest = {hottest}");

    // ------------------------------------------------------------------
    // Example 8: The same thing with iterator methods
    // ------------------------------------------------------------------
    println!("\n--- Example 8: iterator methods ---");
    // Many loops that build a total or a new list can be written as a chain.
    // Both are fine; the chain is common in real Rust code.
    let nums = [1, 2, 3, 4, 5, 6];
    let sum: i32 = nums.iter().sum();
    let max = nums.iter().max();
    let evens_squared: Vec<i32> = nums.iter().filter(|&&n| n % 2 == 0).map(|n| n * n).collect();
    println!("sum = {sum}, max = {:?}, evens squared = {:?}", max, evens_squared);
}
