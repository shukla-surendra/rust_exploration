// Enums: a value that is ONE of several options
//
// A struct says "this AND that" (a user has a name AND an age).
// An enum says "this OR that" (a light is red OR yellow OR green).
//
//   enum Direction { North, East, South, West }   // the options ("variants")
//   let d = Direction::North;                     // a value is exactly one of them
//
// Variants can carry data, and each variant can carry different data:
// | Variant style   | Example                         | Like            |
// |-----------------|---------------------------------|-----------------|
// | no data         | Quit                            | a unit struct   |
// | tuple-like      | Write(String)                   | a tuple struct  |
// | struct-like     | Move { x: i32, y: i32 }         | a named struct  |
//
// You read an enum with `match` (12_match.rs), and the compiler makes sure you
// handle EVERY variant. Add a new variant later, and every match that forgot it
// becomes a compile error: the main reason enums are so useful.
//
// You already know two enums from the standard library:
//   enum Option<T>    { Some(T), None }      // 32_option_result.rs
//   enum Result<T, E> { Ok(T), Err(E) }      // 31_results.rs

// ---- 1. A simple enum (no data) ----
// derive: Debug to print, Clone/Copy so it copies like an i32, PartialEq for ==.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    // Methods on enums work like on structs. `self` here is the variant.
    fn next(self) -> TrafficLight {
        match self {
            TrafficLight::Red => TrafficLight::Green,
            TrafficLight::Green => TrafficLight::Yellow,
            TrafficLight::Yellow => TrafficLight::Red,
        }
    }

    fn wait_seconds(&self) -> u32 {
        match self {
            TrafficLight::Red => 30,
            TrafficLight::Yellow => 5,
            TrafficLight::Green => 25,
        }
    }
}

// ---- 2. Variants that carry data ----
#[derive(Debug)]
enum Message {
    Quit,                       // no data
    Write(String),              // one value
    ChangeColor(u8, u8, u8),    // several values
    Move { x: i32, y: i32 },    // named fields
}

impl Message {
    fn describe(&self) -> String {
        match self {
            Message::Quit => "quit".to_string(),
            Message::Write(text) => format!("write '{text}'"), // pull the data out
            Message::ChangeColor(r, g, b) => format!("color #{r:02x}{g:02x}{b:02x}"),
            Message::Move { x, y } => format!("move to ({x}, {y})"),
        }
    }
}

// ---- 3. Enum with explicit numbers (C-style) ----
// Each variant has an integer value; `as` converts it.
#[derive(Debug, Clone, Copy)]
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    ServerError = 500,
}

// ---- 4. Modeling a real thing: payment methods ----
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: String, credit: bool },
    Voucher(u32), // voucher amount
}

fn fee(p: &Payment, amount: f64) -> f64 {
    match p {
        Payment::Cash => 0.0,
        Payment::Card { credit: true, .. } => amount * 0.02, // match a field VALUE
        Payment::Card { .. } => 0.25,
        Payment::Voucher(_) => 0.0,
    }
}

// ---- 5. A recursive enum: an expression tree ----
// An enum can't contain itself directly (infinite size), so the inner values
// go in a Box (22_box_heap_allocation.rs).
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

fn eval(e: &Expr) -> f64 {
    match e {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
    }
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Simple enum, match, methods
    // ------------------------------------------------------------------
    println!("--- Example 1: traffic light ---");
    let mut light = TrafficLight::Red; // EnumName::Variant
    for _ in 0..4 {
        println!("{:?}: wait {}s", light, light.wait_seconds());
        light = light.next();
    }
    println!("is it red? {}", light == TrafficLight::Red); // needs PartialEq

    // ------------------------------------------------------------------
    // Example 2: Variants with data
    // ------------------------------------------------------------------
    println!("\n--- Example 2: messages ---");
    let inbox = vec![
        Message::Write(String::from("hello")),
        Message::Move { x: 3, y: -1 },
        Message::ChangeColor(255, 128, 0),
        Message::Quit,
    ];
    // One Vec can hold all variants: they're all the same type, Message.
    for m in &inbox {
        println!("{:<45} -> {}", format!("{:?}", m), m.describe());
    }

    // ------------------------------------------------------------------
    // Example 3: if let and matches! for one variant
    // ------------------------------------------------------------------
    println!("\n--- Example 3: if let / matches! ---");
    for m in &inbox {
        if let Message::Write(text) = m {
            println!("found a Write with {} chars", text.len());
        }
    }
    let quits = inbox.iter().filter(|m| matches!(m, Message::Quit)).count();
    println!("{quits} Quit message(s)");

    // ------------------------------------------------------------------
    // Example 4: Enums with numbers
    // ------------------------------------------------------------------
    println!("\n--- Example 4: discriminants ---");
    for s in [HttpStatus::Ok, HttpStatus::NotFound, HttpStatus::ServerError] {
        println!("{:?} = {}", s, s as i32);
    }

    // ------------------------------------------------------------------
    // Example 5: Modeling with enums
    // ------------------------------------------------------------------
    println!("\n--- Example 5: payments ---");
    let payments = [
        Payment::Cash,
        Payment::Card { last4: "4242".to_string(), credit: true },
        Payment::Card { last4: "1111".to_string(), credit: false },
        Payment::Voucher(20),
    ];
    for p in &payments {
        let label = match p {
            Payment::Cash => "cash".to_string(),
            Payment::Card { last4, credit } => format!("{} card ****{last4}", if *credit { "credit" } else { "debit" }),
            Payment::Voucher(v) => format!("voucher worth {v}"),
        };
        println!("{label:<26} fee on 100.00: {:.2}", fee(p, 100.0));
    }
    // An enum makes impossible states impossible: a Cash payment simply has no
    // card number field, so you can't forget to clear it.

    // ------------------------------------------------------------------
    // Example 6: Option is just an enum
    // ------------------------------------------------------------------
    println!("\n--- Example 6: Option ---");
    let maybe_age: Option<u32> = Some(31);
    let unknown: Option<u32> = None;
    for a in [maybe_age, unknown] {
        match a {
            Some(n) => println!("age is {n}"),
            None => println!("age unknown"),
        }
    }

    // ------------------------------------------------------------------
    // Example 7: Recursive enum
    // ------------------------------------------------------------------
    println!("\n--- Example 7: expression tree ---");
    // (2 + 3) * 4
    let expr = Expr::Mul(
        Box::new(Expr::Add(Box::new(Expr::Num(2.0)), Box::new(Expr::Num(3.0)))),
        Box::new(Expr::Num(4.0)),
    );
    println!("(2 + 3) * 4 = {}", eval(&expr));
}
