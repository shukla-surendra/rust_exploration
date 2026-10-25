// Option and Result: values that might not be there
//
// Some functions can't always give you a value ("item 10 of a 3-item list",
// "turn "abc" into a number"). Instead of null or exceptions, Rust returns a BOX
// that says whether the value is inside, and you must open the box before you
// can use the value. The compiler won't let you skip that step.
//
//   enum Option<T> {          enum Result<T, E> {
//       Some(T),  // value        Ok(T),   // success, here's the value
//       None,     // no value     Err(E),  // failure, here's why
//   }                         }
//
// | Type         | Has value | No value | Carries a reason? | Use when                |
// |--------------|-----------|----------|-------------------|-------------------------|
// | Option<T>    | Some(v)   | None     | No                | missing is normal       |
// | Result<T, E> | Ok(v)     | Err(e)   | Yes, in e         | something can go wrong  |
//
// Both are plain enums from the standard library, always in scope (no `use`
// needed): you can write Some, None, Ok, Err directly.

use std::collections::HashMap;

// ---- Functions that RETURN Option / Result (Examples 3 and 6) ----

// Option: a user may simply not exist. That's not an error, just "nothing".
fn find_user(id: u32) -> Option<String> {
    let mut users = HashMap::new();
    users.insert(1, "Alice".to_string());
    users.insert(2, "Bob".to_string());
    // HashMap::get returns Option<&String>; cloned() turns it into Option<String>.
    users.get(&id).cloned()
}

// Result: dividing by zero is a failure, and we say WHY in the Err.
// Here the error type is String, so we can return any message.
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// `?` with Option: inside a function returning Option,
//   Some(v)? -> v (continue)
//   None?    -> return None from this function right away
fn first_char_upper(s: &str) -> Option<char> {
    let c = s.chars().next()?;
    Some(c.to_ascii_uppercase())
}

// `?` with Result: inside a function returning Result,
//   Ok(v)?  -> v (continue)
//   Err(e)? -> return Err(e) from this function right away
fn parse_and_double(s: &str) -> Result<i32, std::num::ParseIntError> {
    let n: i32 = s.parse()?;
    Ok(n * 2)
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Option from the standard library
    // ------------------------------------------------------------------
    println!("--- Example 1: Option from std ---");
    let v = vec![10, 20, 30];
    // {:?} (Debug) prints the box itself, so you can see Some(...) / None.
    println!("v.get(1)       = {:?}", v.get(1)); // Some(20): index 1 exists
    println!("v.get(10)      = {:?}", v.get(10)); // None: no index 10, nothing went wrong
    println!("v.iter().max() = {:?}", v.iter().max()); // Some(30): an empty Vec would give None
    println!("first char     = {:?}", "Rust".chars().next()); // Some('R')
    println!("first char     = {:?}", "".chars().next()); // None: empty string

    // ------------------------------------------------------------------
    // Example 2: Result from the standard library
    // ------------------------------------------------------------------
    println!("\n--- Example 2: Result from std ---");
    // parse::<i32>() = "parse into an i32" (the ::<> picks the target type).
    println!("\"42\".parse()  = {:?}", "42".parse::<i32>()); // Ok(42)
    println!("\"abc\".parse() = {:?}", "abc".parse::<i32>()); // Err(...): says WHY it failed
    println!("env HOME      = {:?}", std::env::var("HOME")); // Ok("/Users/..."), see 37_std_env.rs
    println!("env NOPE      = {:?}", std::env::var("NOPE")); // Err(NotPresent)

    // ------------------------------------------------------------------
    // Example 3: Your own functions returning Option / Result
    // ------------------------------------------------------------------
    println!("\n--- Example 3: your own functions ---");
    println!("find_user(1)  = {:?}", find_user(1)); // Some("Alice")
    println!("find_user(99) = {:?}", find_user(99)); // None
    println!("divide(10, 2) = {:?}", divide(10, 2)); // Ok(5)
    println!("divide(10, 0) = {:?}", divide(10, 0)); // Err("cannot divide by zero")

    // ------------------------------------------------------------------
    // Example 4: Opening the box with match / if let
    // ------------------------------------------------------------------
    println!("\n--- Example 4: match and if let ---");
    // match: handle every case. The compiler errors if you forget one
    // (try deleting the None arm).
    match find_user(99) {
        Some(name) => println!("match: found {name}"),
        None => println!("match: no such user"),
    }
    match divide(10, 0) {
        Ok(result) => println!("match: result {result}"),
        Err(msg) => println!("match: error: {msg}"), // the reason is inside Err
    }

    // if let: when you only care about the "has value" case.
    if let Some(name) = find_user(2) {
        println!("if let: found {name}");
    }
    // if let ... else: both cases, shorter than match for two arms.
    if let Ok(n) = "7".parse::<i32>() {
        println!("if let: parsed {n}");
    } else {
        println!("if let: not a number");
    }

    // ------------------------------------------------------------------
    // Example 5: Opening the box with defaults (no crash)
    // ------------------------------------------------------------------
    println!("\n--- Example 5: defaults ---");
    // unwrap_or(x): the value, or x if there's none.
    println!("unwrap_or:         {}", find_user(99).unwrap_or("Guest".to_string()));
    // unwrap_or_default(): the value, or the type's default ("" for String, 0 for i32).
    println!("unwrap_or_default: {:?}", find_user(99).unwrap_or_default());
    // Works the same on Result: Err is replaced by the default.
    println!("Result unwrap_or:  {}", divide(10, 0).unwrap_or(0));

    // ------------------------------------------------------------------
    // Example 6: Passing it upward with ?
    // ------------------------------------------------------------------
    println!("\n--- Example 6: the ? operator ---");
    // See first_char_upper() and parse_and_double() above.
    println!("first_char_upper(\"rust\") = {:?}", first_char_upper("rust")); // Some('R')
    println!("first_char_upper(\"\")     = {:?}", first_char_upper("")); // None
    println!("parse_and_double(\"21\")   = {:?}", parse_and_double("21")); // Ok(42)
    println!("parse_and_double(\"x\")    = {:?}", parse_and_double("x")); // Err(...)

    // ------------------------------------------------------------------
    // Example 7: Opening the box by force: unwrap / expect
    // ------------------------------------------------------------------
    println!("\n--- Example 7: unwrap and expect ---");
    // unwrap(): "I'm sure it's Some/Ok; crash (panic) if it isn't."
    println!("unwrap: {}", find_user(1).unwrap());
    // expect("msg"): same, but the crash message is yours. Easier to debug.
    println!("expect: {}", divide(9, 3).expect("9 / 3 should work"));
    // These would panic (see 33_unwrap_usage.rs, Example 8):
    // find_user(99).unwrap();               // None
    // divide(1, 0).expect("divide failed"); // Err

    // ------------------------------------------------------------------
    // Example 8: Converting between Option and Result
    // ------------------------------------------------------------------
    println!("\n--- Example 8: converting ---");
    // Option -> Result: ok_or(error) gives a reason to a None.
    let r: Result<String, &str> = find_user(99).ok_or("user not found");
    println!("ok_or: {:?}", r); // Err("user not found")
    // Result -> Option: ok() throws the error reason away.
    let o: Option<i32> = divide(10, 0).ok();
    println!("ok():  {:?}", o); // None
}

// Quick test: which type would you use?
//   Find a user by ID              -> Option (may just not exist)
//   Read a config file             -> Result (fails for a reason: not found, no permission)
//   Largest number in a list       -> Option (empty list has none; that's why max() returns Option)
//   Convert user input to a number -> Result (bad input has a reason: "invalid digit")
