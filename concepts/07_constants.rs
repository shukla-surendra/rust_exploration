// Constants
//
// `const` declares a value that never changes. It is fixed when the program is
// compiled, and can live outside any function (here: above main, usable everywhere).
//
// const vs let:
// | Question              | const                          | let                     |
// |-----------------------|--------------------------------|-------------------------|
// | Type written out?     | Required (`: u32`)             | Optional (inferred)     |
// | Can it be `mut`?      | Never                          | Yes, with `let mut`     |
// | Where can it go?      | Anywhere, including top level  | Only inside a function  |
// | Value                 | Must be known at compile time  | Can be computed at      |
// |                       | (100_000, 60 * 60, ...)        | runtime (user input...) |
// | Naming convention     | SCREAMING_SNAKE_CASE           | snake_case              |

// u32 = unsigned 32-bit integer: 0 up to about 4.29 billion, no negatives.
// 100_000: underscores are only for readability; it's the same as 100000.
const MAX_POINTS: u32 = 100_000;

fn main() {
    println!("The maximum points are: {}", MAX_POINTS);
}
