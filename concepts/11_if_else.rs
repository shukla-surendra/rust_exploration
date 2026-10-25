// if / else if / else: choosing what runs
//
//   if condition {            // condition MUST be a bool (10_booleans.rs)
//       ...                   // runs when it's true
//   } else if other {         // checked only if the first was false
//       ...
//   } else {                  // runs when nothing above matched
//       ...
//   }
//
// Syntax rules:
// - No parentheses needed around the condition: `if x > 5`, not `if (x > 5)`.
// - The braces { } are always required, even for one line.
// - Only the FIRST true branch runs; the rest are skipped.
//
// `if` is an EXPRESSION: it produces a value, so it replaces the ?: ternary
// operator of other languages:
//   let label = if n > 0 { "positive" } else { "not positive" };
//
// | Form              | Use when                                        |
// |-------------------|-------------------------------------------------|
// | if / else         | two or a few conditions                         |
// | let x = if ...    | picking a value                                 |
// | if let Some(v)    | "if this Option/enum has a value, use it"       |
// | let ... else      | "get the value or leave the function"           |
// | match             | many cases of one value (12_match.rs)           |

fn grade(score: u32) -> char {
    // Order matters: check the highest first, else 95 would match `>= 60`.
    if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else if score >= 60 {
        'D'
    } else {
        'F'
    }
    // This whole if/else is the function's last expression, so its value is returned.
}

// let-else: unwrap a value, or leave the function early.
fn parse_age(text: &str) -> String {
    let Ok(age) = text.parse::<u32>() else {
        // The else block MUST leave: return, break, continue, or panic.
        return format!("'{text}' is not a valid age");
    };
    // From here on, `age` is a plain u32.
    format!("age is {age}")
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: if, else if, else
    // ------------------------------------------------------------------
    println!("--- Example 1: basic if ---");
    let temperature = 22;
    if temperature > 30 {
        println!("It's hot");
    } else if temperature > 15 {
        println!("It's pleasant"); // this runs
    } else {
        println!("It's cold");
    }
    // if without else: do something or nothing.
    let unread = 3;
    if unread > 0 {
        println!("You have {unread} new messages");
    }
    // if 1 { }  // error[E0308]: expected `bool`, found integer. Write `if unread != 0`.

    // ------------------------------------------------------------------
    // Example 2: Combining conditions
    // ------------------------------------------------------------------
    println!("\n--- Example 2: && and || ---");
    let age = 20;
    let has_id = true;
    if age >= 18 && has_id {
        println!("Entry allowed");
    }
    let day = "Sat";
    if day == "Sat" || day == "Sun" {
        println!("Weekend");
    }

    // ------------------------------------------------------------------
    // Example 3: if as an expression (instead of ?:)
    // ------------------------------------------------------------------
    println!("\n--- Example 3: if gives a value ---");
    let n = -4;
    let sign = if n > 0 { "positive" } else if n < 0 { "negative" } else { "zero" };
    println!("{n} is {sign}");
    // Each branch's last line has NO semicolon: that's the branch's value.
    let fee = if age < 12 { 5 } else { 10 };
    println!("fee = {fee}");
    // All branches must give the SAME type:
    // let bad = if age > 18 { 1 } else { "no" };  // error[E0308]: `if` and `else` have incompatible types
    // And a value-producing if needs an else (what would the value be otherwise?).
    println!("grades: 95={} 85={} 72={} 61={} 40={}", grade(95), grade(85), grade(72), grade(61), grade(40));

    // ------------------------------------------------------------------
    // Example 4: Nested if
    // ------------------------------------------------------------------
    println!("\n--- Example 4: nested ---");
    let logged_in = true;
    let is_admin = false;
    if logged_in {
        if is_admin {
            println!("Admin dashboard");
        } else {
            println!("User dashboard"); // this runs
        }
    } else {
        println!("Please log in");
    }
    // Deep nesting gets hard to read; `&&` or an early `return` often flattens it.

    // ------------------------------------------------------------------
    // Example 5: if let: act only when a pattern matches
    // ------------------------------------------------------------------
    println!("\n--- Example 5: if let ---");
    let scores = vec![70, 85, 92];
    // .first() returns Option<&i32>: Some(&70) here, None if the Vec were empty.
    if let Some(first) = scores.first() {
        println!("first score: {first}");
    }
    let empty: Vec<i32> = Vec::new();
    if let Some(first) = empty.first() {
        println!("first: {first}");
    } else {
        println!("no scores yet"); // runs: the pattern didn't match
    }
    // if let can be chained with conditions using a nested if:
    if let Some(&top) = scores.last() {
        if top > 90 {
            println!("last score {top} is excellent");
        }
    }

    // ------------------------------------------------------------------
    // Example 6: let-else: get the value or bail out
    // ------------------------------------------------------------------
    println!("\n--- Example 6: let else ---");
    println!("{}", parse_age("34"));
    println!("{}", parse_age("thirty"));

    // ------------------------------------------------------------------
    // Example 7: A small real-world decision
    // ------------------------------------------------------------------
    println!("\n--- Example 7: shipping cost ---");
    for (total, express) in [(120.0, false), (45.0, false), (45.0, true)] {
        let shipping = if total >= 100.0 {
            0.0 // free shipping
        } else if express {
            15.0
        } else {
            5.0
        };
        println!("order {total:>6.2} express={express:<5} -> shipping {shipping:.2}");
    }
}
