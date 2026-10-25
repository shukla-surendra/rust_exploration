// Reading environment variables with std::env
//
// Environment variables are key=value text settings the OS gives every program
// (HOME, PATH, USER, ...). You can set your own for one run:
//   MY_NAME=Ravi ./target/18_std_env
//
// | Function                    | Returns                     | Use when                    |
// |-----------------------------|-----------------------------|-----------------------------|
// | env::vars()                 | iterator of (key, value)    | you want all of them        |
// | env::var("KEY")             | Result<String, VarError>    | you want one of them        |
// | env::var("KEY").unwrap_or() | String (your default)       | one, with a fallback        |
// | env::var("KEY")             | String ("" if not set)      | one, blank if missing       |
// |   .unwrap_or_default()      |                             |                             |
//
// env::var returns Result because the variable might not exist:
//   Ok(value)                 -> it is set
//   Err(VarError::NotPresent) -> it is not set
//   Err(VarError::NotUnicode) -> it is set, but isn't valid UTF-8 text (rare)

use std::env;

fn main() {
    // ------------------------------------------------------------------
    // Example 1: All environment variables
    // ------------------------------------------------------------------
    println!("--- Example 1: all variables ---");
    // vars() yields one (key, value) pair of Strings per variable.
    for (key, value) in env::vars() {
        println!("{key}: {value}");
    }

    // ------------------------------------------------------------------
    // Example 2: One variable (handle "not set" yourself)
    // ------------------------------------------------------------------
    println!("\n--- Example 2: one variable ---");
    match env::var("HOME") {
        Ok(home) => println!("HOME is {home}"),
        Err(e) => println!("HOME is not available: {e}"),
    }
    // Shorter, when you are sure it's set (panics if not, see 14_unwrap_usage.rs):
    // let home = env::var("HOME").unwrap();

    // ------------------------------------------------------------------
    // Example 3: One variable with a default value
    // ------------------------------------------------------------------
    println!("\n--- Example 3: with a default ---");
    // unwrap_or(default): the value if set, otherwise the default.
    // "Guest" is a &str, so .to_string() turns it into a String to match Ok's type.
    let name = env::var("MY_NAME").unwrap_or("Guest".to_string());
    println!("MY_NAME = {name}");
    // Try:  MY_NAME=Ravi ./target/18_std_env   -> MY_NAME = Ravi

    // Common real-world use: a config value that is a number, with a default.
    // parse() also returns Result, so it gets its own default too.
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string()) // not set     -> "8080"
        .parse()
        .unwrap_or(8080); // not a number -> 8080
    println!("PORT = {port}");

    // ------------------------------------------------------------------
    // Example 4: One variable, blank if not set
    // ------------------------------------------------------------------
    println!("\n--- Example 4: blank if not set ---");
    // unwrap_or_default(): the value if set, otherwise String's default, "" (empty).
    let value = env::var("NOT_SET_ANYWHERE").unwrap_or_default();
    // {:?} prints with quotes, so an empty String shows as "" instead of nothing.
    println!("NOT_SET_ANYWHERE = {value:?}");

    // Check whether it's set at all (true/false), without reading it.
    println!("Is MY_NAME set? {}", env::var("MY_NAME").is_ok());
}
