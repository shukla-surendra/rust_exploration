pub fn main_test() {
    let mut s1 = String::from("Hello");
    s1.push_str(" World");              // append a &str
    s1.push('!');                       // append a single char

    let s2 = String::from("Hi");
    let s3 = s2 + " there";             // consumes s2 || use this if you want to access s2 let s3 = s2.clone() + " there";
    // Explanation for s2 which become no longer accessible further
    /*
    The + operator in Rust is actually shorthand for calling the method:
    fn add(self, s: &str) -> String
    Notice it takes self by value, not by reference.
    That means s2 is moved into the function (consumed).
    After this, s2 is no longer valid; ownership has transferred into the new String (s3).
    So s2 cannot be used anymore.
    */

    println!("{}", s1);  // "Hello World!"
    println!("{}", s3);  // "Hi there"
    // println!("{}", s2);  // "Hi there"
}
