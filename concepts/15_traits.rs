trait Animal {
    fn sound(&self);
}

struct Dog;

impl Animal for Dog {
    fn sound(&self) {
        println!("Woof!");
    }
}

struct Cat;

impl Animal for Cat {
    fn sound(&self) {
        println!("Meow!");
    }
}

fn main() {
    let dog = Dog;
    let cat = Cat;

    dog.sound();
    cat.sound();
}