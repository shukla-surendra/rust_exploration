// HashMap<K, V>: look up a value by its key
//
// A HashMap stores KEY -> VALUE pairs. Give it a key, get the value back fast,
// no matter how many entries there are (it doesn't search one by one like a Vec).
//
//   "Alice" -> 30
//   "Bob"   -> 25
//
// - Keys are unique: inserting an existing key replaces its value.
// - All keys have one type, all values have one type: HashMap<&str, i32>.
// - NO order: iterating can give the entries in any order, different between runs.
//   (Need sorted keys? Use BTreeMap, see collections_others.rs.)
// - Not in scope by default: needs `use std::collections::HashMap;`
//
// Cheat sheet:
// | Want to                       | Use                                   | Returns           |
// |-------------------------------|---------------------------------------|-------------------|
// | add / replace                 | m.insert(k, v)                        | Option<old value> |
// | read                          | m.get(&k)                             | Option<&V>        |
// | change in place               | m.get_mut(&k)                         | Option<&mut V>    |
// | remove                        | m.remove(&k)                          | Option<V>         |
// | check                         | m.contains_key(&k)                    | bool              |
// | insert if missing, then edit  | m.entry(k).or_insert(default)         | &mut V            |

use std::collections::HashMap;

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Creating and inserting
    // ------------------------------------------------------------------
    println!("--- Example 1: insert ---");
    let mut ages = HashMap::new(); // types inferred from the first insert
    ages.insert("Alice", 30);
    ages.insert("Bob", 25);

    // insert returns Option: the OLD value if the key was already there.
    let old = ages.insert("Alice", 31); // replaces 30 with 31
    println!("insert Alice again -> old value {:?}", old); // Some(30)
    println!("ages = {:?}  (len {})", ages, ages.len()); // order may vary

    // Create from a list of (key, value) pairs.
    let prices: HashMap<&str, f64> = HashMap::from([("apple", 0.5), ("bread", 2.25)]);
    println!("prices = {:?}", prices);

    // ------------------------------------------------------------------
    // Example 2: Reading values: get() vs []
    // ------------------------------------------------------------------
    println!("\n--- Example 2: get ---");
    // get returns Option<&V>: the key might not exist. See option_result.rs.
    println!("get Alice = {:?}", ages.get("Alice")); // Some(31)
    println!("get Zoe   = {:?}", ages.get("Zoe")); // None

    match ages.get("Bob") {
        Some(age) => println!("Bob is {}", age),
        None => println!("no Bob"),
    }
    // A default when missing. copied() turns Option<&i32> into Option<i32>.
    println!("Zoe's age or 0: {}", ages.get("Zoe").copied().unwrap_or(0));
    // ages["Zoe"] would PANIC (key not found), like v[99] on a Vec.
    println!("contains Bob? {}", ages.contains_key("Bob"));

    // ------------------------------------------------------------------
    // Example 3: Changing and removing
    // ------------------------------------------------------------------
    println!("\n--- Example 3: get_mut, remove ---");
    // get_mut gives a mutable reference to the value inside the map.
    if let Some(age) = ages.get_mut("Bob") {
        *age += 1; // * to change the value it points to
    }
    println!("Bob after birthday: {:?}", ages.get("Bob")); // Some(26)

    // remove returns Option<V>: the value that was removed, if the key existed.
    println!("remove Bob -> {:?}", ages.remove("Bob")); // Some(26)
    println!("remove Bob -> {:?}", ages.remove("Bob")); // None: already gone

    // ------------------------------------------------------------------
    // Example 4: The entry API: "insert if missing, then update"
    // ------------------------------------------------------------------
    println!("\n--- Example 4: entry (word count) ---");
    // Classic use: counting. Without entry you'd need get, check None, insert...
    let text = "the cat and the dog and the bird";
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for word in text.split_whitespace() {
        // entry(word): look up the key.
        // or_insert(0): if missing, insert 0. Either way, returns &mut to the value.
        let count = counts.entry(word).or_insert(0);
        *count += 1;
    }
    println!("counts = {:?}", counts);

    // Grouping: a map whose values are Vecs.
    let mut by_letter: HashMap<char, Vec<&str>> = HashMap::new();
    for name in ["Alice", "Bob", "Anna", "Ben", "Carl"] {
        let first = name.chars().next().unwrap(); // names aren't empty, so Some
        // or_default(): insert the type's default (empty Vec) if missing.
        by_letter.entry(first).or_default().push(name);
    }
    println!("by_letter = {:?}", by_letter);

    // ------------------------------------------------------------------
    // Example 5: Iterating
    // ------------------------------------------------------------------
    println!("\n--- Example 5: iterating ---");
    let scores = HashMap::from([("Alice", 90), ("Bob", 72), ("Carol", 85)]);

    // &scores: borrow each (key, value) pair. Order is NOT guaranteed.
    for (name, score) in &scores {
        println!("{name}: {score}");
    }
    // Only keys, or only values.
    let total: i32 = scores.values().sum();
    println!("total of values = {}", total);

    // Need a stable order? Collect the keys and sort them.
    let mut names: Vec<_> = scores.keys().collect();
    names.sort();
    println!("sorted keys = {:?}", names);

    // ------------------------------------------------------------------
    // Example 6: Ownership: String keys are moved in
    // ------------------------------------------------------------------
    println!("\n--- Example 6: ownership ---");
    let key = String::from("color");
    let value = String::from("blue");
    let mut settings = HashMap::new();
    settings.insert(key, value); // key and value are MOVED into the map
    // println!("{}", key); // error: borrow of moved value: `key`

    // get still takes a &str, even though the keys are Strings.
    println!("color = {:?}", settings.get("color"));
}
