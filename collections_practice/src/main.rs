mod collections;
use collections::{
    array, binary_heap, btree_map, btree_set, hash_map, hash_set, linked_list, vec_deque, vector,
};

// Run everything:            cargo run
// Run one topic only:        cargo run -- hash_map
// (each module is fully self-contained, so read them in any order)
fn main() {
    let filter: Option<String> = std::env::args().nth(1);

    let topics: Vec<(&str, fn())> = vec![
        ("array", array::main_test),
        ("vector", vector::main_test),
        ("vec_deque", vec_deque::main_test),
        ("hash_map", hash_map::main_test),
        ("btree_map", btree_map::main_test),
        ("hash_set", hash_set::main_test),
        ("btree_set", btree_set::main_test),
        ("binary_heap", binary_heap::main_test),
        ("linked_list", linked_list::main_test),
    ];

    for (name, run) in topics {
        match &filter {
            Some(wanted) if wanted != name => continue,
            _ => {}
        }
        println!("\n=============== {name} ===============");
        run();
    }
}
