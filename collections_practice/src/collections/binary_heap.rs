// ============================================================================
// BinaryHeap<T>  —  priority queue. Rust's is a MAX-heap by default.
// ----------------------------------------------------------------------------
// Mental model: a complete binary tree stored FLAT in a Vec. For index i:
//     parent = (i-1)/2      left = 2i+1      right = 2i+2
// No pointers, no allocations per node — just arithmetic on a contiguous array,
// which is why it's fast. The only invariant is "every parent >= its children"
// (the heap property). Note that is NOT full sortedness: the max is at the
// root, but the rest is only loosely ordered. Iterating a heap gives you
// arbitrary order — that surprises people.
//
// Complexity: push O(log n) | pop O(log n) | peek O(1) | build from Vec O(n)
//
// The core trade vs a sorted structure: a heap gives you the single best
// element cheaply and doesn't pay to order everything else. When you only need
// the top (or top-k), that's exactly the right amount of work.
// ============================================================================

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn main_test() {
    // -------------------------------------------------- max-heap (default)
    let mut heap = BinaryHeap::new();
    for n in [3, 1, 4, 1, 5, 9, 2, 6] {
        heap.push(n);
    }
    println!("peek (largest) = {:?}", heap.peek()); // O(1), Option<&T>
    println!("len            = {}", heap.len());

    // pop repeatedly = descending order. This is heapsort, O(n log n).
    let mut descending = Vec::new();
    let mut h = heap.clone();
    while let Some(top) = h.pop() {
        descending.push(top);
    }
    println!("popped in order = {:?}", descending);

    // Careful: .iter() does NOT come out sorted — it walks the raw array.
    println!("raw iter (unsorted!) = {:?}", heap.iter().collect::<Vec<_>>());
    // into_sorted_vec() is the ascending-order escape hatch:
    println!("into_sorted_vec      = {:?}", heap.clone().into_sorted_vec());

    // -------------------------------------------- min-heap via Reverse
    // Rust has no MinHeap type. You wrap values in std::cmp::Reverse, which
    // flips their Ord — so the "largest Reverse" is the smallest value.
    let mut min_heap = BinaryHeap::new();
    for n in [3, 1, 4, 1, 5] {
        min_heap.push(Reverse(n));
    }
    if let Some(Reverse(smallest)) = min_heap.peek() {
        println!("min-heap peek = {}", smallest); // destructure to unwrap
    }
    let mut ascending = Vec::new();
    while let Some(Reverse(n)) = min_heap.pop() {
        ascending.push(n);
    }
    println!("min-heap drained = {:?}", ascending);

    // -------------------------------------------------- build in O(n)
    // Heapifying an existing Vec is O(n), NOT O(n log n) — cheaper than
    // pushing one at a time. Use From/into when you already have the data.
    let bulk: BinaryHeap<i32> = vec![7, 2, 9, 4].into();
    println!("heapified peek = {:?}", bulk.peek());

    // ---------------------------------------- priority by tuple ordering
    // Tuples compare lexicographically: first field, then second as tie-break.
    // A (priority, payload) tuple is the cheapest custom-priority trick there is.
    let mut tasks = BinaryHeap::new();
    tasks.push((2, "write tests"));
    tasks.push((5, "fix prod outage"));
    tasks.push((1, "update readme"));
    tasks.push((5, "page oncall")); // same priority, tie broken by the &str
    while let Some((p, task)) = tasks.pop() {
        println!("  [p{}] {}", p, task);
    }

    // ---------------------------------------- custom Ord on your own type
    // When tuple ordering isn't enough, implement Ord. Note we compare
    // other.cost.cmp(&self.cost) — reversing the comparison turns the max-heap
    // into a min-heap for this type, which is what Dijkstra needs.
    #[derive(PartialEq, Eq, Debug)]
    struct Job {
        cost: u32,
        name: &'static str,
    }
    impl Ord for Job {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.cost.cmp(&self.cost) // reversed => smallest cost wins
        }
    }
    impl PartialOrd for Job {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    let mut jobs = BinaryHeap::new();
    jobs.push(Job { cost: 50, name: "big" });
    jobs.push(Job { cost: 5, name: "small" });
    jobs.push(Job { cost: 20, name: "mid" });
    println!("cheapest job = {:?}", jobs.pop());

    // ==================== USE CASE 1: top-k in O(n log k) ==================
    // Sorting everything to take 3 is O(n log n) and wastes work. Keep a
    // min-heap of size k: the smallest of the current best-k sits at the top,
    // ready to be evicted. Memory is O(k), not O(n) — that matters for streams
    // that don't fit in RAM.
    let stream = [7, 1, 9, 3, 12, 5, 8];
    println!("top 3 of {:?} = {:?}", stream, top_k(&stream, 3));

    // ==================== USE CASE 2: merge k sorted lists =================
    let lists = vec![vec![1, 4, 7], vec![2, 5, 8], vec![3, 6, 9]];
    println!("merged = {:?}", merge_sorted(&lists));

    // ==================== USE CASE 3: Dijkstra's frontier ==================
    // The heap is what makes Dijkstra O(E log V): always expand the currently
    // cheapest node. Reverse gives us the min-heap.
    let graph: Vec<Vec<(usize, u32)>> = vec![
        vec![(1, 4), (2, 1)], // 0 -> 1 (4), 0 -> 2 (1)
        vec![(3, 1)],         // 1 -> 3 (1)
        vec![(1, 2), (3, 5)], // 2 -> 1 (2), 2 -> 3 (5)
        vec![],
    ];
    println!("shortest distances from 0 = {:?}", dijkstra(&graph, 0));

    // ------------------------------------------------------- known limitation
    // BinaryHeap has NO "decrease-key" and no way to remove an arbitrary
    // element. The standard workaround (used in dijkstra below) is to push the
    // improved entry and skip stale ones on pop. If you truly need keyed
    // updates, reach for a BTreeSet of (priority, id) instead.
}

