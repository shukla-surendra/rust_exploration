// ============================================================================
// Vec<T>  —  the growable array. Your default collection, 90% of the time.
// ----------------------------------------------------------------------------
// Mental model: a Vec is 3 words on the stack — (pointer, len, capacity) —
// pointing at a single heap block. `len` is how many you have, `capacity` is
// how many fit before a reallocation. push() past capacity allocates a bigger
// block (roughly 2x) and MEMCPYs everything over.
//
// That amortized doubling is why push is "O(1) amortized" not O(1): most
// pushes are free, occasional ones are O(n), averaging out to O(1).
//
// Complexity:  index O(1) | push/pop back O(1)* | insert/remove middle O(n)
//              contains O(n) | sort O(n log n)
// ============================================================================

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let mut v: Vec<i32> = Vec::new(); // empty, no allocation yet
    let literal = vec![1, 2, 3]; // the vec! macro
    let repeated = vec![0; 5]; // five zeros
    let with_cap: Vec<i32> = Vec::with_capacity(100); // pre-allocate

    println!("literal={:?} repeated={:?}", literal, repeated);
    println!(
        "with_capacity: len={} cap={} (cap reserved, len still 0)",
        with_cap.len(),
        with_cap.capacity()
    );

    // If you know the final size, with_capacity avoids every intermediate
    // realloc + memcpy. Cheap win in hot paths; interviewers notice it.

    // ------------------------------------------------- growth, watched live
    for i in 1..=5 {
        v.push(i);
        println!("push {} -> len={} cap={}", i, v.len(), v.capacity());
    }

    // ------------------------------------------------------- add & remove
    println!("pop      -> {:?} (Option: the Vec may be empty)", v.pop());
    v.insert(0, 100); // O(n): shifts everything right
    println!("insert(0,100) = {:?}", v);
    let removed = v.remove(0); // O(n): shifts everything left
    println!("remove(0) -> {} leaving {:?}", removed, v);

    // swap_remove: O(1) removal — moves the LAST element into the hole.
    // Destroys ordering. Perfect when order doesn't matter (free lists, pools).
    let mut sr = vec![1, 2, 3, 4, 5];
    let got = sr.swap_remove(1);
    println!("swap_remove(1) -> {} leaving {:?} (order broken)", got, sr);

    // ------------------------------------------------------------- accessing
    let data = vec![10, 20, 30];
    println!("data[1]  = {}", data[1]); // panics if out of range
    println!("get(1)   = {:?}", data.get(1)); // Some(&20)
    println!("get(9)   = {:?}", data.get(9)); // None
    println!("first={:?} last={:?}", data.first(), data.last());

    // ------------------------------------------------------------- iterating
    let mut nums = vec![1, 2, 3, 4, 5, 6];

    let total: i32 = nums.iter().sum();
    let evens: Vec<&i32> = nums.iter().filter(|&&x| x % 2 == 0).collect();
    println!("total={} evens={:?}", total, evens);

    for n in nums.iter_mut() {
        *n += 100;
    }
    println!("after iter_mut = {:?}", nums);

    // enumerate when you need the index too
    for (i, n) in nums.iter().enumerate().take(3) {
        println!("  index {} -> {}", i, n);
    }

    // ------------------------------------------------- the iterator pipeline
    // This is where Rust reads better than a manual loop. Nothing runs until
    // a consumer (collect/sum/count/any/...) pulls — iterators are lazy.
    let squares_of_odds: Vec<i32> = (1..=10)
        .filter(|n| n % 2 == 1) // keep odds
        .map(|n| n * n) // square them
        .collect(); // now it actually runs
    println!("odd squares      = {:?}", squares_of_odds);

    let words = vec!["apple", "fig", "banana"];
    let long: Vec<String> = words
        .iter()
        .filter(|w| w.len() > 3)
        .map(|w| w.to_uppercase())
        .collect();
    println!("long uppercased  = {:?}", long);

    println!("any > 100        : {}", nums.iter().any(|&n| n > 100));
    println!("all > 0          : {}", nums.iter().all(|&n| n > 0));
    println!("count of evens   : {}", nums.iter().filter(|n| *n % 2 == 0).count());
    println!("find first > 103 : {:?}", nums.iter().find(|&&n| n > 103));

    // fold = the general reducer (accumulator, element) -> accumulator
    let product: i64 = vec![1i64, 2, 3, 4].iter().fold(1, |acc, &x| acc * x);
    println!("fold product     = {}", product);

    // ---------------------------------------------------- sorting & dedup
    let mut s = vec![5, 1, 4, 1, 5, 9, 2, 6];
    s.sort(); // stable, O(n log n)
    println!("sorted   = {:?}", s);
    s.dedup(); // removes CONSECUTIVE duplicates -> sort first!
    println!("deduped  = {:?}", s);

    s.sort_by(|a, b| b.cmp(a)); // descending
    println!("desc     = {:?}", s);

    let mut people = vec![("ana", 32), ("bo", 25), ("cy", 41)];
    people.sort_by_key(|&(_, age)| age); // sort by a field
    println!("by age   = {:?}", people);

    // Floats have no total order (NaN), so .sort() won't compile on f64.
    let mut fs = vec![2.5, 1.0, 3.75];
    fs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("floats   = {:?}", fs);

    // --------------------------------------------------- retain / drain
    let mut r = vec![1, 2, 3, 4, 5, 6, 7, 8];
    r.retain(|&x| x % 2 == 0); // keep what matches, in place, O(n)
    println!("retain even = {:?}", r);

    let mut d = vec![1, 2, 3, 4, 5];
    let taken: Vec<i32> = d.drain(1..3).collect(); // remove a range, get it back
    println!("drained {:?}, remaining {:?}", taken, d);

    // ------------------------------------------------------ joining & slicing
    let mut a = vec![1, 2];
    let mut b = vec![3, 4];
    a.append(&mut b); // moves b's elements over; b is left EMPTY
    println!("append -> a={:?} b={:?}", a, b);
    a.extend(vec![5, 6]); // copy/move in from any IntoIterator
    a.extend_from_slice(&[7, 8]);
    println!("extended = {:?}", a);
    println!("as slice = {:?}", &a[2..5]);

    // ------------------------------------------------------ Vec of Strings
    // Non-Copy elements: indexing out a String would move it out of the Vec,
    // which Rust forbids (it would leave a hole). Borrow or clone instead.
    let names = vec![String::from("ana"), String::from("bo")];
    let first_ref: &String = &names[0]; // borrow  ✅
    let first_clone = names[0].clone(); // clone   ✅
    // let moved = names[0];             // ❌ cannot move out of index
    println!("ref={} clone={}", first_ref, first_clone);

    // ------------------------------------------------------------- 2-D Vec
    let rows = 3;
    let cols = 4;
    let mut grid = vec![vec![0; cols]; rows]; // Vec<Vec<i32>>
    grid[1][2] = 7;
    for row in &grid {
        println!("{:?}", row);
    }
    // Perf note: Vec<Vec<T>> is `rows` separate heap allocations, pointer-
    // chased. A flat Vec<T> of len rows*cols indexed as [r * cols + c] is one
    // allocation and cache friendly. Say that out loud in an interview.

    // -------------------------------------------------- ownership gotcha
    let owned = vec![1, 2, 3];
    let consumed: i32 = owned.iter().sum(); // borrows -> owned still alive
    println!("sum={} owned still usable={:?}", consumed, owned);
    let moved: Vec<i32> = owned.into_iter().map(|x| x + 1).collect(); // MOVES
    // println!("{:?}", owned);           // ❌ owned was consumed above
    println!("moved  = {:?}", moved);
}
