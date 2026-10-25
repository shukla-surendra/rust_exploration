// Variable shadowing
//
// variable shadowing occurs when a variable declared within a certain scope (like a block) has the same name as a variable declared in an outer scope. The inner variable "shadows" the outer variable, meaning that within the inner scope, the inner variable is used instead of the outer one.
//
// It happens two ways:
// 1. Inside a block { }: the inner `a` hides the outer `a` only until the `}`.
//    After the block, the outer `a` is visible again, unchanged.
// 2. `let a = ...` again in the same scope: a NEW `a` hides the old one for
//    the rest of the scope.
//
// Shadowing vs mut:
// | Question                 | let mut a = 5; a = 15; | let a = 5; let a = 15;   |
// |--------------------------|------------------------|--------------------------|
// | New variable?            | No, same one changes   | Yes, a second variable   |
// | Can the type change?     | No                     | Yes (see below)          |
// | Is the result mutable?   | Yes                    | No (unless `let mut`)    |
//
// Changing the type is the common real use:
//   let input = "42";               // &str (text)
//   let input: i32 = input.parse().unwrap();  // now an i32, same name
fn main(){
    let a = 5;
    {
        let a = 10; // This shadows the outer `a` scope is limited to this block
        println!("Inner a: {}", a); // Prints 10
    }
    println!("Outer a: {}", a); // Prints 5
    let a = 15; // This shadows the previous `a`
    println!("New outer a: {}", a); // Prints 15
}
