// A struct is just a data container.
// You can use it without `impl` if you only want to store data.
struct Email {
    from: String,
    to: String,
    subject: String,
    body: String,
    is_read: bool,
}

// ------------------- impl block -------------------
// We add `impl` when we want to associate behavior (methods) with the struct.
// Methods let the struct "do things", not just hold data.
impl Email {
    // Constructor-like function (convenience method)
    fn new(from: &str, to: &str, subject: &str, body: &str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            is_read: false,
        }
    }

    // A method that modifies the struct
    fn mark_as_read(&mut self) {
        self.is_read = true;
    }

    // A method that returns computed information
    fn summary(&self) -> String {
        format!("From: {}, Subject: {}", self.from, self.subject)
    }
}

// ------------------- Rule of thumb -------------------
// 1. If you only need to *store data* → just use `struct`.
// 2. If you need to *add behavior (functions tied to the data)* → use `impl`.
// 3. You could write free functions (fn summary(mail: &Email)), but
//    `impl` keeps related code grouped and is more idiomatic Rust.

pub fn main_test() {
    // Using struct directly (no impl needed if you only access fields)
    let mut mail = Email::new("alice@example.com", "bob@example.com", "Hello", "Just checking in.");

    // Using methods defined in impl
    println!("{}", mail.summary());
    mail.mark_as_read();
    println!("Is read? {}", mail.is_read);
}
