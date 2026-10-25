//! Comments
//!
//! This first block uses `//!`: an INNER doc comment. It documents the thing it
//! sits inside (here: the whole file/crate), so it must come at the very top.
//!
//! | Syntax        | Kind                | Documents             | Shows in rustdoc? |
//! |---------------|---------------------|-----------------------|-------------------|
//! | // text       | line comment        | nothing (for readers) | No                |
//! | /* text */    | block comment       | nothing (for readers) | No                |
//! | /// text      | outer doc comment   | the item BELOW it     | Yes               |
//! | //! text      | inner doc comment   | the item it's INSIDE  | Yes               |
//!
//! Doc comments are written in Markdown. Generate HTML docs from them with:
//!   rustdoc 02_comments.rs      (single file -> doc/ folder)
//!   cargo doc --open            (in a cargo project)
//!
//! Every page on https://doc.rust-lang.org/std is built from `///` comments
//! like the ones below.

/// Adds two numbers.
///
/// `///` documents the item right after it: this function.
/// Doc comments usually have these sections:
///
/// # Examples
///
/// ```
/// let total = add(2, 3);
/// assert_eq!(total, 5);
/// ```
///
/// In a cargo library, `cargo test` actually RUNS the code in that block,
/// so the examples in your docs can't silently go out of date.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Divides `a` by `b`.
///
/// # Panics
///
/// Panics if `b` is zero. (Document when your function can crash.)
fn divide(a: i32, b: i32) -> i32 {
    a / b
}

/// A point on a 2D grid. Doc comments work on structs and their fields too.
struct Point {
    /// Horizontal position.
    x: i32,
    /// Vertical position.
    y: i32,
}

fn main() {
    // A line comment: everything after // to the end of the line is ignored.
    let a = 10; // can also go at the end of a line of code

    /* A block comment can
       span several lines. */
    let b = /* or sit in the middle of code */ 20;

    /* Block comments can be nested: /* inner */ still inside the outer one. */

    // Commenting out code is the usual way to switch it off for a moment:
    // println!("this line does not run");

    println!("add = {}", add(a, b));
    println!("divide = {}", divide(b, a));

    let p = Point { x: 1, y: 2 };
    println!("point = ({}, {})", p.x, p.y);

    // Good comments explain WHY, not WHAT. The code already says what:
    //   bad:  i += 1; // add 1 to i
    //   good: i += 1; // skip the header row
    //
    // Common markers that editors and tools search for:
    // TODO: something to do later
    // FIXME: something known to be broken
    // NOTE: something surprising worth knowing
}
