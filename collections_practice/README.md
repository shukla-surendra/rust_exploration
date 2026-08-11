# collections_practice

One file per Rust collection, tutorial written as inline comments. Same shape as
`string_practice/`: each module exposes `pub fn main_test()`, `main.rs` runs them.

```bash
cargo run                 # run every topic
cargo run -- hash_map     # run one topic
```

| File | Type | Ops | Ordered? | Reach for it when |
|---|---|---|---|---|
| `array.rs` | `[T; N]`, `&[T]` | index O(1) | insertion | fixed size known at compile time; write helpers against `&[T]` |
| `vector.rs` | `Vec<T>` | push/pop back O(1)*, index O(1) | insertion | the default, 90% of the time |
| `vec_deque.rs` | `VecDeque<T>` | push/pop both ends O(1) | insertion | the FRONT is in play — BFS, sliding window, bounded history |
| `hash_map.rs` | `HashMap<K,V>` | get/insert O(1) avg | no (randomized) | key → value lookup; turning O(n²) into O(n) |
| `btree_map.rs` | `BTreeMap<K,V>` | O(log n) | sorted by key | range queries, first/last, deterministic output |
| `hash_set.rs` | `HashSet<T>` | contains O(1) avg | no (randomized) | "seen before?", dedup, set algebra |
| `btree_set.rs` | `BTreeSet<T>` | O(log n) | sorted | sorted unique values, ranges, min/max |
| `binary_heap.rs` | `BinaryHeap<T>` | push/pop O(log n), peek O(1) | heap order only | "give me the best one" — top-k, Dijkstra, k-way merge |
| `linked_list.rs` | `LinkedList<T>` | splice O(1), everything else O(n) | insertion | almost never — file explains why, plus a `Box`-based list built from scratch |

`*` amortized — see the capacity trace at the top of `vector.rs`.

Worked problems embedded in the files: two-sum and group-anagrams (`hash_map`),
sliding-window maximum and BFS (`vec_deque`), longest-consecutive-sequence
(`hash_set`), top-k / merge-k-sorted / Dijkstra (`binary_heap`), reverse a list
in place (`linked_list`).
