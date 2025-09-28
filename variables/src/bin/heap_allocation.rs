fn main(){
    let first: Box<i32> = Box::new(5);
    let second: Box<i32> = Box::new(50);
    let sum: i32 = *first + *second; 
    println!("sum = {}", sum);
}