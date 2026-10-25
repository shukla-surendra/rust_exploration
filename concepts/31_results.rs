// Result: a function that can fail
//
// A function that might fail returns Result<T, E> instead of a plain value:
//   Ok(value)  -> it worked, here is the value (type T)
//   Err(error) -> it failed, here is why       (type E)
// The caller has to check which one it got before using the value.
//
//   fn divide(a: i32, b: i32) -> Result<i32, String>
//                                        ^^^  ^^^^^^
//                     success value is an i32  error is a String message
//
// More on Result (and its sibling Option) in 32_option_result.rs.
/*
In Rust, Result, Ok, and Err are not reserved keywords. They are defined in Rust's standard library.

enum Result<T, E> {
    Ok(T),  //Ok is an enum variant, not a function.
    Err(E), //Err is an enum variant, not a function.
}

*/

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        // `return` exits early. "..." is a &str (fixed text); to_string() turns
        // it into an owned String, the error type this function promised.
        return Err("Cannot divide by zero".to_string());
    }

    // Last line with NO semicolon = the value the function returns.
    Ok(a / b)
}


// Two different ways to fail, each with its own message.
// f64 = 64-bit decimal number (0.0, 99.5, ...). Note: not called from main yet;
// try adding  match withdraw(100.0, 30.0) { ... }  below to see it work.
fn withdraw(balance: f64, amount: f64) -> Result<f64, String> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }

    if amount > balance {
        return Err("Insufficient balance".to_string());
    }

    Ok(balance - amount)
}


// For comparison: a plain return type, no Result. The function can't say
// "something went wrong": -100 is just a number, and the caller can't tell an
// error from a real negative balance.
fn get_balance() -> i32 {
    -100
}

fn main() {
    // Function that returns an error
    // `match` checks which variant came back and runs the matching arm.
    // `result` / `error` are new variables holding the value inside Ok / Err.
    // You MUST handle both arms, or it won't compile.
    match divide(10, 0) {
        Ok(result) => println!("Result: {}", result),
        Err(error) => println!("Error: {}", error), // this one runs: b is 0
    }

    // Function that returns a negative number
    // No Result, so no match needed, and no way to know if -100 means trouble.
    let balance = get_balance();
    println!("Balance: {}", balance);
}
