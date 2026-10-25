//Generics = One piece of code, multiple data types.
/*
Common conventions:
- T → Type
- K → Key
- V → Value
- E → Error
You can also use descriptive names like Item or MyType.
Remember: The name is just a placeholder for a type. Using T is the most common convention, but it isn't mandatory.

1. Simple generic
Vec<T>

Means a vector containing values of type T.
For example:
Vec<i32>

A vector of integers.
2. One level of nesting
Option<Vec<i32>>

Read it from the inside out:
1. i32 → Integer.
2. Vec<i32> → A vector of integers.
3. Option<Vec<i32>> → An optional vector of integers.
Meaning: You might have a vector of integers, or you might have nothing (None).
3. Two levels of nesting
Option<Vec<Option<i32>>>

Read it from the inside out:
1. i32 → Integer.
2. Option<i32> → An optional integer.
3. Vec<Option<i32>> → A vector containing optional integers.
4. Option<Vec<Option<i32>>> → An optional vector containing optional integers.
For example, the vector could contain [Some(10), None, Some(30)], or the entire vector could be None.


*/

// A generic struct: <T> after the name says "T is a type I'll pick later".
// `value: T` means value has whatever type T turns out to be.
struct Container<T> {
    value: T,
}

// The same idea works for functions. To DO something with T you must say what T
// can do, using a trait bound (see 15_traits.rs):
//   fn show<T: std::fmt::Display>(item: T) { println!("{}", item); }
//   (T can be any type that can be printed with {})
//
// No runtime cost: the compiler writes a separate copy of the code for each
// type actually used (Container<i32>, Container<&str>, ...).

fn main() {
    // T is worked out from the value: here T = i32, so a is a Container<i32>.
    let a = Container { value: 100 };
    // Same struct, different type: T = &str, so b is a Container<&str>.
    let b = Container { value: "Hello" };
    // a and b are never used, so the compiler warns "unused variable".
    // Try:  println!("{} {}", a.value, b.value);
}