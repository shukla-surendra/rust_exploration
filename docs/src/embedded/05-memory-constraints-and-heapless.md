# 5. Memory Constraints & `heapless`

## The scale is genuinely different

`hello-kernel` reserved a 64 KiB stack
([Systems, Chapter 8](../systems/08-hello-kernel-build-and-linking.md))
inside RAM that starts at `0x40000000` on an emulated machine with
however much QEMU is told to provide. A real microcontroller might have
**20 KB of RAM total** — stack, statics, and any heap all sharing that
one budget, with no swap, no virtual memory
([Systems, Chapter 3](../systems/03-ram-and-virtual-memory.md)'s MMU
story generally doesn't apply — most Cortex-M cores have no MMU at all,
just a much simpler optional MPU for coarse memory *protection*, not
translation).

## No heap by default — and often, no heap at all

`#![no_std]` alone (covered from [Chapter 1](./01-no-std-and-the-embedded-toolchain.md))
removes the standard library, but `alloc` (the crate providing
`Vec`/`String`/`Box`) is *separately* opt-in, and requires you to supply
a **global allocator** — there's no OS to provide one, unlike a hosted
`no_std` kernel that might implement its own bump/paging allocator (see
[OxideOS Concepts, Chapter 3](../oxideos/oxide_cocepts/03_memory_management.md)
for exactly that, at the OS-kernel scale).

```rust
#[global_allocator]
static ALLOCATOR: embedded_alloc::Heap = embedded_alloc::Heap::empty();

fn init_heap() {
    use core::mem::MaybeUninit;
    static mut HEAP_MEM: [MaybeUninit<u8>; 1024] = [MaybeUninit::uninit(); 1024];
    unsafe { ALLOCATOR.init(HEAP_MEM.as_ptr() as usize, 1024) }
}
```

A 1 KB heap, carved explicitly out of a fixed static array — a decision
made deliberately, not a default you get for free. Plenty of embedded
projects skip this entirely and never link in `alloc` at all, using
only stack-allocated and `static` data — genuinely closer to how
`hello-kernel` operates (no heap, no `Box`, everything sized at compile
time) than to how a hosted OS kernel or ordinary Rust program does.

## `heapless` — the collections without the allocator

```rust
use heapless::{Vec, String};

let mut v: Vec<u8, 32> = Vec::new();   // capacity 32, fixed, on the stack
v.push(1).unwrap();                     // Err if already at capacity — no silent growth

let mut s: String<64> = String::new();
```

`heapless::Vec<T, N>` and friends are the same collection *interface*
as `std`'s (push, pop, iteration — see
[Collections, Closures & Iterators](../workbook/05-collections-closures-iterators.md))
with the capacity baked into the **type itself**, as a const generic —
memory for the whole collection is reserved at compile time, on the
stack or in a `static`, and `.push()` returns `Result` instead of
silently reallocating, because there's nowhere for it to reallocate
*to*. This is the practical embedded answer to
[Stack vs Heap](../foundation/stack-vs-heap.md)'s tradeoff pushed to its
logical extreme: when the heap itself might not exist, "stack-allocated,
fixed-capacity, fallible on overflow" becomes the default posture rather
than the exception.

## Stack overflow is a real, silent danger here

With no MMU (per [Systems, Chapter 3](../systems/03-ram-and-virtual-memory.md)),
there's typically no guard page to fault on overflow the way a hosted
OS process gets — a stack that grows past its allocated region can
silently corrupt whatever static data or heap happens to sit adjacent
to it in the flat, unprotected address space, a failure mode that can
be far harder to diagnose than a clean segfault. Deep recursion,
oversized stack-local buffers (`let buf: [u8; 8192] = [0; 8192];` inside
a function on a chip with 20 KB total RAM), and unbounded `heapless`
capacities chosen too large are the concrete things worth watching for
— tools like `cargo-call-stack` exist specifically to estimate worst-
case stack depth ahead of time, since you generally can't just "add more
RAM" to a shipped device the way you'd resize a cloud VM.
