// ============================================================================
// ARRAY  [T; N]   and   SLICE  &[T]
// ----------------------------------------------------------------------------
// Mental model: an array is N values laid out back-to-back, and N is baked into
// the *type*. `[i32; 3]` and `[i32; 4]` are two different types, as different
// as i32 and String. That is why an array lives on the stack: the compiler
// knows its exact size at compile time.
//
// A slice `&[T]` is the "borrowed view" of a contiguous run of T. It is a fat
// pointer: (start_address, length). It does NOT own the data. Arrays, Vecs and
// Strings all hand out slices — that's why one function taking `&[i32]` works
// for all of them. Write your helpers against slices, not arrays/Vecs.
//
// Interview angle: index -> O(1). Search on unsorted -> O(n). Search on sorted
// -> O(log n) via binary_search. No insert/remove: the length is fixed forever.
// ============================================================================

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let a = [1, 2, 3, 4, 5]; // type inferred as [i32; 5]
    let zeros = [0u8; 4]; // repeat syntax: 4 copies of 0  -> [0,0,0,0]
    let explicit: [f64; 3] = [1.5, 2.5, 3.5]; // spelled-out type

    println!("a       = {:?}", a);
    println!("zeros   = {:?}", zeros);
    println!("explicit= {:?}", explicit);
    println!("len     = {} (known at compile time)", a.len());

    // ------------------------------------------------------------- accessing
    println!("a[0]    = {}", a[0]); // panics at runtime if out of bounds
    println!("get(0)  = {:?}", a.get(0)); // Some(&1)  -- the safe way
    println!("get(99) = {:?}", a.get(99)); // None      -- no panic

    // Rule of thumb: `a[i]` when the index is provably valid, `.get(i)` when it
    // comes from user input / arithmetic. Bounds checks are what make Rust
    // memory-safe here; the compiler elides most of them in hot loops.

    // ------------------------------------------------------------- mutating
    let mut m = [10, 20, 30];
    m[1] = 99; // needs `mut` — arrays are values, not references
    println!("mutated = {:?}", m);

    // -------------------------------------------------------- the 3 iterators
    // .iter()       -> &T        borrow each element (read only)
    // .iter_mut()   -> &mut T    borrow each element mutably (edit in place)
    // .into_iter()  -> T         consume/copy out each element by value
    let mut nums = [1, 2, 3];

    let sum: i32 = nums.iter().sum(); // &i32 items, sum() handles the deref
    println!("sum     = {}", sum);

    for n in nums.iter_mut() {
        *n *= 10; // `*n` because n is &mut i32
    }
    println!("x10     = {:?}", nums);

    // For arrays of Copy types, `for n in nums` copies; nums stays usable.
    // For arrays of non-Copy types (e.g. [String; 2]) it MOVES the array.
    let doubled: Vec<i32> = nums.into_iter().map(|n| n * 2).collect();
    println!("doubled = {:?}", doubled);

    // ---------------------------------------------------------------- slices
    let full: &[i32] = &a; // whole array as a slice
    let middle = &a[1..4]; // half-open range: indices 1,2,3  -> [2,3,4]
    let head = &a[..2]; // [1,2]
    let tail = &a[3..]; // [4,5]
    println!("middle  = {:?}, head = {:?}, tail = {:?}", middle, head, tail);
    println!("sum via helper: {}", sum_slice(full));
    println!("same helper on a Vec: {}", sum_slice(&vec![7, 8, 9]));

    // first/last return Option — empty slices are a real case, not an accident
    println!("first={:?} last={:?}", middle.first(), middle.last());

    // ------------------------------------------------------ search & sorting
    let mut s = [5, 3, 9, 1, 7];
    s.sort(); // in place, O(n log n), stable
    println!("sorted  = {:?}", s);

    // binary_search REQUIRES the slice to already be sorted.
    // Ok(i)  -> found at index i
    // Err(i) -> not found; i is where it *would* be inserted (very useful!)
    println!("find 7  -> {:?}", s.binary_search(&7));
    println!("find 4  -> {:?}", s.binary_search(&4));

    // Linear scans, no sorting needed:
    println!("contains 9      : {}", s.contains(&9));
    println!("position of 9   : {:?}", s.iter().position(|&x| x == 9));
    println!("max / min       : {:?} / {:?}", s.iter().max(), s.iter().min());

    // sort_by_key / sort_by for custom orders. Reverse sort:
    s.sort_by(|x, y| y.cmp(x));
    println!("desc    = {:?}", s);

    // ------------------------------------------- windows & chunks (DSA gold)
    let w = [1, 2, 3, 4, 5];
    // windows(k): every OVERLAPPING run of k -> sliding-window problems
    let sums: Vec<i32> = w.windows(3).map(|win| win.iter().sum()).collect();
    println!("windows(3) sums = {:?}", sums); // [6, 9, 12]

    // chunks(k): NON-overlapping blocks; last chunk may be shorter
    let blocks: Vec<&[i32]> = w.chunks(2).collect();
    println!("chunks(2)       = {:?}", blocks); // [[1,2],[3,4],[5]]

    // ----------------------------------------------- two-pointer style moves
    let mut tp = [1, 2, 3, 4, 5];
    tp.swap(0, 4); // swap two indices without fighting the borrow checker
    println!("swapped = {:?}", tp);
    tp.reverse();
    println!("reversed= {:?}", tp);

    // split_at_mut hands you TWO mutable halves at once. Plain indexing can't
    // do this (two &mut into one array), which is exactly the merge-sort need.
    let (left, right) = tp.split_at_mut(2);
    left[0] = 100;
    right[0] = 200;
    println!("after split_at_mut = {:?}", tp);

    // ------------------------------------------------------------ 2-D arrays
    // An array of arrays. Rows are contiguous, so iterate row-major for cache.
    let mut grid = [[0i32; 4]; 3]; // 3 rows x 4 cols, all zero
    for r in 0..grid.len() {
        for c in 0..grid[0].len() {
            grid[r][c] = (r * 4 + c) as i32;
        }
    }
    for row in &grid {
        println!("{:?}", row);
    }

    // Gotcha: `[[0; 4]; 3]` copies the inner array — fine for Copy types.
    // For non-Copy (e.g. Vec/String) it won't compile; use vec![vec![]; n].

    // --------------------------------------------------------- array <-> Vec
    let v: Vec<i32> = a.to_vec(); // heap copy
    let back: [i32; 5] = v.clone().try_into().unwrap(); // length must match
    println!("to_vec  = {:?}, back to array = {:?}", v, back);

    // ------------------------------------------------------ copy semantics
    // [i32; 3] is Copy, so this is a copy, not a move: `orig` is still alive.
    let orig = [1, 2, 3];
    let copy = orig;
    println!("orig={:?} copy={:?} (both usable: i32 is Copy)", orig, copy);
}

// Take `&[T]`, never `&[T; N]` or `&Vec<T>` — this one signature accepts
// arrays, Vecs, and sub-slices of either. This is the idiomatic Rust habit.
fn sum_slice(xs: &[i32]) -> i32 {
    xs.iter().sum()
}
