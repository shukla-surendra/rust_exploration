// Arrays vs Vec vs slices
//
// Yes, Rust has arrays. Rust has three "list of items" types, and they're
// different things:
//
// | Type      | Name              | Size                      | Lives on       | Grows? |
// |-----------|-------------------|---------------------------|----------------|--------|
// | [T; N]    | array             | FIXED, known at compile   | stack (inline) | No     |
// |           |                   | time: N is part of type   |                |        |
// | Vec<T>    | vector            | DYNAMIC, changes at       | heap (data),   | Yes    |
// |           | (dynamic array)   | runtime                   | stack (ptr,len,|        |
// |           |                   |                           | capacity)      |        |
// | &[T]      | slice             | a VIEW of part or all of  | wherever the   | No     |
// |           |                   | an array or Vec           | data already is|        |
//
// So:
// - Array = fixed size. [i32; 3] and [i32; 4] are DIFFERENT types.
// - Vec   = Rust's dynamic array (like C++ std::vector, Java ArrayList, Python list).
//           It is not fixed size: push() grows it.
// - Slice = "some items in a row, somewhere", borrowed. Functions usually take
//           &[T] so they work with arrays AND Vecs.
//
// Memory picture:
//
//   let a = [1, 2, 3];               let v = vec![1, 2, 3];
//
//   stack                            stack                 heap
//   ┌───┬───┬───┐                    ┌──────────┐          ┌───┬───┬───┐
//   │ 1 │ 2 │ 3 │  the items         │ ptr ─────┼────────> │ 1 │ 2 │ 3 │
//   └───┴───┴───┘  themselves        │ len = 3  │          └───┴───┴───┘
//                                    │ cap = 3  │
//                                    └──────────┘
//
// When to use which:
// - Size never changes and is small (RGB color, 3D point, days of week) -> array
// - Size changes, comes from input, or is large                          -> Vec
// - Function that only reads a list                                      -> take &[T]

