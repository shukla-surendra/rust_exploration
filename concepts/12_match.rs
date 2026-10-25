// match: compare one value against many patterns
//
//   match value {
//       pattern1 => expression,
//       pattern2 => { several; lines }
//       _ => fallback,              // _ matches anything
//   }
//
// - Patterns are checked TOP to BOTTOM; the first one that matches wins.
// - match must be EXHAUSTIVE: every possible value must be covered, or it
//   won't compile. That's the big advantage over a chain of if/else: forgotten
//   cases become compile errors (especially with enums, 30_enums.rs).
// - match is an expression: it gives a value (all arms must have the same type).
//
// Pattern cheat sheet:
// | Pattern            | Matches                          |
// |--------------------|----------------------------------|
// | 5                  | exactly 5                        |
// | 1 | 2 | 3          | any of these                     |
// | 1..=9              | inclusive range (ints and chars) |
// | 'a'..='z'          | char range                       |
// | _                  | anything, value ignored          |
// | n                  | anything, and names it n         |
// | n if n > 100       | anything that also passes a test |
// | n @ 10..=19        | the range, and names the value n |
// | (0, y)             | tuple with 0 first, y = second   |
// | Some(x) / None     | Option variants                  |
// | Point { x, .. }    | struct, taking field x           |

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn describe_number(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1 | 2 | 3 => "small", // | = "or"
        4..=9 => "medium",    // inclusive range
        i32::MIN..=-1 => "negative",
        _ => "large", // everything else (10 and up)
    }
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Matching numbers
    // ------------------------------------------------------------------
    println!("--- Example 1: numbers ---");
    let day = 3;
    match day {
        1 => println!("Monday"),
        2 => println!("Tuesday"),
        3 => println!("Wednesday"), // this runs
        4 => println!("Thursday"),
        5 => println!("Friday"),
        6 | 7 => println!("Weekend"),
        _ => println!("Not a day"), // without _, error[E0004]: non-exhaustive patterns
    }
    for n in [0, 2, 7, 42, -5] {
        println!("{n:>3} is {}", describe_number(n));
    }

    // ------------------------------------------------------------------
    // Example 2: match gives a value
    // ------------------------------------------------------------------
    println!("\n--- Example 2: match as expression ---");
    let code = 404;
    let message = match code {
        200 => "OK",
        301 | 302 => "Redirect",
        404 => "Not Found",
        500..=599 => "Server Error",
        _ => "Unknown",
    }; // note the ; : the whole match is the right side of `let`
    println!("{code} -> {message}");

    // ------------------------------------------------------------------
    // Example 3: Binding the value, and guards
    // ------------------------------------------------------------------
    println!("\n--- Example 3: bindings and guards ---");
    for temp in [-3, 12, 25, 38] {
        let feel = match temp {
            t if t < 0 => format!("{t}: freezing"), // guard: `if` after the pattern
            t @ 0..=15 => format!("{t}: cold"),      // @: test a range AND keep the value
            t @ 16..=30 => format!("{t}: warm"),
            t => format!("{t}: hot"),                 // plain name matches anything
        };
        println!("{feel}");
    }

    // ------------------------------------------------------------------
    // Example 4: Matching chars and strings
    // ------------------------------------------------------------------
    println!("\n--- Example 4: chars and &str ---");
    for c in ['a', 'Z', '7', '?'] {
        let kind = match c {
            'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
            'a'..='z' => "lowercase consonant",
            'A'..='Z' => "uppercase letter",
            '0'..='9' => "digit",
            _ => "symbol",
        };
        println!("'{c}' -> {kind}");
    }
    let command = "stop";
    match command {
        "start" => println!("starting"),
        "stop" => println!("stopping"),
        other => println!("unknown command: {other}"),
    }
    // A String is matched via .as_str():  match my_string.as_str() { "stop" => ... }

    // ------------------------------------------------------------------
    // Example 5: Matching tuples (several values at once)
    // ------------------------------------------------------------------
    println!("\n--- Example 5: tuples ---");
    // FizzBuzz: match on two bools together instead of nested ifs.
    for n in 1..=15 {
        let out = match (n % 3 == 0, n % 5 == 0) {
            (true, true) => "FizzBuzz".to_string(),
            (true, false) => "Fizz".to_string(),
            (false, true) => "Buzz".to_string(),
            (false, false) => n.to_string(),
        }; // all 4 combinations covered: no _ needed
        print!("{out} ");
    }
    println!();

    // ------------------------------------------------------------------
    // Example 6: Matching Option
    // ------------------------------------------------------------------
    println!("\n--- Example 6: Option ---");
    let names = ["Ann", "Ben"];
    for i in [0, 5] {
        match names.get(i) {
            Some(name) => println!("index {i}: {name}"),
            None => println!("index {i}: nothing there"),
        }
    }

    // ------------------------------------------------------------------
    // Example 7: Destructuring structs
    // ------------------------------------------------------------------
    println!("\n--- Example 7: structs ---");
    for p in [Point { x: 0, y: 0 }, Point { x: 0, y: 5 }, Point { x: 3, y: 0 }, Point { x: 2, y: 2 }] {
        match p {
            Point { x: 0, y: 0 } => println!("{p:?} origin"),
            Point { x: 0, y } => println!("{p:?} on the y axis at {y}"),
            Point { x, y: 0 } => println!("{p:?} on the x axis at {x}"),
            Point { x, y } if x == y => println!("{p:?} on the diagonal"),
            Point { .. } => println!("{p:?} somewhere else"), // .. = ignore all fields
        }
    }

    // ------------------------------------------------------------------
    // Example 8: matches! and if let: shortcuts for one pattern
    // ------------------------------------------------------------------
    println!("\n--- Example 8: shortcuts ---");
    let ch = 'k';
    // matches!(value, pattern) -> bool
    println!("is '{ch}' a lowercase letter? {}", matches!(ch, 'a'..='z'));
    // if let = a match where you only care about one arm (11_if_else.rs)
    if let Some(name) = names.get(1) {
        println!("second name is {name}");
    }
}
