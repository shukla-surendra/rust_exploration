// Tuples: a fixed group of values, each with its own type
//
//   let person: (&str, u32, bool) = ("Ann", 31, true);
//                 ^0    ^1   ^2     access with person.0, person.1, person.2
//
// - Fixed length: you can't add or remove items.
// - Items can be DIFFERENT types (unlike arrays and Vecs).
// - The type includes every item's type: (i32, bool) and (bool, i32) differ.
// - () is the empty tuple, called "unit": the "nothing" a function without
//   -> returns (16_functions.rs).
//
// Tuple vs array vs struct:
// | Type                | Items          | Access     | Pick it when                         |
// |---------------------|----------------|------------|--------------------------------------|
// | (i32, String, bool) | mixed types    | t.0, t.1   | a few values, short-lived, obvious   |
// | [i32; 3]            | all one type   | a[0]       | a list of the same kind of thing     |
// | struct Person {..}  | mixed, NAMED   | p.name     | it's an important thing in your code |
// Typical tuple uses: returning several values, (key, value) pairs, enumerate's
// (index, item), quick grouping in a Vec.

// Return two values at once.
fn divide_with_remainder(a: i32, b: i32) -> (i32, i32) {
    (a / b, a % b)
}

// Take a tuple as a parameter, unpacked right in the parameter list.
fn distance((x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

fn stats(values: &[f64]) -> (f64, f64, f64) {
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    (min, max, avg)
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Creating and reading
    // ------------------------------------------------------------------
    println!("--- Example 1: create and access ---");
    let person = ("Ann", 31, true); // type inferred: (&str, i32, bool)
    println!("name = {}, age = {}, active = {}", person.0, person.1, person.2);
    let point: (i32, i32) = (3, -4); // type written out
    println!("point = {:?}", point); // tuples print with {:?}
    // person.3  // error[E0609]: no field `3` on type `(&str, i32, bool)`

    // ------------------------------------------------------------------
    // Example 2: Destructuring (unpacking)
    // ------------------------------------------------------------------
    println!("\n--- Example 2: destructuring ---");
    let (name, age, active) = person; // one variable per item
    println!("{name} is {age}, active: {active}");
    let (x, _) = point; // _ = ignore that item
    println!("only x: {x}");
    let ((a, b), c) = ((1, 2), 3); // nested tuples unpack the same way
    println!("a={a} b={b} c={c}");

    // ------------------------------------------------------------------
    // Example 3: Changing a tuple
    // ------------------------------------------------------------------
    println!("\n--- Example 3: mut ---");
    let mut counter = ("clicks", 0);
    counter.1 += 1; // the whole tuple must be `mut`
    counter.1 += 1;
    println!("{:?}", counter); // ("clicks", 2)
    // Swapping two variables with a tuple:
    let mut left = 1;
    let mut right = 2;
    (left, right) = (right, left);
    println!("after swap: left={left} right={right}");

    // ------------------------------------------------------------------
    // Example 4: Returning several values from a function
    // ------------------------------------------------------------------
    println!("\n--- Example 4: multiple return values ---");
    let (q, r) = divide_with_remainder(17, 5);
    println!("17 / 5 = {q} remainder {r}");
    println!("distance = {}", distance((0.0, 0.0), (3.0, 4.0))); // 5
    let (lo, hi, avg) = stats(&[3.5, 9.0, 1.5, 6.0]);
    println!("min {lo}, max {hi}, avg {avg}");
    // Standard library functions return tuples too:
    let (before, after) = "key=value".split_once('=').unwrap();
    println!("split_once -> ({before:?}, {after:?})");

    // ------------------------------------------------------------------
    // Example 5: Tuples in collections
    // ------------------------------------------------------------------
    println!("\n--- Example 5: Vec of tuples ---");
    let mut inventory = vec![("pens", 12, 0.5), ("notebooks", 3, 2.75), ("staplers", 1, 8.0)];
    inventory.push(("tape", 6, 1.2));
    for (item, qty, price) in &inventory {
        println!("{item:<10} {qty:>3} x {price:>5.2} = {:>6.2}", *qty as f64 * price);
    }
    // Sort by the second item (quantity):
    inventory.sort_by_key(|&(_, qty, _)| qty);
    println!("by quantity: {:?}", inventory.iter().map(|t| t.0).collect::<Vec<_>>());
    // enumerate and zip produce tuples:
    for (i, ch) in "abc".chars().enumerate() {
        print!("({i}, {ch}) ");
    }
    println!();

    // ------------------------------------------------------------------
    // Example 6: Comparing and matching tuples
    // ------------------------------------------------------------------
    println!("\n--- Example 6: compare and match ---");
    // == compares item by item; < compares the first item, then the next on a tie.
    println!("(1, 2) == (1, 2): {}", (1, 2) == (1, 2));
    println!("(1, 9) < (2, 0):  {}", (1, 9) < (2, 0)); // true: 1 < 2 decides
    println!("(1, 2) < (1, 3):  {}", (1, 2) < (1, 3)); // true: tie, then 2 < 3
    for p in [(0, 0), (0, 7), (5, 0), (2, 3)] {
        let where_ = match p {
            (0, 0) => "origin",
            (0, _) => "on the y axis",
            (_, 0) => "on the x axis",
            _ => "elsewhere",
        };
        println!("{:?} is {where_}", p);
    }

    // ------------------------------------------------------------------
    // Example 7: Unit () and one-item tuples
    // ------------------------------------------------------------------
    println!("\n--- Example 7: () and (x,) ---");
    let unit = ();
    println!("unit = {:?}, size = {} bytes", unit, std::mem::size_of::<()>()); // 0
    let one = (5,); // the comma makes it a one-item tuple: type (i32,)
    // Without the comma, (5) is NOT a tuple, just the number 5 in parentheses
    // (and the compiler warns: "unnecessary parentheses").
    println!("one = {:?}, one.0 = {}", one, one.0);
}
