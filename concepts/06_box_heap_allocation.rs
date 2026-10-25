fn main(){
    let x = Box::new(5);
    let y = Box::new(10);
    // let z = x + y; // This line will cause an error because Box<T> does not implement the Add trait for Box<T>
    // we need to dereference the boxes to get the values inside them
    let z = *x + *y;
    println!("Value of x: {}", x);
    println!("Value of y: {}", y);
    println!("Value of z: {}", z);
    
    //explicitly specifying the type of the boxes
    let x: Box<i32> = Box::new(5);
    let y: Box<i32> = Box::new(10);

    let z = *x + *y;

    println!("z = {}", z);

}