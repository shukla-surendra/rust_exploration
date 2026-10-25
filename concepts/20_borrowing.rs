// Borrowing: using a value without taking ownership
//
// A REFERENCE (&) lets you use a value that someone else owns, like borrowing
// a book: you can read it, but you must give it back, and the owner keeps it.
//
// | Write     | Name                | Can read? | Can change? | How many at once?       |
// |-----------|---------------------|-----------|-------------|-------------------------|
// | &x        | shared reference    | Yes       | No          | as many as you like     |
// | &mut x    | mutable reference   | Yes       | Yes         | exactly ONE, and no &x  |
//
// THE borrowing rule (checked at compile time by the "borrow checker"):
//   At any moment you can have EITHER
//     - any number of shared references (&T),  OR
//     - exactly one mutable reference (&mut T).
//   And a reference must never outlive the value it points to.
//
// Why? If one part of the code could change data while another part reads it,
// you'd get data races and "the Vec moved while I held a pointer into it" bugs.
// Rust rules them out before the program ever runs.
//
// A borrow lasts until the reference's LAST USE, not until the end of the
// block (this is called non-lexical lifetimes). Example 4 shows why that helps.
//
// Common borrow-checker errors:
// | Code     | Message (short)                                         | Example |
// |----------|---------------------------------------------------------|---------|
// | E0502    | cannot borrow as mutable because also borrowed as immutable | 4   |
// | E0499    | cannot borrow as mutable more than once at a time       | 3       |
// | E0596    | cannot borrow as mutable (variable not declared `mut`)  | 2       |
// | E0597    | does not live long enough                               | 7       |
// | E0106    | missing lifetime specifier (returning a dangling &)     | 7       |

// Shared borrow: read only.
fn print_len(s: &String) {
    println!("'{s}' is {} bytes", s.len());
}

// Mutable borrow: may change the caller's value.
fn append_world(s: &mut String) {
    s.push_str(" world");
}

// Better parameter type for text: &str accepts &String and literals (18_strings.rs).
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

// Slices (&[T]) are borrows of part or all of an array / Vec.
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

// Returning a reference: the result borrows from the INPUT, so it lives as long
// as the input does. The 'a ("lifetime a") says exactly that: "the returned &str
// lives as long as both inputs". The compiler needs it here because there are
// two inputs and it can't guess which one the result comes from.
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Shared borrows (&)
    // ------------------------------------------------------------------
    println!("--- Example 1: shared borrows ---");
    let s = String::from("hello");
    print_len(&s); // lend it
    print_len(&s); // lend it again: fine
    let r1 = &s;
    let r2 = &s; // many readers at once: fine
    println!("r1 = {r1}, r2 = {r2}, s = {s}"); // the owner can read too
    println!("words: {}", count_words(&s));
    // r1.push('!');  // error[E0596]: cannot borrow `*r1` as mutable, as it is behind a `&` reference

    // ------------------------------------------------------------------
    // Example 2: Mutable borrows (&mut)
    // ------------------------------------------------------------------
    println!("\n--- Example 2: mutable borrows ---");
    let mut greeting = String::from("hello"); // the VARIABLE must be mut too
    append_world(&mut greeting);
    println!("{greeting}");
    let mut count = 0;
    let c = &mut count;
    *c += 1; // * reaches through the reference to the i32 it points to
    *c += 1;
    println!("count = {count}"); // 2
    // let fixed = String::from("x");
    // append_world(&mut fixed);  // error[E0596]: cannot borrow `fixed` as mutable

    // ------------------------------------------------------------------
    // Example 3: Only one &mut at a time
    // ------------------------------------------------------------------
    println!("\n--- Example 3: one writer ---");
    let mut data = vec![1, 2, 3];
    let w1 = &mut data;
    w1.push(4);
    // let w2 = &mut data;  // error[E0499]: cannot borrow `data` as mutable more than once
    // w1.push(5);          //  (only an error because w1 is used again after w2)
    println!("data = {:?}", data);
    // One after the other is fine: each borrow ends at its last use.
    {
        let w = &mut data;
        w.push(5);
    }
    let w = &mut data;
    w.push(6);
    println!("data = {:?}", data);

    // ------------------------------------------------------------------
    // Example 4: Readers OR a writer, never both
    // ------------------------------------------------------------------
    println!("\n--- Example 4: no reading while writing ---");
    let mut scores = vec![10, 20, 30];
    let first = &scores[0]; // shared borrow of scores...
    println!("first = {first}"); // ...last used here, so the borrow ends
    scores.push(40); // so this mutable use is now allowed
    println!("scores = {:?}", scores);
    // Swap the two lines and it fails:
    //   let first = &scores[0];
    //   scores.push(40);          // error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
    //   println!("{first}");
    // Why: push may move the Vec's items to a bigger heap block, and `first`
    // would then point at freed memory. In C++ that compiles and crashes later.
    // Fix: copy the value out first:  let first = scores[0];  (i32 is Copy)

    // Same rule in loops: you can't change a Vec while iterating over it.
    // for x in &scores { scores.push(*x); }  // error[E0502]
    let extra: Vec<i32> = scores.iter().map(|x| x * 10).collect(); // build a new one instead
    scores.extend(extra);
    println!("extended: {:?}", scores);

    // ------------------------------------------------------------------
    // Example 5: Borrowing in loops and structs
    // ------------------------------------------------------------------
    println!("\n--- Example 5: loops ---");
    let mut prices = vec![100, 250, 75];
    for p in &prices {
        print!("{p} "); // read
    }
    println!();
    for p in &mut prices {
        *p = *p * 9 / 10; // 10% off, changed in place
    }
    println!("after discount: {:?}", prices);
    println!("total: {}", sum(&prices)); // &Vec<i32> works as &[i32]
    println!("first two: {}", sum(&prices[..2])); // a slice borrows part of it

    // ------------------------------------------------------------------
    // Example 6: Borrow vs clone vs move
    // ------------------------------------------------------------------
    println!("\n--- Example 6: choosing ---");
    let report = String::from("quarterly numbers");
    let borrowed = &report; // cheap: just a pointer, report still owned here
    let cloned = report.clone(); // costs a copy: an independent String
    println!("borrowed: {borrowed}, cloned: {cloned}");
    let moved = report; // report gives up ownership
    println!("moved: {moved}");
    // Rule of thumb: borrow by default; clone when you really need a second
    // owner; move when the old variable is done anyway.

    // ------------------------------------------------------------------
    // Example 7: No dangling references
    // ------------------------------------------------------------------
    println!("\n--- Example 7: lifetimes ---");
    // A function can't return a reference to its own local variable:
    //   fn dangle() -> &String {          // error[E0106]: missing lifetime specifier
    //       let s = String::from("hi");
    //       &s                            // s is dropped at the }, &s would point at nothing
    //   }
    // Fix: return the String itself (move it out): fn no_dangle() -> String { ... }
    let a = String::from("short");
    let b = String::from("much longer");
    let winner = longest(&a, &b); // borrows from a or b
    println!("longest: {winner}");
    // This would fail, because winner might point into b after b is dropped:
    //   let winner;
    //   { let b = String::from("much longer"); winner = longest(&a, &b); } // error[E0597]
    //   println!("{winner}");
}
