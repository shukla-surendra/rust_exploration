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

struct Container<T> {
    value: T,
}

fn main() {
    let a = Container { value: 100 };
    let b = Container { value: "Hello" };
}