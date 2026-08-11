// ============================================================================
// VecDeque<T>  —  double-ended queue ("deck"). Push/pop O(1) at BOTH ends.
// ----------------------------------------------------------------------------
// Why it exists: `Vec::remove(0)` is O(n) because everything shifts left. If
// your algorithm eats from the front (BFS queues, sliding-window maxima,
// round-robin schedulers, LRU-ish recency lists), a Vec turns an O(n) algorithm
// into O(n^2).
//
// Mental model: a RING BUFFER. One heap block plus a `head` index. Popping the
// front just moves `head` forward — nothing is shifted. When head/tail wrap
// past the end they come back around to index 0. That wrap is exactly why the
// elements are NOT guaranteed contiguous, so a VecDeque cannot hand you a
// single `&[T]` for the whole thing (see as_slices below).
//
// Complexity: push/pop front & back O(1) | index O(1) | insert/remove middle O(n)
// ============================================================================

use std::collections::VecDeque;

pub fn main_test() {
    // ---------------------------------------------------------------- create
    let mut dq: VecDeque<i32> = VecDeque::new();
    let from_vec: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    println!("from_vec = {:?}", from_vec);

    // ------------------------------------------------------ push both ends
    dq.push_back(2); // [2]
    dq.push_back(3); // [2, 3]
    dq.push_front(1); // [1, 2, 3]   <-- this is the O(1) Vec can't do
    dq.push_front(0); // [0, 1, 2, 3]
    println!("after pushes = {:?}", dq);

    // ------------------------------------------------------- pop both ends
    // Both return Option<T> — an empty deque is a normal state, not an error.
    println!("pop_front -> {:?}", dq.pop_front()); // Some(0)
    println!("pop_back  -> {:?}", dq.pop_back()); // Some(3)
    println!("now = {:?}", dq);

    // -------------------------------------------------------------- peeking
    println!("front={:?} back={:?}", dq.front(), dq.back());
    if let Some(f) = dq.front_mut() {
        *f *= 10; // peek and edit in place
    }
    println!("after front_mut = {:?}", dq);

    // ------------------------------------------------ indexing & iteration
    // Indexing is O(1) — internally (head + i) % capacity.
    let d = VecDeque::from(vec![10, 20, 30, 40]);
    println!("d[2] = {}", d[2]);
    println!("get(9) = {:?}", d.get(9));
    let doubled: Vec<i32> = d.iter().map(|x| x * 2).collect();
    println!("doubled = {:?}", doubled);
    println!("sum = {}", d.iter().sum::<i32>());
    println!("contains 30: {}", d.contains(&30));

    // ---------------------------------------------- the ring-buffer showing
    // Because of wrap-around the storage is two runs, not one.
    let mut ring = VecDeque::with_capacity(4);
    ring.push_back(1);
    ring.push_back(2);
    ring.push_front(0); // wraps to the physical end of the buffer
    let (a, b) = ring.as_slices();
    println!("as_slices -> front-run {:?}, back-run {:?}", a, b);
    // make_contiguous() rotates it into one run and returns a real &mut [T]
    let mut ring2 = ring.clone();
    println!("make_contiguous -> {:?}", ring2.make_contiguous());

    // ==================== USE CASE 1: BFS queue ============================
    // Textbook BFS. push_back to enqueue, pop_front to dequeue — both O(1).
    let graph = vec![
        vec![1, 2], // 0 -> 1, 2
        vec![3],    // 1 -> 3
        vec![3],    // 2 -> 3
        vec![],     // 3 -> .
    ];
    println!("BFS order from 0: {:?}", bfs(&graph, 0));

    // ============ USE CASE 2: sliding-window maximum (monotonic deque) ======
    // Classic hard-ish interview problem. Keep INDICES in the deque, largest
    // value at the front, values decreasing toward the back. Each index is
    // pushed once and popped once -> O(n) total, not O(n*k).
    let nums = [1, 3, -1, -3, 5, 3, 6, 7];
    println!("window maxima (k=3): {:?}", sliding_window_max(&nums, 3));

    // ==================== USE CASE 3: fixed-size recent history ============
    // Bounded buffer: drop the oldest once we exceed capacity.
    let mut recent: VecDeque<&str> = VecDeque::new();
    for event in ["login", "click", "scroll", "buy", "logout"] {
        recent.push_back(event);
        if recent.len() > 3 {
            recent.pop_front(); // evict oldest, O(1)
        }
    }
    println!("last 3 events = {:?}", recent);

    // ==================== USE CASE 4: rotation =============================
    let mut r: VecDeque<i32> = (1..=5).collect();
    r.rotate_left(2); // O(min(mid, len-mid)), no reallocation
    println!("rotate_left(2) = {:?}", r);

    // -------------------------------------------------------- interop notes
    let back_to_vec: Vec<i32> = r.clone().into(); // VecDeque -> Vec
    println!("into Vec = {:?}", back_to_vec);

    // When NOT to use it: if you only ever push/pop the back, use Vec — it's
    // simpler, contiguous, and slices for free. Reach for VecDeque the moment
    // the FRONT is in play.
}

fn bfs(graph: &[Vec<usize>], start: usize) -> Vec<usize> {
    let mut visited = vec![false; graph.len()];
    let mut order = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back(start);
    visited[start] = true;

    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &next in &graph[node] {
            if !visited[next] {
                visited[next] = true; // mark on ENQUEUE, not on dequeue,
                queue.push_back(next); // else a node can enter the queue twice
            }
        }
    }
    order
}

fn sliding_window_max(nums: &[i32], k: usize) -> Vec<i32> {
    let mut dq: VecDeque<usize> = VecDeque::new(); // holds indices
    let mut out = Vec::new();

    for i in 0..nums.len() {
        // 1. drop indices that have slid out of the window on the left
        if let Some(&front) = dq.front() {
            if front + k <= i {
                dq.pop_front();
            }
        }
        // 2. drop smaller values from the back — they can never be a max
        //    while nums[i] is in the window, so they are dead weight
        while let Some(&back) = dq.back() {
            if nums[back] <= nums[i] {
                dq.pop_back();
            } else {
                break;
            }
        }
        dq.push_back(i);

        // 3. once the first full window exists, the front is its maximum
        if i + 1 >= k {
            out.push(nums[*dq.front().unwrap()]);
        }
    }
    out
}
