// ============================================================================
// LinkedList<T>  —  doubly-linked list. Real, in std, and almost always the
//                   WRONG choice. Read this file for why, not for how.
// ----------------------------------------------------------------------------
// Mental model: each element is its own heap allocation holding (prev, next,
// value). Nothing is contiguous.
//
// The textbook pitch is "O(1) insert/remove in the middle". The catch: that's
// O(1) only if you ALREADY hold a cursor at that position. Getting there is
// O(n) — there's no indexing. Meanwhile every hop is a pointer chase to a
// random heap address, so you take a cache miss per element, while a Vec's
// "slow" O(n) memmove runs at memory bandwidth on prefetched cache lines. In
// practice Vec/VecDeque beat LinkedList even at the operations the list is
// supposed to win.
//
// Rust adds a second problem: a doubly-linked list means two owners per node,
// which the borrow checker forbids outright. std's LinkedList is implemented
// with raw pointers and unsafe internally. Writing your own is the classic
// "fighting the borrow checker" exercise (see the Box<T> version at the bottom).
//
// Interview angle: knowing WHEN NOT to use a linked list, and being able to say
// "cache locality beats asymptotic complexity at these sizes," is a stronger
// signal than reciting the O(1) claim.
// ============================================================================

use std::collections::LinkedList;

pub fn main_test() {
    // ---------------------------------------------------- std::LinkedList
    let mut list: LinkedList<i32> = LinkedList::new();
    list.push_back(2);
    list.push_back(3);
    list.push_front(1);
    println!("list = {:?}", list);

    println!("front = {:?}, back = {:?}", list.front(), list.back());
    println!("pop_front = {:?}", list.pop_front());
    println!("pop_back  = {:?}", list.pop_back());
    println!("now = {:?} len = {}", list, list.len());

    // NO INDEXING. `list[2]` does not compile. You walk:
    let walk: LinkedList<i32> = (1..=5).collect();
    let third = walk.iter().nth(2); // O(n)
    println!("3rd element via iter().nth(2) = {:?}", third);
    println!("sum = {}", walk.iter().sum::<i32>());
    println!("contains 4 = {}", walk.contains(&4));

    // ------------------------------------------- the ONE genuine advantage
    // append() splices two lists in O(1) — just relink two pointers. No
    // copying, no reallocation, regardless of size. Vec::append is O(n).
    let mut a: LinkedList<i32> = (1..=3).collect();
    let mut b: LinkedList<i32> = (4..=6).collect();
    a.append(&mut b); // b is left empty; O(1) pointer surgery
    println!("appended = {:?}, b is now {:?}", a, b);

    // split_off is the mirror image, but O(n) to walk to the split point.
    let mut s: LinkedList<i32> = (1..=6).collect();
    let tail = s.split_off(3);
    println!("split_off(3) -> head {:?} | tail {:?}", s, tail);

    // ------------------------------------------------- the honest comparison
    // Need                        Reach for
    // -----------------------     -------------------------------------------
    // indexed access              Vec
    // push/pop at back            Vec
    // push/pop at both ends       VecDeque       <- what people usually want
    // O(1) splice of huge lists   LinkedList     <- the actual niche
    // O(1) lookup by key          HashMap
    // sorted + ranges             BTreeMap / BTreeSet
    // "give me the best one"      BinaryHeap
    //
    // If you're about to use LinkedList, ask "would VecDeque do?" — it almost
    // always does, and it's faster.

    // ==================== BUILD ONE: singly-linked with Box ================
    // The educational half. Box<T> = a single owned heap pointer, which is
    // exactly the ownership shape a SINGLY-linked list needs: each node owns
    // the next one, so there's one owner per node and the borrow checker is
    // happy. Option<Box<Node>> is the "next or nothing" link, and Rust's null
    // pointer optimization makes it the same size as a raw pointer.
    let mut my = Stack::new();
    my.push(1);
    my.push(2);
    my.push(3);
    println!("custom stack       = {:?}", my.to_vec());
    println!("peek               = {:?}", my.peek());
    println!("pop                = {:?}", my.pop());
    println!("after pop          = {:?}", my.to_vec());
    println!("len                = {}", my.len());
    my.reverse();
    println!("reversed           = {:?}", my.to_vec());

    // Why the DOUBLY-linked version isn't here: node B would need an owning
    // link from A AND a back-link from C — two owners. In safe Rust that needs
    // Rc<RefCell<Node>> (reference counting + runtime borrow checks), which
    // leaks on cycles unless the back-links are Weak. That's the real lesson:
    // the data structure you'd sketch on a whiteboard in C is the one Rust
    // makes you justify.
}

// ---------------------------------------------------------------------------
// A singly-linked stack. `Option<Box<Node<T>>>` is the idiomatic link type.
// ---------------------------------------------------------------------------
#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>, // None = end of list
}

#[derive(Debug)]
struct Stack<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<T: Clone + std::fmt::Debug> Stack<T> {
    fn new() -> Self {
        Stack { head: None, len: 0 }
    }

    // O(1): the new node takes over the old head.
    fn push(&mut self, value: T) {
        // .take() swaps the field out for None and hands us ownership. You
        // cannot just read self.head — that would move a field out of &mut self.
        // take() is THE move-out-of-a-struct-field idiom in Rust.
        let node = Box::new(Node {
            value,
            next: self.head.take(),
        });
        self.head = Some(node);
        self.len += 1;
    }

    // O(1): unlink the head and return its value.
    fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next; // node is owned here, so we can move out
            self.len -= 1;
            node.value
        })
    }

    // as_ref() turns &Option<Box<Node>> into Option<&Box<Node>> so we can look
    // without taking ownership.
    fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    fn len(&self) -> usize {
        self.len
    }

    // Walk the chain with a moving borrow — the manual version of an iterator.
    fn to_vec(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut cur = self.head.as_ref();
        while let Some(node) = cur {
            out.push(node.value.clone());
            cur = node.next.as_ref();
        }
        out
    }

    // Classic interview problem: reverse in place, O(n) time, O(1) extra space.
    // Three-pointer walk — in Rust the "pointers" are owned Options.
    fn reverse(&mut self) {
        let mut prev: Option<Box<Node<T>>> = None;
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take(); // remember the rest
            node.next = prev; // flip this link backwards
            prev = Some(node); // this node becomes the new "prev"
        }
        self.head = prev;
    }
}
