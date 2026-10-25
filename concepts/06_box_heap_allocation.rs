// Box<T>: putting a value on the heap
//
// Normally a local value like `let x = 5;` lives on the STACK. Box::new(5) puts
// the 5 on the HEAP instead, and x holds a pointer to it.
//
//   let x = Box::new(5);
//
//   stack           heap
//   ┌─────────┐     ┌───┐
//   │ x: ptr ─┼───> │ 5 │
//   └─────────┘     └───┘
//
// When x goes out of scope, the Box frees the heap memory automatically
// (no free()/delete needed). The Box OWNS its value: it moves like a String
// (see 05_passing_values_to_function.rs).
//
// Boxing an i32 is just for learning. Real reasons to use Box:
// | Use                               | Example                                   |
// |-----------------------------------|-------------------------------------------|
// | Big value, cheap to move          | Box<[u8; 1_000_000]> moves only a pointer |
// | Type that contains itself         | enum List { Node(i32, Box<List>), End }   |
// | "Any type with this trait"        | Box<dyn Animal> (see 15_traits.rs)        |
//
// `*x` = dereference: follow the pointer and get the value inside.
fn main(){
    let x = Box::new(5);
    let y = Box::new(10);
    // let z = x + y; // This line will cause an error because Box<T> does not implement the Add trait for Box<T>
    // we need to dereference the boxes to get the values inside them
    let z = *x + *y; // 5 + 10, z is a plain i32 (not a Box)
    // println! can print a Box directly: it prints the value inside, not the address.
    println!("Value of x: {}", x);
    println!("Value of y: {}", y);
    println!("Value of z: {}", z);
    
    //explicitly specifying the type of the boxes
    // Box<i32> = "a Box holding an i32". These new x, y shadow the ones above
    // (see 02_variable_shadowing.rs); the old boxes are freed at the end of main.
    let x: Box<i32> = Box::new(5);
    let y: Box<i32> = Box::new(10);

    let z = *x + *y;

    println!("z = {}", z);

}
