// HashSet<T>: a collection of UNIQUE values
//
// A HashSet is like a HashMap with keys only (no values). It answers one question
// fast: "is this value in the set?"
//
// - Each value appears at most once: inserting a duplicate does nothing.
// - NO order (like HashMap). Need sorted? Use BTreeSet, see 28_collections_others.rs.
// - Needs `use std::collections::HashSet;`
//
// Use it for: removing duplicates, "have I seen this before?", comparing groups
// (what's in both, what's in one but not the other).
//
// Set operations (a = {1, 2, 3}, b = {2, 3, 4}):
// | Operation            | Method                    | Result    | Meaning           |
// |----------------------|---------------------------|-----------|-------------------|
// | union                | a.union(&b)               | {1,2,3,4} | in either         |
// | intersection         | a.intersection(&b)        | {2,3}     | in both           |
// | difference           | a.difference(&b)          | {1}       | in a, not in b    |
// | symmetric difference | a.symmetric_difference(&b)| {1,4}     | in exactly one    |

use std::collections::HashSet;

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Inserting: duplicates are ignored
    // ------------------------------------------------------------------
    println!("--- Example 1: insert ---");
    let mut fruits = HashSet::new();
    // insert returns bool: true if the value was new, false if it was already there.
    println!("insert apple  -> {}", fruits.insert("apple")); // true
    println!("insert banana -> {}", fruits.insert("banana")); // true
    println!("insert apple  -> {}", fruits.insert("apple")); // false: duplicate
    println!("fruits = {:?} (len {})", fruits, fruits.len()); // 2 items, any order

    // ------------------------------------------------------------------
    // Example 2: Checking and removing
    // ------------------------------------------------------------------
    println!("\n--- Example 2: contains, remove ---");
    println!("contains apple? {}", fruits.contains("apple")); // true
    println!("contains mango? {}", fruits.contains("mango")); // false
    // remove returns bool: true if it was there.
    println!("remove banana -> {}", fruits.remove("banana")); // true
    println!("fruits = {:?}", fruits);

    // ------------------------------------------------------------------
    // Example 3: Removing duplicates from a Vec
    // ------------------------------------------------------------------
    println!("\n--- Example 3: dedupe a Vec ---");
    let numbers = vec![3, 1, 3, 2, 1, 3];
    // collect() can build a HashSet from an iterator, just like a Vec.
    let unique: HashSet<i32> = numbers.iter().cloned().collect();
    println!("{:?} -> {:?}", numbers, unique);

    // Keep the ORIGINAL order: use the set only to remember what we've seen.
    let mut seen = HashSet::new();
    let mut in_order = Vec::new();
    for n in &numbers {
        // insert returns false for duplicates, so only first appearances are kept.
        if seen.insert(*n) {
            in_order.push(*n);
        }
    }
    println!("in original order: {:?}", in_order); // [3, 1, 2]

    // ------------------------------------------------------------------
    // Example 4: Set operations
    // ------------------------------------------------------------------
    println!("\n--- Example 4: union, intersection, difference ---");
    let a: HashSet<i32> = HashSet::from([1, 2, 3]);
    let b: HashSet<i32> = HashSet::from([2, 3, 4]);

    // These return iterators over references; collect them into a Vec and sort
    // so the printed order is stable.
    println!("union        = {:?}", sorted(a.union(&b)));
    println!("intersection = {:?}", sorted(a.intersection(&b)));
    println!("a - b        = {:?}", sorted(a.difference(&b)));
    println!("b - a        = {:?}", sorted(b.difference(&a)));
    println!("in only one  = {:?}", sorted(a.symmetric_difference(&b)));

    // Comparing whole sets.
    let small: HashSet<i32> = HashSet::from([2, 3]);
    println!("small is subset of a? {}", small.is_subset(&a)); // true
    println!("a and b share nothing? {}", a.is_disjoint(&b)); // false

    // ------------------------------------------------------------------
    // Example 5: Real-world: who is in both groups?
    // ------------------------------------------------------------------
    println!("\n--- Example 5: common members ---");
    let rust_club: HashSet<&str> = HashSet::from(["Alice", "Bob", "Carol"]);
    let go_club: HashSet<&str> = HashSet::from(["Bob", "Dave", "Carol"]);
    println!("in both clubs:   {:?}", sorted(rust_club.intersection(&go_club)));
    println!("only Rust club:  {:?}", sorted(rust_club.difference(&go_club)));
}

// Helper: turn an iterator of &T into a sorted Vec<T> so output order is stable.
// T: Ord + Copy means "T can be compared/sorted and copied", true for i32 and &str.
fn sorted<'a, T: Ord + Copy + 'a>(items: impl Iterator<Item = &'a T>) -> Vec<T> {
    let mut v: Vec<T> = items.copied().collect();
    v.sort();
    v
}
