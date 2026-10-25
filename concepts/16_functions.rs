// Functions
//
//   fn name(param1: Type1, param2: Type2) -> ReturnType {
//       body
//   }
//
// - `fn` declares a function; names are snake_case.
// - Every parameter MUST have a type. (Types inside the body can be inferred;
//   at function boundaries they must be written, so the signature tells you
//   everything about how to call it.)
// - `-> Type` is the return type. No arrow = returns nothing, written () ("unit").
// - The LAST expression, with NO semicolon, is the return value.
//   `return x;` also works, mostly used to leave early.
//
// Statement vs expression (the semicolon rule):
// | Code            | Kind        | Value                       |
// |-----------------|-------------|-----------------------------|
// | a + b           | expression  | the sum                     |
// | a + b;          | statement   | none: the ; throws it away  |
// | let x = 5;      | statement   | none                        |
// | { ...; a + b }  | expression  | the last line's value       |
// | if c { 1 } else { 2 } | expression | 1 or 2              |
//
// What Rust does NOT have: default parameter values, named arguments,
// overloading (two functions with the same name). Use different names,
// Option parameters, or a struct of settings instead.
//
// How arguments are passed (copy / move / borrow) is its own lesson:
// 21_passing_values_to_function.rs.

// No parameters, no return value.
fn say_hello() {
    println!("Hello!");
}

// Parameters, no return value.
fn greet(name: &str, times: u32) {
    for _ in 0..times {
        println!("Hi, {name}");
    }
}

// Returns a value: the last expression, no semicolon.
fn square(x: i32) -> i32 {
    x * x
    // `x * x;` (with ;) would be error[E0308]: mismatched types,
    // expected `i32`, found `()`. The ; turns the value into nothing.
}

// Early return with `return`.
fn safe_divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return 0.0; // leave right away
    }
    a / b
}

// Return several values with a tuple (24_tuples.rs).
fn min_max(numbers: &[i32]) -> (i32, i32) {
    let mut min = numbers[0];
    let mut max = numbers[0];
    for &n in numbers {
        if n < min {
            min = n;
        }
        if n > max {
            max = n;
        }
    }
    (min, max)
}

// Return "maybe a value" with Option (32_option_result.rs).
fn find_index(items: &[&str], target: &str) -> Option<usize> {
    for (i, &item) in items.iter().enumerate() {
        if item == target {
            return Some(i);
        }
    }
    None
}

// Functions calling functions.
fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

fn ring_area(outer: f64, inner: f64) -> f64 {
    circle_area(outer) - circle_area(inner)
}

// Recursion: a function that calls itself. Needs a base case to stop.
fn factorial(n: u64) -> u64 {
    if n <= 1 {
        1 // base case
    } else {
        n * factorial(n - 1)
    }
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// A function that changes the caller's variable needs &mut (20_borrowing.rs).
fn add_bonus(score: &mut u32, bonus: u32) {
    *score += bonus;
}

// Functions are values too: this one takes another function as a parameter.
// fn(i32) -> i32 = "any function that takes an i32 and returns an i32".
fn apply_twice(f: fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

fn double(x: i32) -> i32 {
    x * 2
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Calling functions
    // ------------------------------------------------------------------
    println!("--- Example 1: calling ---");
    say_hello();
    greet("Ravi", 2);
    let s = square(7);
    println!("square(7) = {s}");
    // Arguments are checked: wrong count or type is a compile error.
    // square(7, 8);    // error[E0061]: this function takes 1 argument but 2 were supplied
    // square("7");     // error[E0308]: mismatched types

    // ------------------------------------------------------------------
    // Example 2: Return values
    // ------------------------------------------------------------------
    println!("\n--- Example 2: returning ---");
    println!("10 / 4 = {}", safe_divide(10.0, 4.0));
    println!("10 / 0 = {} (early return)", safe_divide(10.0, 0.0));
    let (lo, hi) = min_max(&[4, -2, 9, 0]); // unpack the returned tuple
    println!("min = {lo}, max = {hi}");
    let names = ["ann", "ben", "cara"];
    println!("find 'ben'  -> {:?}", find_index(&names, "ben")); // Some(1)
    println!("find 'zoe'  -> {:?}", find_index(&names, "zoe")); // None
    // A function with no -> returns (), the empty "unit" value:
    let nothing = say_hello();
    println!("say_hello() returned {:?}", nothing); // ()

    // ------------------------------------------------------------------
    // Example 3: Expressions: blocks have values
    // ------------------------------------------------------------------
    println!("\n--- Example 3: expressions ---");
    let total = {
        let price = 20;
        let tax = 3;
        price + tax // no ; : the block's value
    };
    println!("total = {total}");
    println!("ring area = {:.2}", ring_area(3.0, 2.0));

    // ------------------------------------------------------------------
    // Example 4: Recursion
    // ------------------------------------------------------------------
    println!("\n--- Example 4: recursion ---");
    println!("5! = {}", factorial(5)); // 120
    print!("fibonacci: ");
    for i in 0..10 {
        print!("{} ", fibonacci(i));
    }
    println!();

    // ------------------------------------------------------------------
    // Example 5: Changing the caller's value
    // ------------------------------------------------------------------
    println!("\n--- Example 5: &mut parameter ---");
    let mut score = 50;
    add_bonus(&mut score, 25);
    println!("score after bonus = {score}"); // 75

    // ------------------------------------------------------------------
    // Example 6: Functions as values, and closures
    // ------------------------------------------------------------------
    println!("\n--- Example 6: functions as values ---");
    println!("apply_twice(double, 3) = {}", apply_twice(double, 3)); // 12
    println!("apply_twice(square, 3) = {}", apply_twice(square, 3)); // 81
    let op: fn(i32) -> i32 = double; // store a function in a variable
    println!("op(21) = {}", op(21));
    // A CLOSURE is an unnamed function written inline: |params| body.
    // Unlike fn, it can use variables from around it (38_simple_threads.rs has more).
    let bonus = 7;
    let add_bonus_to = |x: i32| x + bonus; // uses `bonus` from outside
    println!("closure: 10 + bonus = {}", add_bonus_to(10));
    let doubled: Vec<i32> = vec![1, 2, 3].iter().map(|n| n * 2).collect();
    println!("map with a closure: {:?}", doubled);

    // ------------------------------------------------------------------
    // Example 7: Functions inside functions
    // ------------------------------------------------------------------
    println!("\n--- Example 7: nested fn ---");
    // A fn can be defined inside another; only visible there (17_scope.rs).
    // A nested fn can NOT see the outer function's variables (a closure can).
    fn celsius_to_fahrenheit(c: f64) -> f64 {
        c * 9.0 / 5.0 + 32.0
    }
    for c in [0.0, 37.0, 100.0] {
        println!("{c}C = {}F", celsius_to_fahrenheit(c));
    }
}
