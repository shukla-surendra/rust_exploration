// type alias is used to create a new name for an existing type. 
// benefits of using type alias include improving code readability and making it easier to change the underlying type in the future.
//
// An alias is NOT a new type: Kilometers and i32 are the same thing to the
// compiler, so they mix freely:
//   let x: Kilometers = 5;
//   let y: i32 = x + 10;   // fine
//
// Where you'll see aliases in real code: shortening long types, e.g.
//   type Result<T> = std::result::Result<T, MyError>;
//   (std::io::Result<T> in 25_requests.rs is exactly this kind of alias.)
//
// Want the compiler to STOP you mixing Kilometers with plain numbers? That needs
// a new type, not an alias: `struct Kilometers(i32);` (the "newtype" pattern).
type Kilometers = i32;  
fn main(){
    let x: Kilometers = 5;
    println!("x is {} kilometers", x);
}