use std::mem::size_of;

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Arrays: fixed size, size is part of the type
    // ------------------------------------------------------------------
    println!("--- Example 1: arrays ---");
    let days: [&str; 3] = ["Mon", "Tue", "Wed"]; // type: 3 items of &str
    let zeros = [0; 5]; // [0, 0, 0, 0, 0]: value 0 repeated 5 times -> [i32; 5]
    println!("days = {:?}, len = {}", days, days.len());
    println!("zeros = {:?}", zeros);

    // Items can change (with mut), but the NUMBER of items cannot.
    let mut point = [1, 2, 3];
    point[0] = 100;
    println!("point = {:?}", point);
    // point.push(4); // error: no method named `push` found for array `[i32; 3]`

    // N is part of the type: these two are different types.
    let _three: [i32; 3] = [1, 2, 3];
    // let _wrong: [i32; 3] = [1, 2, 3, 4]; // error[E0308]: mismatched types
    //                                      // (expected `[i32; 3]`, found `[i32; 4]`)

    // ------------------------------------------------------------------
    // Example 2: Vec: Rust's dynamic array
    // ------------------------------------------------------------------
    println!("\n--- Example 2: Vec (dynamic array) ---");
    let mut v = vec![1, 2, 3]; // type: Vec<i32>, no size in the type
    v.push(4); // grows at runtime
    v.push(5);
    println!("v = {:?}, len = {}", v, v.len());
    v.pop();
    println!("after pop: {:?}", v);
    // More Vec operations in 25_collections_vec.rs.

    // ------------------------------------------------------------------
    // Example 3: How a Vec grows: len vs capacity
    // ------------------------------------------------------------------
    println!("\n--- Example 3: Vec growth ---");
    // When len reaches capacity, the Vec allocates a BIGGER block on the heap
    // (usually double), copies the items over, and frees the old block.
    let mut grow: Vec<i32> = Vec::new();
    let mut last_cap = grow.capacity();
    println!("start: len 0, capacity {}", last_cap);
    for i in 1..=20 {
        grow.push(i);
        if grow.capacity() != last_cap {
            last_cap = grow.capacity();
            println!("  len {:2} -> capacity grew to {}", grow.len(), last_cap);
        }
    }
    // Know the size in advance? Reserve it once and avoid the regrowing.
    let reserved: Vec<i32> = Vec::with_capacity(20);
    println!("with_capacity(20): len {}, capacity {}", reserved.len(), reserved.capacity());

    // ------------------------------------------------------------------
    // Example 4: Where the data lives (size_of)
    // ------------------------------------------------------------------
    println!("\n--- Example 4: size on the stack ---");
    // size_of::<T>() = bytes the value itself takes (not counting heap data).
    // An array holds its items INLINE, so it grows with N.
    println!("[i32; 3]    = {} bytes", size_of::<[i32; 3]>()); // 3 × 4 = 12
    println!("[i32; 1000] = {} bytes", size_of::<[i32; 1000]>()); // 4000: all on the stack
    // A Vec is always ptr + len + capacity (3 × 8 bytes on 64-bit), whatever it holds.
    println!("Vec<i32>    = {} bytes (items are on the heap)", size_of::<Vec<i32>>()); // 24
    // A slice reference is ptr + len.
    println!("&[i32]      = {} bytes (ptr + len)", size_of::<&[i32]>()); // 16
    // That's why huge arrays are a bad idea: the stack is small (8 MB for main on
    // macOS/Linux by default). [0u8; 100_000_000] would overflow it; use a Vec.

    // ------------------------------------------------------------------
    // Example 5: Copy vs move (see 21_passing_values_to_function.rs)
    // ------------------------------------------------------------------
    println!("\n--- Example 5: copy vs move ---");
    // An array of Copy items is itself Copy: assigning makes a full copy.
    let a = [1, 2, 3];
    let mut b = a; // copy
    b[0] = 99;
    println!("a = {:?}, b = {:?}  <- a unchanged, a is still usable", a, b);

    // A Vec is never Copy: assigning MOVES ownership of the heap data.
    let v1 = vec![1, 2, 3];
    let v2 = v1; // move
    // println!("{:?}", v1); // error: borrow of moved value: `v1`
    let mut v3 = v2.clone(); // explicit copy of the heap data
    v3[0] = 99;
    println!("v2 = {:?}, v3 = {:?}", v2, v3);

    // ------------------------------------------------------------------
    // Example 6: Out of bounds
    // ------------------------------------------------------------------
    println!("\n--- Example 6: out of bounds ---");
    let arr = [10, 20, 30];
    // arr[10] with a constant index: COMPILE error, the compiler knows len is 3:
    //   "error: this operation will panic at runtime"
    // With a runtime index it's a PANIC, same as a Vec. get() returns Option.
    let i = arr.len() + 7;
    println!("arr.get({}) = {:?}", i, arr.get(i)); // None, no crash
    // arr[i] would panic: "index out of bounds: the len is 3 but the index is 10"

    // ------------------------------------------------------------------
    // Example 7: Slices: one function for arrays AND Vecs
    // ------------------------------------------------------------------
    println!("\n--- Example 7: slices ---");
    let arr = [1, 2, 3, 4, 5];
    let vec = vec![10, 20, 30];
    // &arr and &vec both turn into &[i32] automatically.
    println!("sum(&arr)       = {}", sum(&arr));
    println!("sum(&vec)       = {}", sum(&vec));
    println!("sum(&arr[1..3]) = {}  <- part of an array", sum(&arr[1..3])); // 2 + 3

    // ------------------------------------------------------------------
    // Example 8: Converting between them
    // ------------------------------------------------------------------
    println!("\n--- Example 8: converting ---");
    let arr = [1, 2, 3];
    let v: Vec<i32> = arr.to_vec(); // array -> Vec (copies the items to the heap)
    let v_also = Vec::from(arr); // same thing
    println!("array -> Vec: {:?} {:?}", v, v_also);

    // Vec -> array: can FAIL (the Vec might have the wrong length), so it returns
    // Result (see 32_option_result.rs).
    let ok: Result<[i32; 3], _> = v.clone().try_into();
    let bad: Result<[i32; 2], _> = v.try_into();
    println!("Vec(len 3) -> [i32; 3]: {:?}", ok); // Ok([1, 2, 3])
    println!("Vec(len 3) -> [i32; 2]: {:?}", bad.is_err()); // true: wrong size

    // ------------------------------------------------------------------
    // Example 9: 2D arrays and 2D Vecs
    // ------------------------------------------------------------------
    println!("\n--- Example 9: 2D ---");
    // Fixed grid: an array of arrays. [[i32; 3]; 2] = 2 rows of 3.
    let grid = [[1, 2, 3], [4, 5, 6]];
    println!("grid[1][2] = {}", grid[1][2]); // 6: row 1, column 2
    // Grid whose size is known only at runtime: a Vec of Vecs.
    let (rows, cols) = (2, 4);
    let mut board = vec![vec!['.'; cols]; rows];
    board[0][1] = 'X';
    for row in &board {
        println!("{}", row.iter().collect::<String>());
    }
}

// Takes a slice, so it accepts &[i32; N] arrays, &Vec<i32>, and parts of either.
fn sum(items: &[i32]) -> i32 {
    items.iter().sum()
}
