// A struct is just a data container.
// You can use it without `impl` if you only want to store data.
#[derive(Debug)]
struct Email {
    from: String,
    to: String,
    subject: String,
    body: String,
    is_read: bool,
}

pub fn main_test() {
    let mail = Email {
        from: "alice@example.com".to_string(),
        to: "bob@example.com".to_string(),
        subject: "Hello".to_string(),
        body: "Just checking in.".to_string(),
        is_read: false,
    };
    println !("{}", mail.from);
    println!("{:?}", mail); // {:?} works because of #[derive(Debug)]
}