fn top_k(nums: &[i32], k: usize) -> Vec<i32> {
    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
    for &n in nums {
        heap.push(Reverse(n));
        if heap.len() > k {
            heap.pop(); // evicts the SMALLEST — heap stays at size k
        }
    }
    let mut out: Vec<i32> = heap.into_iter().map(|Reverse(n)| n).collect();
    out.sort_by(|a, b| b.cmp(a));
    out
}

fn merge_sorted(lists: &[Vec<i32>]) -> Vec<i32> {
    // Heap holds one candidate per list: (value, which list, index in it).
    // Pop the global smallest, then push that list's next element. O(n log k).
    let mut heap: BinaryHeap<Reverse<(i32, usize, usize)>> = BinaryHeap::new();
    for (li, list) in lists.iter().enumerate() {
        if let Some(&first) = list.first() {
            heap.push(Reverse((first, li, 0)));
        }
    }
    let mut out = Vec::new();
    while let Some(Reverse((val, li, idx))) = heap.pop() {
        out.push(val);
        if let Some(&next) = lists[li].get(idx + 1) {
            heap.push(Reverse((next, li, idx + 1)));
        }
    }
    out
}

fn dijkstra(graph: &[Vec<(usize, u32)>], start: usize) -> Vec<u32> {
    let mut dist = vec![u32::MAX; graph.len()];
    let mut heap = BinaryHeap::new();

    dist[start] = 0;
    heap.push(Reverse((0u32, start))); // (distance, node) — min-heap

    while let Some(Reverse((d, node))) = heap.pop() {
        if d > dist[node] {
            continue; // stale entry from before we found a better path
        }
        for &(next, weight) in &graph[node] {
            let candidate = d + weight;
            if candidate < dist[next] {
                dist[next] = candidate;
                heap.push(Reverse((candidate, next)));
            }
        }
    }
    dist
}
