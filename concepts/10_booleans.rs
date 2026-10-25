// Booleans: true and false
//
// The bool type has exactly two values: true and false. It takes 1 byte.
// You get bools from comparisons (x > 5), logic (a && b), and methods that
// ask a yes/no question (s.is_empty(), v.contains(&3)).
//
// Truth table:
// | a     | b     | a && b | a || b | !a    | a ^ b (exactly one) |
// |-------|-------|--------|--------|-------|---------------------|
// | true  | true  | true   | true   | false | false               |
// | true  | false | false  | true   | false | true                |
// | false | true  | false  | true   | true  | true                |
// | false | false | false  | false  | true  | false               |
//
// No "truthiness": unlike JavaScript/Python/C, 0, "", and empty lists are not
// false. A condition MUST be a bool:
//   if 1 { }         // error[E0308]: expected `bool`, found integer
//   if count != 0 { } // say what you mean

// Functions that answer a yes/no question return bool. By convention their
// names start with is_, has_, can_ ...
fn is_even(n: i32) -> bool {
    n % 2 == 0 // the comparison itself is the bool, no if needed
}

fn can_vote(age: u32, is_citizen: bool) -> bool {
    age >= 18 && is_citizen
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Creating bools
    // ------------------------------------------------------------------
    println!("--- Example 1: creating bools ---");
    let is_rust_fun = true;
    let is_raining: bool = false; // type written out
    let is_big = 100 > 50; // from a comparison
    println!("{is_rust_fun} {is_raining} {is_big}");
    println!("is_even(4) = {}, is_even(7) = {}", is_even(4), is_even(7));
    println!("can_vote(20, true) = {}, can_vote(16, true) = {}", can_vote(20, true), can_vote(16, true));

    // ------------------------------------------------------------------
    // Example 2: Using bools in conditions
    // ------------------------------------------------------------------
    println!("\n--- Example 2: in if ---");
    let logged_in = true;
    let is_admin = false;
    if logged_in && is_admin {
        println!("show admin panel");
    } else if logged_in {
        println!("show user dashboard"); // this runs
    } else {
        println!("show login page");
    }
    // Write `if logged_in`, not `if logged_in == true`: it's already a bool.

    // ------------------------------------------------------------------
    // Example 3: Bools as values
    // ------------------------------------------------------------------
    println!("\n--- Example 3: bools are values ---");
    // Store a comparison's result to give it a name; easier to read than one long if.
    let temperature = 31;
    let is_hot = temperature > 30;
    let is_weekend = true;
    let go_to_beach = is_hot && is_weekend;
    println!("go to beach? {go_to_beach}");
    // Flip a bool with !
    let mut light_on = false;
    light_on = !light_on;
    println!("light on after toggle: {light_on}");
    // bool -> number
    println!("true as u8 = {}, false as u8 = {}", true as u8, false as u8);
    // Count how many are true:
    let answers = [true, false, true, true];
    let correct = answers.iter().filter(|&&a| a).count();
    println!("{correct} of {} answers correct", answers.len());

    // ------------------------------------------------------------------
    // Example 4: Methods that return bool
    // ------------------------------------------------------------------
    println!("\n--- Example 4: yes/no methods ---");
    let name = "Rustacean";
    let nums = vec![3, 8, 12, 5];
    println!("name.is_empty()        = {}", name.is_empty());
    println!("name.starts_with(\"Rust\") = {}", name.starts_with("Rust"));
    println!("nums.contains(&8)      = {}", nums.contains(&8));
    println!("any > 10? {}  all > 1? {}", nums.iter().any(|&n| n > 10), nums.iter().all(|&n| n > 1));
    println!("'x'.is_alphabetic()    = {}", 'x'.is_alphabetic());
    // matches! checks a value against a pattern and gives a bool:
    let grade = 'B';
    println!("is grade A or B? {}", matches!(grade, 'A' | 'B'));

    // ------------------------------------------------------------------
    // Example 5: bool -> Option with then / then_some
    // ------------------------------------------------------------------
    println!("\n--- Example 5: then_some ---");
    // cond.then_some(v): Some(v) if cond is true, None if false (32_option_result.rs)
    let score = 72;
    let bonus = (score > 70).then_some(10);
    println!("bonus = {:?}", bonus); // Some(10)
    println!("bonus = {:?}", (score > 90).then_some(10)); // None

    // ------------------------------------------------------------------
    // Example 6: Parsing and printing
    // ------------------------------------------------------------------
    println!("\n--- Example 6: text <-> bool ---");
    let parsed: bool = "true".parse().unwrap();
    println!("\"true\" parses to {parsed}");
    println!("\"yes\" parses to {:?}", "yes".parse::<bool>()); // Err: only "true"/"false"
    println!("as text: {}", false.to_string());

    // ------------------------------------------------------------------
    // Example 7: Simplifying conditions (De Morgan's laws)
    // ------------------------------------------------------------------
    println!("\n--- Example 7: De Morgan ---");
    // !(a && b) == !a || !b     and     !(a || b) == !a && !b
    let a = true;
    let b = false;
    println!("!(a && b) = {}  !a || !b = {}", !(a && b), !a || !b);
    println!("!(a || b) = {}  !a && !b = {}", !(a || b), !a && !b);
}
