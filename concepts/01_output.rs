// Output: printing text
//
// Printing is done with MACROS (names ending in !). They check the format string
// at compile time: a wrong number of {} is a compile error, not a runtime bug.
//
// | Macro       | Prints to | Adds newline? | Use for                              |
// |-------------|-----------|---------------|--------------------------------------|
// | println!    | stdout    | yes           | normal output                        |
// | print!      | stdout    | no            | several pieces on one line           |
// | eprintln!   | stderr    | yes           | errors and logs                      |
// | eprint!     | stderr    | no            |                                      |
// | format!     | (nothing) | no            | build a String instead of printing   |
// | dbg!        | stderr    | yes           | quick debugging: shows file:line too |
//
// stdout vs stderr: two separate output streams. Try:
//   ./target/01_output > out.txt     -> stdout goes to the file, stderr still shows on screen
//
// Format cheat sheet (x = 42, pi = 3.14159, s = "hi"):
// | Placeholder | Output        | Meaning                          |
// |-------------|---------------|----------------------------------|
// | {}          | 42            | Display: the normal, user view   |
// | {:?}        | "hi"          | Debug: the programmer view       |
// | {:#?}       | (multi-line)  | pretty Debug                     |
// | {x}         | 42            | variable by name                 |
// | {0} {1}     | by position   | reuse or reorder arguments       |
// | {:5}        | "   42"       | width 5 (numbers align right)    |
// | {:<5}       | "42   "       | align left                       |
// | {:^5}       | " 42  "       | center                           |
// | {:05}       | 00042         | pad with zeros                   |
// | {:.2}       | 3.14          | 2 decimal places                 |
// | {:+}        | +42           | always show the sign             |
// | {:x} {:X}   | 2a 2A         | hexadecimal                      |
// | {:b}        | 101010        | binary                           |
// | {:o}        | 52            | octal                            |
// | {:e}        | 3.14159e0     | scientific notation              |
// | {{ }}       | { }           | literal braces                   |

use std::io::Write; // needed for stdout().flush() in Example 7

fn main() {
    // ------------------------------------------------------------------
    // Example 1: println! and print!
    // ------------------------------------------------------------------
    println!("--- Example 1: println! and print! ---");
    println!("Hello, World!");
    print!("print! does not ");
    print!("add a newline, ");
    print!("so these join up.\n"); // \n = newline, written by hand
    println!(); // empty println! prints just a newline

    // ------------------------------------------------------------------
    // Example 2: Placeholders
    // ------------------------------------------------------------------
    println!("--- Example 2: placeholders ---");
    let name = "Ravi";
    let age = 30;
    println!("{} is {} years old", name, age); // in order
    println!("{name} is {age} years old"); // by variable name
    println!("{0} likes {1}. {1} likes {0}.", "Ann", "Ben"); // by position, reused
    println!("{n} scored {s}", n = "Cara", s = 99); // named arguments
    // println!("{} {}", name); // compile error: 2 placeholders, 1 argument

    // ------------------------------------------------------------------
    // Example 3: Display {} vs Debug {:?}
    // ------------------------------------------------------------------
    println!("\n--- Example 3: Display vs Debug ---");
    let text = "hi";
    let list = vec![1, 2, 3];
    println!("Display: {}", text); // hi
    println!("Debug:   {:?}", text); // "hi" (with quotes)
    // Vec has no Display (there's no single "right" way to show a list to users),
    // so println!("{}", list) won't compile. Debug works:
    println!("Debug:   {:?}", list); // [1, 2, 3]
    println!("Pretty:  {:#?}", list); // one item per line

    // ------------------------------------------------------------------
    // Example 4: Width, alignment, padding: printing a table
    // ------------------------------------------------------------------
    println!("\n--- Example 4: width and alignment ---");
    println!("[{:5}]  [{:<5}]  [{:^5}]  [{:>5}]", 42, 42, 42, 42);
    println!("[{:5}]  [{:<5}]  [{:^5}]  [{:>5}]", "ab", "ab", "ab", "ab"); // text aligns LEFT by default
    println!("[{:05}]  [{:*^9}]", 42, "mid"); // zero padding; * as fill character
    let width = 8;
    println!("[{:>width$}]", "dyn"); // width taken from a variable
    println!("{:<8}|{:>6}|{:>8}", "item", "qty", "price");
    println!("{:<8}|{:>6}|{:>8.2}", "apple", 3, 0.5);
    println!("{:<8}|{:>6}|{:>8.2}", "melon", 12, 2.25);

    // ------------------------------------------------------------------
    // Example 5: Numbers
    // ------------------------------------------------------------------
    println!("\n--- Example 5: numbers ---");
    let pi = 3.14159;
    println!("{:.2}  {:.0}  {:8.3}", pi, pi, pi); // 3.14  3  "   3.142"
    println!("{:+}  {:+}", 5, -5); // +5  -5
    println!("hex {:x}, HEX {:X}, with prefix {:#x}", 255, 255, 255); // ff FF 0xff
    println!("binary {:b}, padded {:08b}, octal {:o}", 5, 5, 8); // 101 00000101 10
    println!("scientific {:e}", 1234.5); // 1.2345e3

    // ------------------------------------------------------------------
    // Example 6: Special characters
    // ------------------------------------------------------------------
    println!("\n--- Example 6: escapes ---");
    println!("Tab:\tafter tab");
    println!("Quote: \"inside\"   Backslash: \\");
    println!("Braces in output: {{ and }}"); // {{ prints {
    println!("Unicode: \u{1F980} \u{00E9}"); // 🦀 é
    // Raw string r"...": backslashes are kept as typed (handy for paths, regex).
    println!(r"C:\Users\new\table");
    // r#"..."# lets you put " inside without escaping.
    println!(r#"He said "hi""#);
    // A string can span lines; a \ at the line end skips the newline and the
    // next line's leading spaces.
    println!("one long line \
              written on two");

    // ------------------------------------------------------------------
    // Example 7: format!, eprintln!, flush, dbg!
    // ------------------------------------------------------------------
    println!("\n--- Example 7: format!, eprintln!, flush, dbg! ---");
    // format! returns a String; nothing is printed.
    let msg = format!("{} has {} items", "cart", 3);
    println!("built with format!: {msg}");

    eprintln!("this goes to stderr (still shows in the terminal)");

    // stdout is LINE-buffered: print! without a newline may not appear yet.
    // flush() pushes it out now (matters for prompts like "Enter name: ").
    print!("Loading... ");
    std::io::stdout().flush().unwrap(); // unwrap: 33_unwrap_usage.rs
    println!("done");

    // dbg! prints file, line, the expression and its value to stderr, then
    // RETURNS the value, so you can wrap it around anything.
    let doubled = dbg!(age * 2); // [01_output.rs:<line>:<column>] age * 2 = 60
    println!("doubled = {doubled}");
}
