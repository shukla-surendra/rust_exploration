// Structs: your own data types
//
// A struct groups related values under one name. Instead of passing around
// `name`, `email`, `age` as three loose variables, you make one `User` that
// holds all three. Each value inside is a FIELD, and each field has a type.
//
//   struct User {            let u = User {
//       name: String,            name: String::from("Alice"),
//       age: u32,                age: 30,
//   }                        };
//   ^ the blueprint          ^ an INSTANCE (an actual value built from it)
//
// Three kinds of struct:
// | Kind          | Looks like                    | Access fields by | Use when                    |
// |---------------|-------------------------------|------------------|-----------------------------|
// | Named fields  | struct User { name: String }  | u.name           | the usual case              |
// | Tuple struct  | struct Point(i32, i32);       | p.0, p.1         | small, order is obvious     |
// | Unit struct   | struct Marker;                | (no fields)      | behavior only (34_traits.rs)|
//
// Methods: functions that belong to a struct go in an `impl` block.
// | First parameter | Meaning                              | Call it as         |
// |-----------------|--------------------------------------|--------------------|
// | &self           | borrow, read only                    | u.greet()          |
// | &mut self       | borrow, can change fields            | u.birthday()       |
// | self            | take ownership (u is used up)        | u.into_name()      |
// | (none)          | "associated function", no instance   | User::new(...)     |

// #[derive(...)] asks the compiler to write some standard code (trait impls) for us:
//   Debug     -> print with {:?} and {:#?}
//   Clone     -> make a full copy with .clone()
//   PartialEq -> compare with == and !=
// Without Debug, println!("{:?}", user) would not compile.
#[derive(Debug, Clone, PartialEq)]
struct User {
    name: String,          // owned text: the struct owns its String
    email: String,
    age: u32,
    active: bool,
    nickname: Option<String>, // optional field: Some("Al") or None (see 32_option_result.rs)
}

// Methods and associated functions for User.
impl User {
    // Associated function (no self): called as User::new(...). Rust has no
    // constructors; `new` is just a naming convention for this kind of function.
    // `Self` = the type this impl is for (User).
    fn new(name: &str, email: &str, age: u32) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
            age,          // field init shorthand: same as `age: age`
            active: true, // default values the caller doesn't have to pass
            nickname: None,
        }
    }

    // &self: reads fields, changes nothing. The caller keeps their User.
    fn greet(&self) -> String {
        // match on the Option field: use the nickname if there is one.
        match &self.nickname {
            Some(nick) => format!("Hi, I'm {} ({})", self.name, nick),
            None => format!("Hi, I'm {}", self.name),
        }
    }

    // &mut self: can change fields. The caller's variable must be `let mut`.
    fn birthday(&mut self) {
        self.age += 1;
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    // self (no &): takes ownership. After calling this, the User is gone;
    // we pull the name String out of it and hand it back.
    fn into_name(self) -> String {
        self.name
    }
}

// Tuple struct: fields have no names, only positions.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point(i32, i32);

impl Point {
    fn distance_from_origin(&self) -> f64 {
        // `as f64` converts the i32 into a decimal number so we can take a square root.
        let x = self.0 as f64;
        let y = self.1 as f64;
        (x * x + y * y).sqrt()
    }
}

// Newtype: a tuple struct with one field, used to make a DISTINCT type.
// Unlike `type Kilometers = i32;` (08_type_alias.rs), Meters and Feet can't be
// mixed by accident: passing Feet where Meters is expected is a compile error.
#[derive(Debug)]
struct Meters(f64);
#[derive(Debug)]
struct Feet(f64);

fn feet_to_meters(f: Feet) -> Meters {
    Meters(f.0 * 0.3048)
}

// Unit struct: no fields at all. Takes no memory; useful as a type to attach
// behavior to (see 34_traits.rs, where Dog and Cat are unit structs).
#[derive(Debug)]
struct Logger;

