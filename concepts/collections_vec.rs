// Vec<T>: a growable list
//
// A Vec stores values of ONE type, in order, next to each other in memory (on the heap).
// It can grow and shrink at runtime. It's the collection you'll use most.
//
//   let mut v = Vec::new();  v.push(10);  v.push(20);  v.push(30);
//
//   stack (the Vec itself)        heap (the data)
//   ┌──────────────┐              ┌────┬────┬────┬────┐
//   │ ptr ─────────┼────────────> │ 10 │ 20 │ 30 │    │
//   │ len = 3      │              └────┴────┴────┴────┘
//   │ capacity = 4 │                index 0   1    2   (spare slot)
//   └──────────────┘
//
//   len      = how many items it holds now
//   capacity = how many fit before it must allocate a bigger block and move them
//
// Cheat sheet:
// | Want to                  | Use                         | Notes                         |
// |--------------------------|-----------------------------|-------------------------------|
// | add at the end           | v.push(x)                   | fast                          |
// | remove from the end      | v.pop()                     | returns Option<T>             |
// | read by index            | v[i]  or  v.get(i)          | [] panics if out of range,    |
// |                          |                             | get returns Option            |
// | change by index          | v[i] = x                    | needs `let mut v`             |
// | insert/remove in middle  | v.insert(i, x) / v.remove(i)| slow: shifts the items after  |
// | size                     | v.len(), v.is_empty()       |                               |
// | search                   | v.contains(&x)              | checks every item             |
// | sort                     | v.sort()                    | changes v in place            |

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Creating a Vec
    // ------------------------------------------------------------------
    println!("--- Example 1: creating ---");
    let empty: Vec<i32> = Vec::new(); // type needed: nothing inside to guess it from
    let numbers = vec![10, 20, 30]; // vec! macro: create with items; type inferred as Vec<i32>
    let zeros = vec![0; 5]; // [0, 0, 0, 0, 0]: value 0, repeated 5 times
    let with_room: Vec<i32> = Vec::with_capacity(100); // empty, but space for 100 already reserved
    println!("empty = {:?}, numbers = {:?}, zeros = {:?}", empty, numbers, zeros);
    println!("with_room: len {} capacity {}", with_room.len(), with_room.capacity());

    // ------------------------------------------------------------------
    // Example 2: Adding and removing
    // ------------------------------------------------------------------
    println!("\n--- Example 2: push, pop, insert, remove ---");
    // `mut` is required to change a Vec (add, remove, edit items).
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    println!("after push:   {:?} (len {}, capacity {})", v, v.len(), v.capacity());

    // pop() removes the LAST item. Returns Option: None if the Vec is empty.
    let last = v.pop();
    println!("pop() -> {:?}, v = {:?}", last, v);

    v.insert(0, 100); // put 100 at index 0; everything else shifts right
    println!("insert(0,100) {:?}", v);
    let removed = v.remove(1); // take out index 1; everything after shifts left
    println!("remove(1) -> {}, v = {:?}", removed, v);

    // ------------------------------------------------------------------
    // Example 3: Reading items: [] vs get()
    // ------------------------------------------------------------------
    println!("\n--- Example 3: [] vs get() ---");
    let v = vec![10, 20, 30];
    // v[i]: gives the item directly. PANICS if i is out of range.
    println!("v[0] = {}", v[0]);
    // v.get(i): returns Option<&T>, so a bad index is None instead of a crash.
    println!("v.get(1)  = {:?}", v.get(1)); // Some(20)
    println!("v.get(99) = {:?}", v.get(99)); // None
    // v[99] would panic: "index out of bounds: the len is 3 but the index is 99"
    println!("first = {:?}, last = {:?}", v.first(), v.last()); // also Option

    // ------------------------------------------------------------------
    // Example 4: Looping: three ways, three ownership rules
    // ------------------------------------------------------------------
    println!("\n--- Example 4: iterating ---");
    let mut v = vec![1, 2, 3];

    // &v: borrow each item to READ it. v is still usable after.
    for x in &v {
        print!("{} ", x);
    }
    println!("<- read with &v");

    // &mut v: borrow each item to CHANGE it. *x dereferences to the real item.
    for x in &mut v {
        *x *= 10;
    }
    println!("{:?} <- changed with &mut v", v);

    // With the index too: enumerate() gives (index, item) pairs.
    for (i, x) in v.iter().enumerate() {
        print!("[{}]={} ", i, x);
    }
    println!("<- enumerate");

    // v (no &): MOVES the Vec into the loop. Each x is owned. v is gone after.
    for x in v {
        print!("{} ", x);
    }
    println!("<- consumed v");
    // println!("{:?}", v); // error: borrow of moved value: `v`

    // ------------------------------------------------------------------
    // Example 5: Common operations
    // ------------------------------------------------------------------
    println!("\n--- Example 5: search, sort, transform ---");
    let mut v = vec![5, 3, 8, 1, 9, 2];
    println!("contains 8? {}", v.contains(&8)); // & because contains borrows the value
    println!("position of 8: {:?}", v.iter().position(|&x| x == 8)); // Option<usize>
    println!("max: {:?}  sum: {}", v.iter().max(), v.iter().sum::<i32>());

    v.sort(); // ascending, in place
    println!("sorted:   {:?}", v);
    v.reverse();
    println!("reversed: {:?}", v);

    // iter().map(...).collect(): build a NEW Vec from an old one (v is unchanged).
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    // filter keeps only items where the closure returns true.
    let evens: Vec<i32> = v.iter().filter(|&&x| x % 2 == 0).cloned().collect();
    println!("doubled: {:?}  evens: {:?}", doubled, evens);

    // ------------------------------------------------------------------
    // Example 6: Slices: borrowing part of a Vec
    // ------------------------------------------------------------------
    println!("\n--- Example 6: slices ---");
    let v = vec![10, 20, 30, 40, 50];
    // &v[start..end]: a view of items start..end-1. No copy is made.
    let middle: &[i32] = &v[1..4]; // [20, 30, 40]
    println!("v[1..4] = {:?}", middle);
    println!("v[..2]  = {:?}", &v[..2]); // from the start
    println!("v[3..]  = {:?}", &v[3..]); // to the end
    // Functions usually take &[T] instead of &Vec<T>: it accepts a whole Vec,
    // part of one, or an array.
    println!("sum of all = {}, sum of middle = {}", total(&v), total(middle));

    // ------------------------------------------------------------------
    // Example 7: Vec of Strings (non-Copy items) and ownership
    // ------------------------------------------------------------------
    println!("\n--- Example 7: Vec<String> ---");
    let mut names = vec!["Alice".to_string(), "Bob".to_string()];
    names.push(String::from("Carol"));

    // Borrow an item: fine, names still owns it.
    let first: &String = &names[0];
    println!("first = {}", first);

    // let taken = names[0]; // error: cannot move out of index of `Vec<String>`
    // A String can't be copied out by index; take it with remove(), or clone() it.
    let copy = names[0].clone(); // a separate String; names keeps the original
    let taken = names.remove(1); // moves "Bob" out of the Vec
    println!("copy = {}, taken = {}, names = {:?}", copy, taken, names);
}

// Takes a slice (&[i32]), so it works with &Vec<i32>, &v[1..4], or &[1, 2, 3].
fn total(items: &[i32]) -> i32 {
    items.iter().sum()
}
