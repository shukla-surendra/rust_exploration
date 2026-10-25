fn main(){
    let x = 5;
    let y = 10;
    let z = 55;
    let _xyz = 67; // _ suppress warning: unused variable: `xyz`
    println!("x = {}, y = {}", x, y);
    println!("x = {x}, y = {y}");
    println!("x = {0}, y = {1}, z = {2}", x, y, z);
    println!("x = {2}, y = {1}, z = {0}", x, y, z);
}