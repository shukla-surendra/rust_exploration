// type alias is used to create a new name for an existing type. 
// benefits of using type alias include improving code readability and making it easier to change the underlying type in the future.
type Kilometers = i32;  
fn main(){
    let x: Kilometers = 5;
    println!("x is {} kilometers", x);
}