impl Logger {
    fn log(&self, msg: &str) {
        println!("[log] {msg}");
    }
}

// A struct can hold other structs.
#[derive(Debug)]
struct Rectangle {
    top_left: Point,
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // Methods can take other parameters after self.
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    // Associated function returning Self: a square is a rectangle with equal sides.
    fn square(top_left: Point, size: u32) -> Self {
        Self { top_left, width: size, height: size }
    }
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Defining and creating a struct, reading fields
    // ------------------------------------------------------------------
    println!("--- Example 1: create and read ---");
    // Every field must be given a value; the order doesn't have to match the definition.
    let alice = User {
        email: String::from("alice@example.com"),
        name: String::from("Alice"),
        age: 30,
        active: true,
        nickname: Some(String::from("Al")),
    };
    // Dot syntax reads a field.
    println!("name = {}, age = {}, active = {}", alice.name, alice.age, alice.active);

    // ------------------------------------------------------------------
    // Example 2: Mutability is for the WHOLE instance
    // ------------------------------------------------------------------
    println!("\n--- Example 2: mutability ---");
    // You can't mark one field as mut; `let mut` makes every field changeable.
    let mut bob = User::new("Bob", "bob@example.com", 25);
    bob.email = String::from("bob@work.example.com"); // needs `let mut bob`
    bob.birthday(); // &mut self method, also needs `let mut bob`
    println!("bob: email = {}, age = {}", bob.email, bob.age); // age = 26
    // alice.age = 31;  // error[E0594]: cannot assign to `alice.age`, as `alice` is not declared as mutable

    // ------------------------------------------------------------------
    // Example 3: Printing with Debug ({:?} and {:#?})
    // ------------------------------------------------------------------
    println!("\n--- Example 3: Debug printing ---");
    // {} needs the Display trait, which structs don't get automatically.
    // {:?} uses Debug (from #[derive(Debug)]): everything on one line.
    println!("{:?}", bob);
    // {:#?} = "pretty" Debug: one field per line, easier to read for big structs.
    println!("{:#?}", alice);

    // ------------------------------------------------------------------
    // Example 4: Methods and associated functions
    // ------------------------------------------------------------------
    println!("\n--- Example 4: methods ---");
    println!("{}", alice.greet()); // Hi, I'm Alice (Al)
    println!("{}", bob.greet()); // Hi, I'm Bob   (nickname is None)
    bob.deactivate();
    println!("bob active? {}", bob.active); // false

    // ------------------------------------------------------------------
    // Example 5: Struct update syntax (..)
    // ------------------------------------------------------------------
    println!("\n--- Example 5: struct update syntax ---");
    // Build a new User from an existing one, changing only some fields.
    // `..alice.clone()` = "take all the other fields from a copy of alice".
    // We clone because name/nickname are Strings: without clone they would
    // MOVE out of alice, and alice couldn't be used any more (Example 9).
    let alice_twin = User {
        email: String::from("twin@example.com"),
        ..alice.clone()
    };
    println!("twin: {} <{}>, age {}", alice_twin.name, alice_twin.email, alice_twin.age);
    // PartialEq (from derive) lets us compare whole structs field by field.
    println!("alice == twin? {}", alice == alice_twin); // false: emails differ
    println!("alice == alice.clone()? {}", alice == alice.clone()); // true

    // ------------------------------------------------------------------
    // Example 6: Tuple structs and newtypes
    // ------------------------------------------------------------------
    println!("\n--- Example 6: tuple structs ---");
    let p = Point(3, 4);
    println!("p = {:?}, x = {}, y = {}", p, p.0, p.1);
    println!("distance from origin = {}", p.distance_from_origin()); // 5
    // Point derives Copy (all its fields are Copy), so `q = p` copies; p is still usable.
    let q = p;
    println!("p = {:?}, q = {:?}, equal? {}", p, q, p == q);

