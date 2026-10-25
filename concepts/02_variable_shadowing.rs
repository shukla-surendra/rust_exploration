// variable shadowing occurs when a variable declared within a certain scope (like a block) has the same name as a variable declared in an outer scope. The inner variable "shadows" the outer variable, meaning that within the inner scope, the inner variable is used instead of the outer one.
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