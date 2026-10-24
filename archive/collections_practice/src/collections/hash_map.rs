// ============================================================================
// HashMap<K, V>  —  key -> value, average O(1) lookup, NO ordering.
// ----------------------------------------------------------------------------
// Mental model: hash(key) picks a bucket; the entry lives there. Average O(1)
// for get/insert/remove, worst case O(n) if every key collides. Rust's default
// hasher (SipHash) is randomly seeded PER PROGRAM RUN — that's deliberate DoS
// protection against attacker-chosen colliding keys, and it's also why
// iteration order changes between runs. Never rely on the order.
//
// K must implement Eq + Hash. Two keys that are equal MUST hash the same —
// that's the contract you promise when you `#[derive(Hash, Eq, PartialEq)]`.
//
// Interview angle: "use a hashmap for O(1) lookup" is the single most common
// optimization from O(n^2) to O(n). Know the entry() API cold.
// ============================================================================

use std::collections::HashMap;

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("ana".to_string(), 90);
    scores.insert("bo".to_string(), 75);

    // insert returns the OLD value if the key was already present
    let old = scores.insert("ana".to_string(), 95);
    println!("insert over 'ana' returned {:?} (the previous value)", old);

    // build straight from pairs
    let from_pairs: HashMap<&str, i32> = HashMap::from([("x", 1), ("y", 2)]);
    println!("from_pairs = {:?}", from_pairs);

    // build from an iterator of tuples
    let squares: HashMap<i32, i32> = (1..=5).map(|n| (n, n * n)).collect();
    println!("squares = {:?} (order is random!)", squares);

    // ------------------------------------------------------------- reading
    // get returns Option<&V> — a missing key is normal, not an error.
    println!("get ana    = {:?}", scores.get("ana"));
    println!("get zoe    = {:?}", scores.get("zoe"));
    println!("with default = {}", scores.get("zoe").copied().unwrap_or(0));
    println!("contains bo  = {}", scores.contains_key("bo"));

    // Note you can `get("ana")` (a &str) on a HashMap<String, _> — Borrow lets
    // you look up with the borrowed form, no allocation needed.

    if let Some(v) = scores.get_mut("bo") {
        *v += 5; // read-modify-write through a mutable borrow
    }
    println!("bo after +5 = {:?}", scores.get("bo"));

    // ------------------------------------------------------------ removing
    println!("remove ana -> {:?}", scores.remove("ana")); // Option<V>
    println!("remove ana -> {:?} (already gone)", scores.remove("ana"));

    // ==================== THE entry() API — learn this ======================
    // entry(k) means "give me the slot for k, occupied or vacant". It hashes
    // ONCE. The naive alternative (contains_key then get_mut then insert)
    // hashes two or three times and reads worse.

    // 1. or_insert: default value if absent, then hand back &mut V
    let mut counts: HashMap<char, i32> = HashMap::new();
    for ch in "hello world".chars().filter(|c| !c.is_whitespace()) {
        *counts.entry(ch).or_insert(0) += 1; // the frequency-count idiom
    }
    let mut freq: Vec<_> = counts.iter().collect();
    freq.sort(); // sort only so the printed output is deterministic
    println!("char frequency = {:?}", freq);

    // 2. or_insert_with: default is EXPENSIVE, so build it lazily
    let mut groups: HashMap<usize, Vec<&str>> = HashMap::new();
    for w in ["ant", "bee", "cow", "wasp", "moth"] {
        groups.entry(w.len()).or_insert_with(Vec::new).push(w);
    }
    println!("grouped by length = {:?}", groups);

    // 3. or_default: uses Default::default() — shortest form of the same idea
    let mut tally: HashMap<&str, i32> = HashMap::new();
    *tally.entry("hits").or_default() += 1;
    println!("tally = {:?}", tally);

    // 4. and_modify + or_insert: "bump if present, else seed"
    let mut visits: HashMap<&str, i32> = HashMap::new();
    for page in ["/home", "/docs", "/home"] {
        visits.entry(page).and_modify(|c| *c += 1).or_insert(1);
    }
    println!("visits = {:?}", visits);

    // ------------------------------------------------------------ iterating
    let m = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
    let mut pairs: Vec<_> = m.iter().collect(); // (&K, &V)
    pairs.sort(); // HashMap order is arbitrary — sort to print stably
    println!("pairs  = {:?}", pairs);

    let mut keys: Vec<_> = m.keys().collect();
    keys.sort();
    println!("keys   = {:?}", keys);
    println!("sum of values = {}", m.values().sum::<i32>());

    // values_mut edits every value in place
    let mut mm = m.clone();
    for v in mm.values_mut() {
        *v *= 100;
    }
    let mut scaled: Vec<_> = mm.iter().collect();
    scaled.sort();
    println!("scaled = {:?}", scaled);

    // Need ordered output? Either collect+sort (as above) or use a BTreeMap.
    // See btree_map.rs — that's the "I need order" version of this file.

    // ------------------------------------------------ filtering into a new map
    let high: HashMap<&str, i32> = m
        .iter()
        .filter(|&(_, &v)| v >= 2)
        .map(|(&k, &v)| (k, v)) // deref out of the references
        .collect();
    println!("values >= 2 -> {} entries", high.len());

    // retain edits in place instead of building a new map
    let mut r = m.clone();
    r.retain(|_, v| *v % 2 == 1);
    println!("retained odd values -> {} entries", r.len());

    // --------------------------------------------------- struct keys & values
    // Any Eq + Hash type works as a key. Derive it on your own structs.
    #[derive(Hash, Eq, PartialEq, Debug)]
    struct Coord {
        x: i32,
        y: i32,
    }
    let mut board: HashMap<Coord, char> = HashMap::new();
    board.insert(Coord { x: 0, y: 0 }, 'X');
    board.insert(Coord { x: 1, y: 1 }, 'O');
    println!("board(0,0) = {:?}", board.get(&Coord { x: 0, y: 0 }));

    // Tuples are Eq + Hash already — a sparse grid needs no custom struct.
    let mut sparse: HashMap<(i32, i32), i32> = HashMap::new();
    sparse.insert((10, 20), 1);
    println!("sparse(10,20) = {:?}", sparse.get(&(10, 20)));

    // ---------------------------------------------------- ownership gotcha
    // insert() MOVES both key and value in. Once inserted, the original name
    // is gone unless the type is Copy.
    let key = String::from("owned-key");
    let mut owner: HashMap<String, i32> = HashMap::new();
    owner.insert(key, 1); // `key` is moved here
    // println!("{}", key);                   // ❌ borrow of moved value
    println!("moved key stored: {:?}", owner.keys().collect::<Vec<_>>());

    // ==================== USE CASE: two-sum in O(n) ========================
    // The canonical "hashmap turns O(n^2) into O(n)" answer.
    println!("two_sum([2,7,11,15], 9) = {:?}", two_sum(&[2, 7, 11, 15], 9));

    // ==================== USE CASE: group anagrams =========================
    let words = ["eat", "tea", "tan", "ate", "nat", "bat"];
    let mut grouped = group_anagrams(&words);
    grouped.sort();
    println!("anagram groups = {:?}", grouped);
}

fn two_sum(nums: &[i32], target: i32) -> Option<(usize, usize)> {
    // seen: value -> index we saw it at
    let mut seen: HashMap<i32, usize> = HashMap::new();
    for (i, &n) in nums.iter().enumerate() {
        // ask for the complement BEFORE inserting n, so n can't match itself
        if let Some(&j) = seen.get(&(target - n)) {
            return Some((j, i));
        }
        seen.insert(n, i);
    }
    None
}

fn group_anagrams(words: &[&str]) -> Vec<Vec<String>> {
    // key = the word's sorted letters; anagrams collapse to the same key
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for w in words {
        let mut letters: Vec<char> = w.chars().collect();
        letters.sort();
        let key: String = letters.into_iter().collect();
        map.entry(key).or_default().push(w.to_string());
    }
    map.into_values().collect()
}