    let height = feet_to_meters(Feet(6.0));
    println!("6 feet = {:?}", height); // Meters(1.8288...)
    println!("as a plain number: {:.2} m", height.0); // .0 gets the f64 out; {:.2} = 2 decimals
    // feet_to_meters(Meters(2.0));  // error[E0308]: mismatched types: expected `Feet`, found `Meters`

    // ------------------------------------------------------------------
    // Example 7: Unit struct
    // ------------------------------------------------------------------
    println!("\n--- Example 7: unit struct ---");
    let logger = Logger; // no fields, so no { }
    logger.log("unit structs carry behavior, not data");
    println!("size of Logger = {} bytes", std::mem::size_of::<Logger>()); // 0

    // ------------------------------------------------------------------
    // Example 8: Structs inside structs
    // ------------------------------------------------------------------
    println!("\n--- Example 8: nested structs ---");
    let big = Rectangle { top_left: Point(0, 0), width: 10, height: 5 };
    let small = Rectangle::square(Point(1, 1), 3);
    // Chain dots to reach a field of a field.
    println!("big starts at x = {}", big.top_left.0);
    println!("big area = {}, small area = {}", big.area(), small.area()); // 50, 9
    println!("big can hold small? {}", big.can_hold(&small)); // true
    println!("small can hold big? {}", small.can_hold(&big)); // false

    // ------------------------------------------------------------------
    // Example 9: Ownership and structs
    // ------------------------------------------------------------------
    println!("\n--- Example 9: ownership ---");
    // A struct with a String field is NOT Copy: assigning it MOVES it
    // (same rules as 21_passing_values_to_function.rs).
    let carol = User::new("Carol", "carol@example.com", 40);
    let carol_moved = carol;
    // println!("{}", carol.name);  // error[E0382]: borrow of moved value: `carol`
    println!("moved: {}", carol_moved.name);

    // You can move ONE field out; the rest of the struct is still readable.
    let dave = User::new("Dave", "dave@example.com", 35);
    let dave_email = dave.email; // moves the String out of dave.email
    println!("dave's email: {dave_email}, dave's age still readable: {}", dave.age);
    // println!("{}", dave.email);  // error: value moved

    // A `self` method consumes the struct.
    let name_only = carol_moved.into_name();
    println!("into_name gave back: {name_only}");
    // carol_moved.greet();  // error[E0382]: carol_moved was moved by into_name()

    // ------------------------------------------------------------------
    // Example 10: Destructuring: unpack fields into variables
    // ------------------------------------------------------------------
    println!("\n--- Example 10: destructuring ---");
    // `..` = ignore the remaining fields. & so we borrow instead of moving out of alice.
    let User { name, age, .. } = &alice;
    println!("name = {name}, age = {age}");
    let Point(x, y) = p;
    println!("x = {x}, y = {y}");
    // Works in match too: match on field values.
    for pt in [Point(0, 0), Point(0, 7), Point(2, 3)] {
        match pt {
            Point(0, 0) => println!("{:?} is the origin", pt),
            Point(0, y) => println!("{:?} is on the y axis at {y}", pt),
            Point(x, y) => println!("{:?} is somewhere else ({x}, {y})", pt),
        }
    }

    // ------------------------------------------------------------------
    // Example 11: A Vec of structs
    // ------------------------------------------------------------------
    println!("\n--- Example 11: Vec of structs ---");
    let mut users = vec![
        User::new("Erin", "erin@example.com", 22),
        User::new("Frank", "frank@example.com", 51),
        User::new("Grace", "grace@example.com", 37),
    ];
    // &mut: change each user in place.
    for u in &mut users {
        u.birthday();
    }
    users[1].deactivate();
    // &: just read.
    for u in &users {
        println!("{:<6} age {:>2}  active: {}", u.name, u.age, u.active);
    }
    // sort_by_key: sort by one field (see 25_collections_vec.rs).
    users.sort_by_key(|u| u.age);
    let names: Vec<&str> = users.iter().map(|u| u.name.as_str()).collect();
    println!("youngest first: {:?}", names);
}
