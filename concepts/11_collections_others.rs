// Other collections: VecDeque, BTreeMap, BTreeSet, BinaryHeap
//
// Vec, HashMap and HashSet cover most needs (see 08_collections_vec.rs,
// 09_collections_hashmap.rs, 10_collections_hashset.rs). Reach for these when you need
// something those three are slow at, or don't do.
//
// | Collection    | Like...              | Pick it when you need                          |
// |---------------|----------------------|------------------------------------------------|
// | Vec<T>        | list                 | the default: ordered items, add/remove at end  |
// | VecDeque<T>   | Vec, open both ends  | fast add/remove at the FRONT too (queues)      |
// | HashMap<K,V>  | dictionary           | fast lookup by key, order doesn't matter       |
// | BTreeMap<K,V> | HashMap, sorted      | keys kept SORTED, or ranges ("keys 10..20")    |
// | HashSet<T>    | unique values        | fast "is it in there?", order doesn't matter   |
// | BTreeSet<T>   | HashSet, sorted      | unique values kept SORTED                      |
// | BinaryHeap<T> | priority queue       | always take the LARGEST item next              |
//
// All of them need a `use std::collections::...;` line.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};

fn main() {
    // ------------------------------------------------------------------
    // Example 1: VecDeque: a queue (first in, first out)
    // ------------------------------------------------------------------
    println!("--- Example 1: VecDeque ---");
    // Vec::remove(0) is slow: every other item shifts left. VecDeque ("vec deck",
    // double-ended queue) adds and removes at BOTH ends quickly.
    let mut queue = VecDeque::new();
    queue.push_back("customer 1"); // join at the back
    queue.push_back("customer 2");
    queue.push_back("customer 3");
    queue.push_front("VIP"); // jump to the front
    println!("queue = {:?}", queue);

    // pop_front returns Option: None when the queue is empty.
    while let Some(next) = queue.pop_front() {
        println!("serving {}", next);
    }

    // ------------------------------------------------------------------
    // Example 2: BTreeMap: a map with SORTED keys
    // ------------------------------------------------------------------
    println!("\n--- Example 2: BTreeMap ---");
    // Same methods as HashMap (insert, get, entry, remove), but iterating
    // always goes in key order.
    let mut scores = BTreeMap::new();
    scores.insert("Carol", 85);
    scores.insert("Alice", 90);
    scores.insert("Bob", 72);
    for (name, score) in &scores {
        println!("{name}: {score}"); // always Alice, Bob, Carol
    }

    // Ranges: something HashMap can't do. Keys from 2 up to (not including) 5.
    let mut by_day = BTreeMap::new();
    for day in 1..=7 {
        by_day.insert(day, format!("day {day} notes"));
    }
    let midweek: Vec<_> = by_day.range(2..5).map(|(day, _)| day).collect();
    println!("days in 2..5: {:?}", midweek);
    println!("first: {:?}, last: {:?}", by_day.first_key_value(), by_day.last_key_value());

    // ------------------------------------------------------------------
    // Example 3: BTreeSet: unique AND sorted
    // ------------------------------------------------------------------
    println!("\n--- Example 3: BTreeSet ---");
    let numbers = vec![5, 1, 5, 3, 1, 9];
    let set: BTreeSet<i32> = numbers.iter().cloned().collect();
    println!("{:?} -> {:?}", numbers, set); // duplicates gone, sorted: {1, 3, 5, 9}
    println!("smallest = {:?}, largest = {:?}", set.first(), set.last());

    // ------------------------------------------------------------------
    // Example 4: BinaryHeap: always get the largest next
    // ------------------------------------------------------------------
    println!("\n--- Example 4: BinaryHeap ---");
    // push in any order; pop always returns the LARGEST remaining item.
    let mut heap = BinaryHeap::new();
    for n in [3, 8, 1, 6] {
        heap.push(n);
    }
    println!("peek (largest) = {:?}", heap.peek()); // Some(8), not removed
    while let Some(n) = heap.pop() {
        print!("{} ", n); // 8 6 3 1
    }
    println!("<- largest first");

    // Want the SMALLEST first? Wrap items in Reverse, which flips the comparison.
    let mut min_heap = BinaryHeap::new();
    for n in [3, 8, 1, 6] {
        min_heap.push(Reverse(n));
    }
    while let Some(Reverse(n)) = min_heap.pop() {
        print!("{} ", n); // 1 3 6 8
    }
    println!("<- smallest first");

    // Real-world: tasks by priority. Tuples compare by their first item first.
    let mut tasks = BinaryHeap::new();
    tasks.push((1, "tidy desk"));
    tasks.push((5, "fix production bug"));
    tasks.push((3, "reply to email"));
    while let Some((priority, task)) = tasks.pop() {
        println!("priority {priority}: {task}");
    }
}
