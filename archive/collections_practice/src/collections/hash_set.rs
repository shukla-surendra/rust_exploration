// ============================================================================
// HashSet<T>  —  unique values, average O(1) membership test, NO ordering.
// ----------------------------------------------------------------------------
// Mental model: it IS a HashMap<T, ()>. Literally — that's how the standard
// library implements it. Everything you know about HashMap applies: T needs
// Eq + Hash, order is arbitrary and randomized per run, average O(1) ops.
//
// Two jobs it does better than anything else:
//   1. "have I seen this before?"  -> O(1) instead of a Vec's O(n) contains()
//   2. set algebra: union / intersection / difference
//
// Interview angle: any time you write `if !visited.contains(&x)` against a Vec
// inside a loop, you've built an O(n^2). A HashSet makes it O(n).
// ============================================================================

use std::collections::HashSet;

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let mut s: HashSet<i32> = HashSet::new();

    // insert returns bool: true = newly added, false = already present.
    // That return value IS the dedup check — no separate contains() needed.
    println!("insert 1 -> {}", s.insert(1)); // true
    println!("insert 2 -> {}", s.insert(2)); // true
    println!("insert 1 -> {} (already there)", s.insert(1)); // false
    println!("set = {:?} len = {}", s, s.len());

    let from_arr = HashSet::from([1, 2, 3]);
    let from_vec: HashSet<i32> = vec![3, 1, 2, 3, 1].into_iter().collect();
    println!("from_arr={:?} deduped-from-vec len={}", from_arr, from_vec.len());

    // ------------------------------------------------------- membership & rm
    println!("contains 2 = {}", s.contains(&2));
    println!("remove 2   = {}", s.remove(&2)); // bool: was it there?
    println!("remove 2   = {} (gone already)", s.remove(&2));

    // ==================== SET ALGEBRA ======================================
    // These return lazy iterators, so collect() (or sort) to look at them.
    let a: HashSet<i32> = [1, 2, 3, 4].into_iter().collect();
    let b: HashSet<i32> = [3, 4, 5, 6].into_iter().collect();

    println!("union             = {:?}", sorted(a.union(&b)));
    println!("intersection      = {:?}", sorted(a.intersection(&b)));
    println!("difference a-b    = {:?}", sorted(a.difference(&b)));
    println!("difference b-a    = {:?}", sorted(b.difference(&a)));
    println!("symmetric_diff    = {:?}", sorted(a.symmetric_difference(&b)));

    // Operator sugar exists too (allocates a new set):
    println!("a & b (intersect) = {:?}", sorted((&a & &b).iter()));
    println!("a | b (union)     = {:?}", sorted((&a | &b).iter()));

    // relationship predicates
    let small: HashSet<i32> = [1, 2].into_iter().collect();
    let far: HashSet<i32> = [90, 91].into_iter().collect();
    println!("small ⊆ a         = {}", small.is_subset(&a));
    println!("a ⊇ small         = {}", a.is_superset(&small));
    println!("a disjoint far    = {}", a.is_disjoint(&far));

    // ------------------------------------------------------------- iterating
    // Order is arbitrary AND changes between program runs. Sort before you
    // print or compare, or you'll write a flaky test.
    println!("iterated (sorted for stability) = {:?}", sorted(a.iter()));

    let mut r = a.clone();
    r.retain(|&x| x % 2 == 0); // filter in place
    println!("evens only = {:?}", sorted(r.iter()));

    // ==================== USE CASE 1: dedup, order not needed ==============
    let raw = vec!["b", "a", "b", "c", "a"];
    let unique: HashSet<&str> = raw.iter().copied().collect();
    println!("unique count = {} from {} raw", unique.len(), raw.len());

    // If you need dedup that PRESERVES first-seen order, a HashSet alone won't
    // do it — pair it with a Vec:
    println!("dedup keeping order = {:?}", dedup_preserving_order(&raw));

    // ==================== USE CASE 2: seen/visited guard ===================
    // The cycle-detection idiom. Without the set this is O(n^2) or infinite.
    println!("first repeat in [3,1,4,1,5] = {:?}", first_repeat(&[3, 1, 4, 1, 5]));

    // ==================== USE CASE 3: O(n) instead of O(n^2) ===============
    // Longest consecutive run. Naive = sort O(n log n) or nested scan O(n^2).
    // With a set: only start counting at a number whose predecessor is absent,
    // so each element is walked at most twice -> O(n).
    println!(
        "longest consecutive in [100,4,200,1,3,2] = {}",
        longest_consecutive(&[100, 4, 200, 1, 3, 2])
    );

    // ------------------------------------------------------- custom elements
    #[derive(Hash, Eq, PartialEq, Debug)]
    struct Tag(String);
    let mut tags: HashSet<Tag> = HashSet::new();
    tags.insert(Tag("prod".into()));
    tags.insert(Tag("prod".into())); // duplicate: dropped
    println!("tag set size = {}", tags.len());

    // Gotcha worth saying out loud: elements must not be mutated while in the
    // set. Change a field that feeds Hash and the element is now in the wrong
    // bucket — permanently unfindable. Rust prevents this by only handing out
    // &T from a set (there is no get_mut), which is the API enforcing the
    // invariant for you.

    // When to use BTreeSet instead: you need sorted iteration, ranges, or
    // min/max. See btree_set.rs.
}

// helper: sets iterate arbitrarily, so sort for readable, stable output
fn sorted<'a, I: Iterator<Item = &'a i32>>(it: I) -> Vec<i32> {
    let mut v: Vec<i32> = it.copied().collect();
    v.sort();
    v
}

fn dedup_preserving_order<'a>(items: &[&'a str]) -> Vec<&'a str> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for &item in items {
        if seen.insert(item) {
            // true only the first time we see it
            out.push(item);
        }
    }
    out
}

fn first_repeat(nums: &[i32]) -> Option<i32> {
    let mut seen = HashSet::new();
    for &n in nums {
        if !seen.insert(n) {
            return Some(n);
        }
    }
    None
}

fn longest_consecutive(nums: &[i32]) -> usize {
    let set: HashSet<i32> = nums.iter().copied().collect();
    let mut best = 0;
    for &n in &set {
        if set.contains(&(n - 1)) {
            continue; // not the start of a run — skip, it'll be counted later
        }
        let mut len = 1;
        while set.contains(&(n + len as i32)) {
            len += 1;
        }
        best = best.max(len);
    }
    best
}
