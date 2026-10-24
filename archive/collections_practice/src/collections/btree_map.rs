// ============================================================================
// BTreeMap<K, V>  —  key -> value, always SORTED by key. O(log n) operations.
// ----------------------------------------------------------------------------
// Mental model: a B-tree, not a binary tree. Each node holds MANY keys (Rust
// uses ~11 per node) so one node fills a cache line or two. Fewer, fatter nodes
// = fewer pointer chases = much better cache behaviour than a classic BST,
// which is why real databases and filesystems use B-trees too.
//
// HashMap vs BTreeMap — the interview answer:
//   HashMap  : O(1) average, arbitrary order, needs K: Eq + Hash
//   BTreeMap : O(log n),     sorted order,    needs K: Ord
// Choose BTreeMap when you need ANY of: sorted iteration, range queries,
// first/last key, or deterministic output. Otherwise HashMap is faster.
//
// Killer feature: range(). That is the thing a HashMap simply cannot do.
// ============================================================================

use std::collections::BTreeMap;

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let mut m: BTreeMap<&str, i32> = BTreeMap::new();
    m.insert("delta", 4);
    m.insert("alpha", 1);
    m.insert("charlie", 3);
    m.insert("bravo", 2);

    // Inserted out of order — iteration comes back sorted, every single run.
    println!("sorted by key = {:?}", m);

    let from_pairs = BTreeMap::from([(3, "three"), (1, "one"), (2, "two")]);
    println!("from_pairs    = {:?}", from_pairs);

    // ------------------------------------- same core API as HashMap
    println!("get alpha  = {:?}", m.get("alpha"));
    println!("contains   = {}", m.contains_key("bravo"));
    println!("remove     = {:?}", m.remove("delta"));
    *m.entry("alpha").or_insert(0) += 100; // entry() works here too
    println!("after entry bump = {:?}", m);

    // ================ WHAT HashMap CANNOT DO ===============================

    // 1. ordered ends — O(log n), no scan
    println!("first = {:?}", m.first_key_value());
    println!("last  = {:?}", m.last_key_value());

    // 2. RANGE QUERIES — the real reason to pick BTreeMap
    let scores = BTreeMap::from([(10, "a"), (20, "b"), (30, "c"), (40, "d"), (50, "e")]);
    let mid: Vec<_> = scores.range(20..=40).collect(); // inclusive upper bound
    println!("range 20..=40 = {:?}", mid);
    let from30: Vec<_> = scores.range(30..).collect(); // open-ended
    println!("range 30..    = {:?}", from30);

    // "the entry just below X" — floor/predecessor lookup, O(log n)
    let floor = scores.range(..35).next_back();
    println!("greatest key < 35 = {:?}", floor);
    // "the entry at or above X" — ceiling/successor lookup
    let ceil = scores.range(35..).next();
    println!("smallest key >= 35 = {:?}", ceil);

    // 3. pop the smallest / largest entry — priority-queue-ish behaviour
    //    with the bonus that you can also index and delete arbitrary keys.
    let mut q = scores.clone();
    println!("pop_first = {:?}", q.pop_first());
    println!("pop_last  = {:?}", q.pop_last());
    println!("left = {:?}", q);

    // 4. split_off: everything >= key moves into a NEW map
    let mut s = BTreeMap::from([(1, "a"), (2, "b"), (3, "c"), (4, "d")]);
    let high = s.split_off(&3);
    println!("split_off(3) -> low {:?} | high {:?}", s, high);

    // ------------------------------------------------------------- iterating
    // Always ascending. rev() gives descending. No sorting step needed.
    for (k, v) in &m {
        println!("  {k} -> {v}");
    }
    let desc: Vec<_> = m.iter().rev().collect();
    println!("descending = {:?}", desc);
    println!("keys sorted = {:?}", m.keys().collect::<Vec<_>>());

    // ==================== USE CASE 1: time-series window ===================
    // Keys are timestamps; "give me events in [t1, t2)" is one range() call.
    let mut events: BTreeMap<u64, &str> = BTreeMap::new();
    events.insert(1_000, "deploy started");
    events.insert(1_120, "pods rolling");
    events.insert(1_450, "error spike");
    events.insert(1_900, "rollback");
    let window: Vec<_> = events.range(1_100..1_500).collect();
    println!("events in [1100,1500) = {:?}", window);

    // ==================== USE CASE 2: bucketed lookup ======================
    // Latency SLO tiers: find which bucket a value falls into with one
    // predecessor query instead of an if/else ladder.
    let tiers = BTreeMap::from([(0, "fast"), (100, "ok"), (500, "slow"), (2000, "critical")]);
    for latency in [42, 250, 1500, 9000] {
        let tier = tiers.range(..=latency).next_back().unwrap().1;
        println!("  {latency}ms -> {tier}");
    }

    // ==================== USE CASE 3: deterministic output =================
    // Config dumps, snapshot tests, anything diffed between runs: BTreeMap
    // prints identically every time. A HashMap would shuffle and create noise.
    let cfg = BTreeMap::from([("replicas", "3"), ("image", "api:v2"), ("cpu", "500m")]);
    for (k, v) in &cfg {
        println!("  {k}: {v}");
    }

    // ------------------------------------------------------- ordering caveat
    // Sorting follows Ord for the key type. For &str/String that's byte order,
    // so uppercase sorts before lowercase and "10" sorts before "9".
    let strings = BTreeMap::from([("Zebra", 1), ("apple", 2), ("10", 3), ("9", 4)]);
    println!("byte-order sort = {:?}", strings.keys().collect::<Vec<_>>());
    // If you need numeric order, use numeric keys; if you need case-insensitive
    // order, normalize the key or wrap it in a newtype with a custom Ord.
}
