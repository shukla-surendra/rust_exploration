pub fn main_test() {
    let s1 = String::from("Hello");     // growable string
    let s2 = "World";                   // string slice (&str)

    println!("{} {}", s1, s2);
}
