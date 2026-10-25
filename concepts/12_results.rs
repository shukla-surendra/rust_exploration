/*
In Rust, Result, Ok, and Err are not reserved keywords. They are defined in Rust's standard library.

enum Result<T, E> {
    Ok(T),  //Ok is an enum variant, not a function.
    Err(E), //Err is an enum variant, not a function.
}

*/

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("Cannot divide by zero".to_string());
    }

    Ok(a / b)
}


fn withdraw(balance: f64, amount: f64) -> Result<f64, String> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }

    if amount > balance {
        return Err("Insufficient balance".to_string());
    }

    Ok(balance - amount)
}


fn get_balance() -> i32 {
    -100
}

fn main() {
    // Function that returns an error
    match divide(10, 0) {
        Ok(result) => println!("Result: {}", result),
        Err(error) => println!("Error: {}", error),
    }

    // Function that returns a negative number
    let balance = get_balance();
    println!("Balance: {}", balance);
}
