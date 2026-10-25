// Traits: shared behavior
//
// A trait is a list of methods. A type that implements the trait promises to
// have those methods. (Similar to an interface in Java/Go/TypeScript.)
//
//   trait Animal { fn sound(&self); }     <- the promise: "has a sound() method"
//   impl Animal for Dog { ... }           <- Dog keeps the promise, its own way
//
// Why bother? Code can then work with "any Animal" without knowing which one:
//   fn make_noise(a: &impl Animal) { a.sound(); }   // takes a Dog, a Cat, ...
//   let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];
// (Box: 22_box_heap_allocation.rs. Generics, which pair with traits: 35_generics.rs.)
//
// You already use std traits: Display is what makes {} work in println!,
// Debug makes {:?} work, Clone gives .clone().

// The trait only lists the method signature, no body.
// &self = the method borrows the value it's called on (like `this` / `self`).
trait Animal {
    fn sound(&self);
}

// A struct with no fields (a "unit struct"). Dog has no data, only behavior.
struct Dog;

// Dog's version of sound().
impl Animal for Dog {
    fn sound(&self) {
        println!("Woof!");
    }
}

struct Cat;

// Same method name, different behavior.
impl Animal for Cat {
    fn sound(&self) {
        println!("Meow!");
    }
}

fn main() {
    // Unit structs are created by just writing the name.
    let dog = Dog;
    let cat = Cat;

    // method call syntax: dog.sound() is the same as Animal::sound(&dog)
    dog.sound();
    cat.sound();
}
