// ============================================================================
// BTreeSet<T>  —  unique values, always SORTED, O(log n) operations.
// ----------------------------------------------------------------------------
// Mental model: it IS a BTreeMap<T, ()>, exactly like HashSet is a
// HashMap<T, ()>. So it inherits everything from btree_map.rs: sorted
// iteration, range queries, first/last, O(log n) instead of O(1).
//
// HashSet vs BTreeSet:
//   HashSet  : O(1) average, arbitrary order, T: Eq + Hash
//   BTreeSet : O(log n),     sorted order,    T: Ord
// Same trade as HashMap vs BTreeMap. Default to HashSet; switch the moment you
// need order, ranges, or min/max.
// ============================================================================

use std::collections::BTreeSet;

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let mut s: BTreeSet<i32> = BTreeSet::new();
    for n in [5, 1, 9, 1, 3] {
        s.insert(n); // insert -> bool, same as HashSet
    }
    println!("set = {:?}  (sorted AND deduped, every run)", s);

    let from_vec: BTreeSet<&str> = vec!["pear", "apple", "fig", "apple"]
        .into_iter()
        .collect();
    println!("words = {:?}", from_vec);

    // "sort + dedup a Vec" in one expression:
    let cleaned: Vec<i32> = vec![4, 2, 7, 2, 4].into_iter().collect::<BTreeSet<_>>().into_iter().collect();
    println!("sorted+deduped Vec = {:?}", cleaned);

    // ------------------------------------------------------ core operations
    println!("contains 3 = {}", s.contains(&3));
    println!("remove 9   = {}", s.remove(&9));
    println!("len        = {}", s.len());

    // ================ WHAT HashSet CANNOT DO ===============================

    // 1. min / max in O(log n) — no full scan
    println!("first = {:?}, last = {:?}", s.first(), s.last());

    // 2. range queries
    let nums: BTreeSet<i32> = (1..=20).filter(|n| n % 3 == 0).collect();
    println!("multiples of 3 = {:?}", nums);
    println!("range 6..=15   = {:?}", nums.range(6..=15).collect::<Vec<_>>());

    // predecessor / successor — "closest value below/above X"
    println!("largest  < 10 = {:?}", nums.range(..10).next_back());
    println!("smallest >= 10 = {:?}", nums.range(10..).next());

    // 3. pop from either end — a sorted worklist
    let mut q = nums.clone();
    println!("pop_first = {:?}", q.pop_first());
    println!("pop_last  = {:?}", q.pop_last());

    // 4. ordered iteration, forwards and backwards, no sort step
    println!("ascending  = {:?}", s.iter().collect::<Vec<_>>());
    println!("descending = {:?}", s.iter().rev().collect::<Vec<_>>());

    // 5. split_off at a value
    let mut sp: BTreeSet<i32> = (1..=6).collect();
    let high = sp.split_off(&4);
    println!("split_off(4) -> low {:?} | high {:?}", sp, high);

    // -------------------------------------------------- set algebra (sorted)
    // Same operations as HashSet, but the results come out in order.
    let a: BTreeSet<i32> = [1, 2, 3, 4].into_iter().collect();
    let b: BTreeSet<i32> = [3, 4, 5, 6].into_iter().collect();
    println!("union        = {:?}", a.union(&b).collect::<Vec<_>>());
    println!("intersection = {:?}", a.intersection(&b).collect::<Vec<_>>());
    println!("difference   = {:?}", a.difference(&b).collect::<Vec<_>>());
    println!("symmetric    = {:?}", a.symmetric_difference(&b).collect::<Vec<_>>());
    println!("a ⊆ union    = {}", a.is_subset(&(&a | &b)));

    // ==================== USE CASE 1: free-slot allocator ==================
    // Hand out the lowest available id, give ids back on release. pop_first is
    // O(log n) and always returns the smallest — a Vec would need a scan.
    let mut free: BTreeSet<u32> = (1..=5).collect();
    let id1 = free.pop_first().unwrap();
    let id2 = free.pop_first().unwrap();
    println!("allocated {id1} and {id2}, free now {:?}", free);
    free.insert(id1); // release: goes back into sorted position automatically
    println!("after releasing {id1}: {:?}", free);

    // ==================== USE CASE 2: interval / booking check =============
    // "Is anything booked between t1 and t2?" is one range().next() call.
    let booked: BTreeSet<u32> = [900, 1030, 1400, 1615].into_iter().collect();
    for (start, end) in [(1000u32, 1100u32), (1100, 1300)] {
        let clash = booked.range(start..end).next();
        match clash {
            Some(t) => println!("  {start}-{end}: CLASH at {t}"),
            None => println!("  {start}-{end}: free"),
        }
    }

    // ==================== USE CASE 3: ordered leaderboard ==================
    // Reverse ordering by storing std::cmp::Reverse, so iteration is descending
    // while still being a set.
    use std::cmp::Reverse;
    let top: BTreeSet<Reverse<u32>> = [42, 17, 99, 63].into_iter().map(Reverse).collect();
    let ranked: Vec<u32> = top.iter().map(|Reverse(v)| *v).collect();
    println!("descending scores = {:?}", ranked);

    // ------------------------------------------------------------ trade-off
    // Cost of the ordering: every op is O(log n) with comparisons, versus one
    // hash. For pure "have I seen this" on hot paths, HashSet wins. Pick
    // BTreeSet when order/range is part of the problem, not as a default.
}
