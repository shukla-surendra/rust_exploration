// Variables: giving a name to a value
//
// `let` creates a variable. Rust works out the type from the value (5 -> i32),
// so you don't have to write it. You can if you want: `let x: i32 = 5;`
//
// Variables are IMMUTABLE by default: once set, they can't change.
// (How to make one changeable: 01_variable_mutability.rs.)
//
// println! placeholders:
// | Placeholder | Fills in                            |
// |-------------|-------------------------------------|
// | {}          | the next argument, left to right    |
// | {x}         | the variable named x                |
// | {0}, {1}    | the argument at that position       |

fn main(){
    let x = 5;
    let y = 10;
    let z = 55;
    let _xyz = 67; // _ suppress warning: unused variable: `xyz`
    // {} {} -> filled in order: first x, then y
    println!("x = {}, y = {}", x, y);
    // {x} -> put the variable's name inside the braces
    println!("x = {x}, y = {y}");
    // {0} {1} {2} -> by position: 0 = x, 1 = y, 2 = z
    println!("x = {0}, y = {1}, z = {2}", x, y, z);
    // positions can come in any order: prints x = 55, y = 10, z = 5
    println!("x = {2}, y = {1}, z = {0}", x, y, z);
}
