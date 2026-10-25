// Scope: where a name is visible, and when a value is cleaned up
//
// A SCOPE is the region of code where a variable can be used. In Rust a scope
// is a block: { ... }. A function body, an if branch, a loop body, or just a
// bare { } are all blocks.
//
//   fn main() {                 // main's scope starts
//       let a = 1;              // a is valid from here...
//       {                       // inner scope starts
//           let b = 2;          // b is valid from here...
//           println!("{a} {b}");// inner code sees outer variables
//       }                       // ...to here: b is DROPPED (freed)
//       // println!("{b}");     // error[E0425]: cannot find value `b` in this scope
//   }                           // ...to here: a is dropped
//
// The rules:
// | Rule                                         | Example             |
// |----------------------------------------------|---------------------|
// | A variable exists from its `let` to the `}`  | Example 1           |
// | Inner blocks can see outer variables         | Example 1           |
// | Outer code can't see inner variables         | Example 1           |
// | A function can't see another fn's variables  | Example 3           |
// | Same name in an inner block = shadowing      | Example 2           |
// | At the `}`, values are dropped, LAST first   | Example 4           |
//
// "Dropped" = the value is cleaned up: its memory (and things like open files)
// are released automatically. No garbage collector, no free(). This is the base
// of ownership (19_ownership.rs).

// Constants and statics are global: visible in every function of the file.
const APP_NAME: &str = "scope-demo";
static VERSION: u32 = 3;

// A type that prints a message when it's dropped, so we can SEE scopes end.
// `impl Drop` = code that runs automatically at cleanup time (34_traits.rs).
struct Noisy {
    name: String,
}

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  (dropping {})", self.name);
    }
}

fn helper() {
    // let x = outer;  // error: `outer` lives in main's scope, not here.
    // Functions only see their parameters, their own variables, and globals:
    println!("helper sees the globals: {APP_NAME} v{VERSION}");
}

fn make_noisy(name: &str) -> Noisy {
    let n = Noisy { name: name.to_string() };
    n // returned: ownership moves to the caller, so it is NOT dropped here
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Block scope
    // ------------------------------------------------------------------
    println!("--- Example 1: blocks ---");
    let outer = 10;
    {
        let inner = 20;
        println!("inside: outer = {outer}, inner = {inner}"); // both visible
    }
    println!("outside: outer = {outer}");
    // println!("{inner}");  // error[E0425]: cannot find value `inner` in this scope

    // Variables declared in if / loop bodies are local to them too.
    if outer > 5 {
        let message = "big";
        println!("in the if: {message}");
    }
    for i in 0..2 {
        let doubled = i * 2; // a new `doubled` every round
        println!("loop round {i}: doubled = {doubled}");
    }
    // Need a value after the block? Declare it before, or let the block return it:
    let result = {
        let temp = outer * 3;
        temp + 1 // the block's value (temp itself is dropped at the })
    };
    println!("result = {result}");

    // ------------------------------------------------------------------
    // Example 2: Shadowing inside a scope
    // ------------------------------------------------------------------
    println!("\n--- Example 2: shadowing ---");
    let level = 1;
    {
        let level = 2; // new variable, hides the outer one until the }
        println!("inner level = {level}");
    }
    println!("outer level = {level}"); // 1 again (more in 05_variable_shadowing.rs)

    // ------------------------------------------------------------------
    // Example 3: Function scope and globals
    // ------------------------------------------------------------------
    println!("\n--- Example 3: functions and globals ---");
    helper();
    println!("main sees them too: {APP_NAME} v{VERSION}");

    // ------------------------------------------------------------------
    // Example 4: Drop happens at the end of scope, in reverse order
    // ------------------------------------------------------------------
    println!("\n--- Example 4: drop order ---");
    {
        let _first = Noisy { name: "first".to_string() };
        let _second = Noisy { name: "second".to_string() };
        println!("  end of block reached");
    } // dropped here: second, THEN first (last created, first cleaned up)
    println!("after the block");

    // ------------------------------------------------------------------
    // Example 5: Dropping early, and moving out of a scope
    // ------------------------------------------------------------------
    println!("\n--- Example 5: drop() and returns ---");
    let early = Noisy { name: "early".to_string() };
    println!("  before drop()");
    drop(early); // clean up now, don't wait for the }
    println!("  after drop()");
    // println!("{}", early.name);  // error[E0382]: borrow of moved value: `early`

    // A value returned from a function outlives the function's scope.
    let kept = make_noisy("returned");
    println!("  got '{}' from make_noisy, still alive", kept.name);

    // ------------------------------------------------------------------
    // Example 6: A reference can't outlive what it points to
    // ------------------------------------------------------------------
    println!("\n--- Example 6: references and scope ---");
    let r;
    {
        let short_lived = 5;
        r = short_lived; // copies the i32 out: fine, r has its own 5
    }
    println!("r = {r}");
    // Keeping a REFERENCE instead does not compile:
    //   let r2: &i32;
    //   {
    //       let short_lived = 5;
    //       r2 = &short_lived;  // error[E0597]: `short_lived` does not live long enough
    //   }                       // short_lived is dropped here...
    //   println!("{r2}");       // ...so r2 would point at nothing
    // (Borrowing rules: 20_borrowing.rs.)
    println!("end of main: 'kept' is dropped next");
}
