// loop: repeat forever, until you break
//
// Rust has three loops:
// | Loop             | Runs                         | Use when                                   |
// |------------------|------------------------------|--------------------------------------------|
// | loop { }         | forever, until `break`       | retrying, waiting, "run until done"        |
// | while c { }      | while condition c is true    | a state changes (14_while_loops.rs)        |
// | for x in .. { }  | once per item                | a range or collection (15_for_loops.rs)    |
//
// Loop control (works in all three):
// | Keyword          | Does                                                   |
// |------------------|--------------------------------------------------------|
// | break            | leave the loop now                                     |
// | break value      | leave and give a value (ONLY in `loop`)                |
// | continue         | skip the rest of this round, start the next            |
// | 'name: loop      | label a loop, so break/continue can target an OUTER one |
//
// Why `loop` and not `while true`? The compiler knows `loop` only ends via
// break, so it can check things like "this variable is always set after the loop"
// and allows `break value`.

fn main() {
    // ------------------------------------------------------------------
    // Example 1: A loop that stops with break
    // ------------------------------------------------------------------
    println!("--- Example 1: break ---");
    let mut count = 0;
    loop {
        count += 1;
        println!("count = {count}");
        if count == 3 {
            break; // without this, it would run forever (stop with Ctrl+C)
        }
    }
    println!("loop ended at {count}");

    // ------------------------------------------------------------------
    // Example 2: break with a value: loop is an expression
    // ------------------------------------------------------------------
    println!("\n--- Example 2: break value ---");
    // Find the first power of 2 above 1000.
    let mut n = 1;
    let result = loop {
        n *= 2;
        if n > 1000 {
            break n; // the loop's value
        }
    };
    println!("first power of 2 above 1000: {result}"); // 1024

    // ------------------------------------------------------------------
    // Example 3: continue
    // ------------------------------------------------------------------
    println!("\n--- Example 3: continue ---");
    let mut i = 0;
    loop {
        i += 1; // change the counter BEFORE continue, or it loops forever
        if i > 10 {
            break;
        }
        if i % 2 == 0 {
            continue; // even: skip the println below
        }
        print!("{i} ");
    }
    println!("(odd numbers only)");

    // ------------------------------------------------------------------
    // Example 4: Labeled loops: break out of an outer loop
    // ------------------------------------------------------------------
    println!("\n--- Example 4: labels ---");
    // Find the first pair (a, b) where a * b == 24, searching a grid.
    let mut found = None;
    'outer: for a in 1..10 {
        for b in 1..10 {
            if a * b == 24 {
                found = Some((a, b));
                break 'outer; // a plain `break` would only leave the inner loop
            }
        }
    }
    println!("first pair with product 24: {:?}", found); // Some((3, 8))
    // A label can also return a value from a labeled loop:
    let mut tries = 0;
    let steps = 'search: loop {
        loop {
            tries += 1;
            if tries == 5 {
                break 'search tries * 10;
            }
        }
    };
    println!("steps = {steps}");

    // ------------------------------------------------------------------
    // Example 5: Retry pattern
    // ------------------------------------------------------------------
    println!("\n--- Example 5: retry until success ---");
    // Pretend these are network calls; only the 3rd one succeeds.
    let responses = [Err("timeout"), Err("timeout"), Ok("data"), Ok("more data")];
    let mut attempt = 0;
    let data = loop {
        let response = responses[attempt];
        attempt += 1;
        match response {
            Ok(d) => break d,
            Err(e) if attempt < 5 => println!("attempt {attempt} failed: {e}, retrying"),
            Err(e) => panic!("giving up: {e}"),
        }
    };
    println!("got '{data}' after {attempt} attempts");

    // ------------------------------------------------------------------
    // Example 6: A simple menu / state machine
    // ------------------------------------------------------------------
    println!("\n--- Example 6: menu loop ---");
    // Real programs read user input here; we use a list of pretend inputs.
    let inputs = ["add", "add", "show", "oops", "quit", "add"];
    let mut total = 0;
    let mut next = 0;
    loop {
        let choice = inputs[next];
        next += 1;
        match choice {
            "add" => total += 1,
            "show" => println!("total = {total}"),
            "quit" => {
                println!("bye (the last \"add\" never runs)");
                break;
            }
            other => println!("unknown choice '{other}'"),
        }
    }
}
