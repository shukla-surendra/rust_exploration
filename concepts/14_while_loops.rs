// while: repeat as long as a condition is true
//
//   while condition {     // checked BEFORE every round
//       ...               // if false at the start, the body never runs
//   }
//
// | Form                     | Keeps going while                     |
// |--------------------------|---------------------------------------|
// | while x < 10 { }         | the bool condition is true            |
// | while let Some(v) = .. { }| the pattern matches (e.g. not None)  |
//
// while vs for vs loop:
// - Know the items or the range up front? Use `for` (15_for_loops.rs). It can't
//   go out of bounds or forget to increment.
// - Repeating until some state changes (balance runs out, queue empties)? `while`.
// - No natural condition, stop from the inside? `loop` (13_loop.rs).
//
// break and continue work in while too. `break value` does not: a while loop
// might end because its condition is false, and then there'd be no value.

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Counting
    // ------------------------------------------------------------------
    println!("--- Example 1: count up ---");
    let mut i = 1;
    while i <= 5 {
        print!("{i} ");
        i += 1; // forget this and the loop never ends
    }
    println!();

    // ------------------------------------------------------------------
    // Example 2: Countdown
    // ------------------------------------------------------------------
    println!("\n--- Example 2: countdown ---");
    let mut n = 3;
    while n > 0 {
        println!("{n}...");
        n -= 1;
    }
    println!("Liftoff!");

    // ------------------------------------------------------------------
    // Example 3: The body may never run
    // ------------------------------------------------------------------
    println!("\n--- Example 3: condition false from the start ---");
    let mut stock = 0;
    while stock > 0 {
        println!("selling one"); // never printed
        stock -= 1;
    }
    println!("stock was {stock}, loop body ran 0 times");

    // ------------------------------------------------------------------
    // Example 4: Loop until some state changes
    // ------------------------------------------------------------------
    println!("\n--- Example 4: until the money runs out ---");
    let mut balance = 100;
    let price = 30;
    let mut bought = 0;
    while balance >= price {
        balance -= price;
        bought += 1;
    }
    println!("bought {bought}, left with {balance}"); // bought 3, left with 10

    // How many years to double savings at 7% a year?
    let mut money = 1000.0;
    let mut years = 0;
    while money < 2000.0 {
        money *= 1.07;
        years += 1;
    }
    println!("doubles after {years} years ({money:.2})");

    // ------------------------------------------------------------------
    // Example 5: break and continue in while
    // ------------------------------------------------------------------
    println!("\n--- Example 5: break and continue ---");
    let readings = [12, -1, 15, 999, 20, 18];
    let mut idx = 0;
    while idx < readings.len() {
        let r = readings[idx];
        idx += 1; // move on first, so `continue` can't skip it
        if r < 0 {
            println!("skipping bad reading {r}");
            continue;
        }
        if r == 999 {
            println!("end marker found, stopping");
            break;
        }
        println!("reading: {r}");
    }

    // ------------------------------------------------------------------
    // Example 6: while let: loop while a pattern matches
    // ------------------------------------------------------------------
    println!("\n--- Example 6: while let ---");
    // Vec::pop() returns Some(last item), or None once the Vec is empty.
    // This is the standard way to use a Vec as a stack.
    let mut stack = vec!["first", "second", "third"];
    while let Some(top) = stack.pop() {
        println!("popped {top}"); // third, second, first (last in, first out)
    }
    println!("stack is now empty: {:?}", stack);

    // Working through a to-do queue that grows while you process it:
    let mut tasks = vec![3];
    while let Some(t) = tasks.pop() {
        print!("task {t} ");
        if t > 1 {
            tasks.push(t - 1); // add more work
        }
    }
    println!();

    // ------------------------------------------------------------------
    // Example 7: Digits of a number (a classic while)
    // ------------------------------------------------------------------
    println!("\n--- Example 7: digits ---");
    let mut num = 9_041;
    let original = num;
    let mut digit_sum = 0;
    while num > 0 {
        digit_sum += num % 10; // last digit
        num /= 10; // drop the last digit
    }
    println!("digit sum of {original} = {digit_sum}"); // 9+0+4+1 = 14

    // Collatz: halve if even, 3n+1 if odd, until you reach 1.
    let mut c: u64 = 27;
    let mut steps = 0;
    while c != 1 {
        c = if c % 2 == 0 { c / 2 } else { 3 * c + 1 };
        steps += 1;
    }
    println!("27 reaches 1 after {steps} steps");
}
