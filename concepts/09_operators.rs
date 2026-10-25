// Operators
//
// | Group        | Operators                  | Example          | Result      |
// |--------------|----------------------------|------------------|-------------|
// | Arithmetic   | + - * / %                  | 7 % 3            | 1           |
// | Assignment   | = += -= *= /= %=           | x += 2           | x = x + 2   |
// | Comparison   | == != > < >= <=            | 3 >= 2           | true        |
// | Logical      | && || !                    | true && !false   | true        |
// | Bitwise      | & | ^ ! << >>              | 6 & 3            | 2           |
// | Range        | ..  ..=                    | 1..=3            | 1, 2, 3     |
// | Reference    | &  &mut  *                 | &x, *r           | 20_borrowing|
// | Error        | ?                          | s.parse()?       | 32_option_result |
//
// Things that surprise people coming from other languages:
// - There is no ++ or --. Write x += 1.
// - Both sides must be the SAME type: 5_i32 + 2_i64 is a compile error.
// - There is no ternary (a ? b : c). `if` is an expression instead (11_if_else.rs).
// - There is no ** for powers. Use .pow() or .powi() / .powf().

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Arithmetic
    // ------------------------------------------------------------------
    println!("--- Example 1: arithmetic ---");
    let a = 17;
    let b = 5;
    println!("{a} + {b} = {}", a + b); // 22
    println!("{a} - {b} = {}", a - b); // 12
    println!("{a} * {b} = {}", a * b); // 85
    println!("{a} / {b} = {}", a / b); // 3 (integer division drops the fraction)
    println!("{a} % {b} = {}", a % b); // 2 (remainder)
    println!("-17 / 5 = {}, -17 % 5 = {}", -17 / 5, -17 % 5); // -3, -2 (rounds toward zero)
    println!("-17 rem_euclid 5 = {}", (-17_i32).rem_euclid(5)); // 3 (always positive, like math mod)
    println!("17.0 / 5.0 = {}", 17.0 / 5.0); // 3.4
    println!("2^10 = {}, 2.0^0.5 = {:.3}", 2_i32.pow(10), 2.0_f64.powf(0.5));
    println!("-a = {}, abs(-7) = {}", -a, (-7_i32).abs());

    // ------------------------------------------------------------------
    // Example 2: Assignment operators
    // ------------------------------------------------------------------
    println!("\n--- Example 2: compound assignment ---");
    let mut x = 10; // must be mut to change (04_variable_mutability.rs)
    x += 5;  println!("x += 5  -> {x}"); // 15
    x -= 3;  println!("x -= 3  -> {x}"); // 12
    x *= 2;  println!("x *= 2  -> {x}"); // 24
    x /= 5;  println!("x /= 5  -> {x}"); // 4
    x %= 3;  println!("x %= 3  -> {x}"); // 1
    // x++;  // error: Rust has no ++ operator

    // ------------------------------------------------------------------
    // Example 3: Comparison: always gives a bool
    // ------------------------------------------------------------------
    println!("\n--- Example 3: comparison ---");
    println!("5 == 5: {}   5 != 3: {}", 5 == 5, 5 != 3);
    println!("5 > 3: {}    5 < 3: {}", 5 > 3, 5 < 3);
    println!("5 >= 5: {}   3 <= 2: {}", 5 >= 5, 3 <= 2);
    // Strings and chars compare alphabetically (by Unicode value):
    println!("\"apple\" < \"banana\": {}   'a' < 'b': {}", "apple" < "banana", 'a' < 'b');
    // Max / min:
    println!("max(3, 9) = {}, min(3, 9) = {}", 3.max(9), 3.min(9));
    // let bad = 5 == 5.0;  // error[E0308]: mismatched types: integer vs float

    // ------------------------------------------------------------------
    // Example 4: Logical operators
    // ------------------------------------------------------------------
    println!("\n--- Example 4: logical ---");
    let age = 25;
    let has_ticket = true;
    println!("age >= 18 && has_ticket: {}", age >= 18 && has_ticket); // AND: both true
    println!("age < 13 || age > 65:    {}", age < 13 || age > 65); // OR: either true
    println!("!has_ticket:             {}", !has_ticket); // NOT: flips it
    // Short-circuit: && stops at the first false, || at the first true, so the
    // right side may never run. Here the division would panic, but it's skipped:
    let d = 0;
    println!("d != 0 && 10 / d > 1: {}", d != 0 && 10 / d > 1); // false, no panic

    // ------------------------------------------------------------------
    // Example 5: Bitwise operators (work on the binary digits)
    // ------------------------------------------------------------------
    println!("\n--- Example 5: bitwise ---");
    let p: u8 = 0b0110; // 6
    let q: u8 = 0b0011; // 3
    println!("p & q  = {:04b} (AND: 1 where both are 1)", p & q); // 0010
    println!("p | q  = {:04b} (OR: 1 where either is 1)", p | q); // 0111
    println!("p ^ q  = {:04b} (XOR: 1 where they differ)", p ^ q); // 0101
    println!("!p     = {:08b} (NOT: flip every bit)", !p); // 11111001
    println!("p << 1 = {} (shift left = *2)", p << 1); // 12
    println!("p >> 1 = {} (shift right = /2)", p >> 1); // 3
    // A common use: flags packed into one number.
    const READ: u8 = 0b100;
    const WRITE: u8 = 0b010;
    let perms = READ | WRITE;
    println!("can write? {}  can execute? {}", perms & WRITE != 0, perms & 0b001 != 0);

    // ------------------------------------------------------------------
    // Example 6: Types must match
    // ------------------------------------------------------------------
    println!("\n--- Example 6: mixing types ---");
    let small: i32 = 5;
    let big: i64 = 10;
    // let sum = small + big;  // error[E0308]: mismatched types, expected `i32`, found `i64`
    let sum = small as i64 + big; // convert one side first (06_data_types.rs)
    println!("{small} + {big} = {sum}");
    let count = 3; // i32
    let avg = 10.0 / count as f64; // int -> float before dividing
    println!("10.0 / {count} = {avg:.3}");

    // ------------------------------------------------------------------
    // Example 7: Precedence (order of operations)
    // ------------------------------------------------------------------
    println!("\n--- Example 7: precedence ---");
    // Highest to lowest (the usual math order, then comparison, then logic):
    //   unary - !   ->   * / %   ->   + -   ->   << >>   ->   &   ->   ^   ->   |
    //   ->   == != < > <= >=   ->   &&   ->   ||   ->   .. ..=   ->   = +=
    println!("2 + 3 * 4   = {}", 2 + 3 * 4); // 14
    println!("(2 + 3) * 4 = {}", (2 + 3) * 4); // 20: parentheses win
    println!("1 + 2 > 2 && 3 < 1 || true = {}", 1 + 2 > 2 && 3 < 1 || true); // true
    // When in doubt, add parentheses. They cost nothing and help the reader.

    // ------------------------------------------------------------------
    // Example 8: + on strings, and ranges
    // ------------------------------------------------------------------
    println!("\n--- Example 8: strings and ranges ---");
    let hello = String::from("Hello");
    let greeting = hello + ", World"; // String + &str -> String (hello is moved, 18_strings.rs)
    println!("{greeting}");
    let r1: Vec<i32> = (1..5).collect(); // .. excludes the end
    let r2: Vec<i32> = (1..=5).collect(); // ..= includes the end
    println!("1..5 = {:?}   1..=5 = {:?}", r1, r2);
    println!("is 7 in 1..=10? {}", (1..=10).contains(&7));
}
