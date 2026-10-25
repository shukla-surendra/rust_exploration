// Ownership: who is responsible for cleaning up a value
//
// Rust has no garbage collector and you never call free(). Instead, every value
// has exactly one OWNER (a variable), and the compiler tracks it:
//
//   The three rules
//   1. Each value has one owner.
//   2. There can only be one owner at a time.
//   3. When the owner goes out of scope, the value is dropped (17_scope.rs).
//
// Assigning or passing a value does one of two things:
// | Type kind                  | `let b = a;` does | a usable after? | Examples                    |
// |----------------------------|-------------------|-----------------|-----------------------------|
// | Copy (simple, fixed size)  | copies the bits   | Yes             | i32, f64, bool, char, (i32, bool), [i32; 3] |
// | Move (owns heap data)      | MOVES ownership   | No              | String, Vec<T>, Box<T>, most structs        |
//
// Why move instead of copy? A String is a small struct on the stack pointing
// to text on the heap:
//
//   let s1 = String::from("hi");      let s2 = s1;   // MOVE
//
//   s1 ┌──────────┐                   s1 (no longer valid)
//      │ ptr ─────┼──┐                s2 ┌──────────┐
//      │ len 2    │  │                   │ ptr ─────┼──┐
//      │ cap 2    │  │                   │ len 2    │  │
//      └──────────┘  ▼                   │ cap 2    │  ▼
//                  ┌───┬───┐             └──────────┘ ┌───┬───┐
//          heap:   │ h │ i │                    heap: │ h │ i │
//                  └───┴───┘                          └───┴───┘
//
// If both s1 and s2 stayed valid, both would free the same heap text at the
// end of scope: a "double free" bug. Rust prevents it by making s1 invalid.
// Want two independent copies? Ask for it: s1.clone() copies the heap data too.
//
// Not wanting to give ownership away is very common: then you BORROW
// (20_borrowing.rs). Passing to functions in detail: 21_passing_values_to_function.rs.

#[derive(Debug)]
struct Account {
    owner: String,
    balance: u32,
}

// Takes ownership: the caller's String is moved in and dropped at the end here.
fn consume(s: String) {
    println!("consume() now owns '{s}'; it is dropped when this function ends");
}

// Gives ownership: the new String moves out to the caller.
fn create() -> String {
    let s = String::from("made inside create()");
    s // moved to the caller, not dropped
}

// Takes ownership and gives it back (works, but borrowing is usually nicer).
fn add_exclamation(mut s: String) -> String {
    s.push('!');
    s
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Copy types: both stay usable
    // ------------------------------------------------------------------
    println!("--- Example 1: copy ---");
    let a = 5;
    let b = a; // i32 is Copy: b gets its own 5
    println!("a = {a}, b = {b}"); // both fine
    let p1 = (1, true);
    let p2 = p1; // a tuple of Copy types is Copy too
    println!("p1 = {:?}, p2 = {:?}", p1, p2);

    // ------------------------------------------------------------------
    // Example 2: Move: the old variable is no longer valid
    // ------------------------------------------------------------------
    println!("\n--- Example 2: move ---");
    let s1 = String::from("hello");
    let s2 = s1; // ownership moves from s1 to s2
    // println!("{s1}");  // error[E0382]: borrow of moved value: `s1`
    println!("s2 = {s2}");

    let v1 = vec![1, 2, 3];
    let v2 = v1; // Vec moves the same way
    println!("v2 = {:?}", v2);

    // ------------------------------------------------------------------
    // Example 3: clone: a real, independent copy
    // ------------------------------------------------------------------
    println!("\n--- Example 3: clone ---");
    let original = String::from("data");
    let mut copy = original.clone(); // copies the heap text too (costs time and memory)
    copy.push_str(" (edited)");
    println!("original = '{original}', copy = '{copy}'"); // independent

    // ------------------------------------------------------------------
    // Example 4: Functions take and give ownership
    // ------------------------------------------------------------------
    println!("\n--- Example 4: functions ---");
    let name = String::from("Ravi");
    consume(name); // name MOVES into the function
    // println!("{name}");  // error[E0382]: value moved in the call above

    let made = create(); // ownership comes OUT of the function
    println!("got: '{made}'");

    let word = String::from("wow");
    let word = add_exclamation(word); // moved in, moved back out (shadowing the name)
    println!("returned: '{word}'");

    let n = 10;
    let doubled = double(n); // i32 is Copy: n is still usable
    println!("n = {n}, doubled = {doubled}");

    // ------------------------------------------------------------------
    // Example 5: Ownership and collections
    // ------------------------------------------------------------------
    println!("\n--- Example 5: collections own their items ---");
    let mut names = Vec::new();
    let first = String::from("Ann");
    names.push(first); // `first` moves INTO the Vec; the Vec now owns it
    // println!("{first}");  // error[E0382]
    names.push(String::from("Ben"));
    println!("names = {:?}", names);

    // You can't just move an item out of the middle of a Vec:
    // let taken = names[0];  // error[E0507]: cannot move out of index of `Vec<String>`
    let cloned = names[0].clone(); // a copy is fine
    let removed = names.remove(1); // or take it out for real (the Vec gives it up)
    println!("cloned = {cloned}, removed = {removed}, left = {:?}", names);

    // for x in vec  -> moves every item out, the Vec is used up (15_for_loops.rs)
    for n in names {
        println!("owned in loop: {n}");
    }
    // println!("{:?}", names);  // error[E0382]: `names` moved by the for loop

    // ------------------------------------------------------------------
    // Example 6: Structs own their fields
    // ------------------------------------------------------------------
    println!("\n--- Example 6: structs ---");
    let owner = String::from("Cara");
    let acct = Account { owner, balance: 100 }; // `owner` moves into the struct
    println!("{:?}", acct);
    let acct2 = acct; // the whole struct moves (it holds a String, so it's not Copy)
    // println!("{:?}", acct);  // error[E0382]
    println!("moved to acct2: {} has {}", acct2.owner, acct2.balance);

    // ------------------------------------------------------------------
    // Example 7: Option::take and std::mem::replace: move out, leave something behind
    // ------------------------------------------------------------------
    println!("\n--- Example 7: take and replace ---");
    let mut slot: Option<String> = Some(String::from("token"));
    let got = slot.take(); // moves the value out, leaves None in its place
    println!("got = {:?}, slot is now {:?}", got, slot);
    let mut current = String::from("old");
    let previous = std::mem::replace(&mut current, String::from("new"));
    println!("previous = {previous}, current = {current}");

    // ------------------------------------------------------------------
    // Example 8: The usual fix: borrow instead of moving
    // ------------------------------------------------------------------
    println!("\n--- Example 8: borrow instead ---");
    let title = String::from("Rust Book");
    let len = length(&title); // & lends it; ownership stays here
    println!("'{title}' has {len} bytes, and we still own it"); // details: 20_borrowing.rs
}

fn double(x: i32) -> i32 {
    x * 2
}

fn length(s: &str) -> usize {
    s.len()
}
