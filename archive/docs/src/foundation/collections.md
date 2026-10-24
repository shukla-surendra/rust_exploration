# Arrays, `Vec`, `HashSet` & `HashMap`

> **Coming from Python:** the mapping is *almost* one-to-one, and the places it
> isn't are where the bugs come from.
>
> | Python | Rust | The catch |
> |---|---|---|
> | `list` | `Vec<T>` | Rust's is homogeneous — one element type, enforced at compile time |
> | tuple-as-fixed-record | `[T; N]` (array) | length is part of the *type*; `[i32; 3]` and `[i32; 4]` are different types |
> | `set` | `HashSet<T>` | both unordered — closest match of the five |
> | `dict` | `HashMap<K, V>` | **Python `dict` preserves insertion order since 3.7. Rust's `HashMap` does not.** |
> | — | `BTreeMap` / `BTreeSet` | no direct Python equivalent; sorted, supports range queries |
>
> The `dict` row is the one that catches people. If you have internalised
> "dictionaries remember their order," you have to un-learn it here.

## The direct answer: is a `HashSet` just a `Vec` with duplicates removed?

No — and the difference isn't only about duplicates. You give up two things.

```rust
let words = ["pear", "apple", "fig", "apple", "date"];

let v: Vec<&str>     = words.to_vec();
let s: HashSet<&str> = words.iter().copied().collect();
let b: BTreeSet<&str>= words.iter().copied().collect();
```

Actual output:

```text
input   : ["pear", "apple", "fig", "apple", "date"]
Vec     : ["pear", "apple", "fig", "apple", "date"]   order kept, duplicate kept
HashSet : {"apple", "fig", "date", "pear"}            deduped, ORDER GONE
BTreeSet: {"apple", "date", "fig", "pear"}            deduped, SORTED
```

Two losses, both permanent:

1. **Order is gone.** Not "changed" — *gone*. `HashSet` has no concept of a first
   or last element.
2. **Indexing is gone.** `v[0]` compiles; `s[0]` does not. `HashSet` has no
   `Index` implementation, because there is no meaningful position to index.

And the ordering situation is stronger than "not insertion order." Run the *same
binary* three times:

```text
HashSet<i32> iteration order this run: [7, 4, 2, 0, 1, 5, 6, 3]
HashSet<i32> iteration order this run: [6, 5, 7, 2, 0, 1, 4, 3]
HashSet<i32> iteration order this run: [7, 0, 4, 2, 6, 5, 3, 1]
```

Different every time. This is deliberate: the default hasher (`RandomState`) is
seeded randomly per process, to make hash-collision denial-of-service attacks
impractical. The consequence for you is a rule:

> **Never write a test, or any logic, that depends on `HashSet`/`HashMap`
> iteration order.** It will pass locally and fail in CI, or pass a hundred times
> and fail once.

If you need "unique *and* ordered," a hash set is the wrong tool — see
[the last section](#choosing-between-them).

So the honest framing is not "a set is a deduplicated list." It's:

> **A `Vec` answers "what is the sequence?" A `HashSet` answers "is this thing
> present?" They are different questions, and the data structure that answers the
> second one fast cannot answer the first one at all.**

## Array `[T; N]` — fixed size, known at compile time

```rust
let counts: [u16; 9] = [0; 9];      // nine zeros
let dirs = [(-1, 0), (1, 0), (0, -1), (0, 1)];   // length inferred: 4
```

The length is **part of the type**. `[i32; 3]` and `[i32; 4]` are as different as
`i32` and `String` — you cannot assign one to the other, and a function taking
`[i32; 3]` will not accept `[i32; 4]`.

That sounds restrictive and buys you something real: arrays live **inline** —
on the stack if the variable is on the stack — with no heap allocation and no
pointer to follow. When the size is genuinely fixed (a Sudoku board, a
direction table, a fixed-size buffer), this is the right choice, and reaching
for `Vec` out of habit is a small unnecessary cost.

Build one without repeating yourself:

```rust
let mut sets: [HashSet<char>; 9] = std::array::from_fn(|_| HashSet::new());
```

`array::from_fn` calls the closure once per index, so you get nine independent
values — and unlike `vec![x; 9]` it needs no `Clone` bound.

### Worked example: a lookup table that can't be the wrong size

```rust
// The four grid moves. Length 4 is guaranteed by the type — a fifth entry
// or a missing one is a compile error, not a runtime surprise.
const DIRS4: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

// One 9-bit mask per Sudoku row. 18 bytes, stack, zero allocation.
let mut row_masks = [0u16; 9];
row_masks[3] |= 1 << 5;                     // row 3 has seen a '6'
assert!(row_masks[3] & (1 << 5) != 0);
```

### Common array methods

Arrays have very few methods of their own — almost everything below is a
**slice** method reachable through auto-deref, which is why the slice section
is worth reading even if you only ever use arrays.

| Method | Does | Note |
|---|---|---|
| `[x; N]` | N copies of `x` | needs `Copy`, or `Clone` in `vec!` |
| `std::array::from_fn(\|i\| ..)` | build from index | fresh value per slot; no `Clone` needed |
| `.len()` | element count | a compile-time constant |
| `.map(\|x\| ..)` | transform, returns `[U; N]` | **array-specific** — `Vec` has no `.map`, use `.iter().map()` |
| `.iter()` | borrowing iterator | yields `&T` |
| `.into_iter()` | consuming iterator | yields `T` (since Rust 2021) |
| `.to_vec()` | copy into a `Vec` | allocates |
| `.as_slice()` | view as `&[T]` | usually implicit via `&arr` |

## `Vec<T>` — growable, heap-allocated, ordered

The default sequence type, and what you want whenever the length isn't known at
compile time.

```rust
let mut v = Vec::new();
v.push(10);
v.push(20);
println!("{}", v[0]);          // 10
```

A `Vec` is three machine words on the stack — **pointer, length, capacity** — 24
bytes total — pointing at a heap buffer. `len` is how many elements exist;
`capacity` is how many fit before it must reallocate.

Watch capacity as you push:

```text
capacity growth: [0, 8, 16, 32, 64]
```

It **doubles**. That's what makes `push` *amortized* O(1): most pushes are free,
an occasional push copies everything to a bigger buffer, and the doubling means
those expensive pushes get rarer exponentially. If you know the final size,
`Vec::with_capacity(n)` skips the intermediate copies entirely.

Indexing has two forms, and the difference matters:

```rust
let x = v[10];          // PANICS if out of bounds
let y = v.get(10);      // returns Option<&T> — None if out of bounds
```

`v[10]` is for when an out-of-range index is a bug you want to hear about
immediately. `v.get(10)` is for when it's an expected condition you'll handle —
which is why every bounds-checked helper in the grid scaffold returns `Option`.

### Worked example: top-3 most frequent words

```rust
let text = "the quick brown fox jumps over the lazy dog the fox";

let mut freq: HashMap<&str, u32> = HashMap::new();
for w in text.split_whitespace() {
    *freq.entry(w).or_insert(0) += 1;
}

let mut ranked: Vec<(&str, u32)> = freq.into_iter().collect();
// count descending, then word ascending as a stable tie-break
ranked.sort_by_key(|&(word, count)| (std::cmp::Reverse(count), word));

println!("{:?}", &ranked[..3]);
// [("the", 3), ("fox", 2), ("brown", 1)]
```

Two things to notice. `HashMap` → `Vec` → `sort` is the standard way to get
ordered output out of a hash map — there's no "sorted iteration" to ask for.
And `Reverse` inside a tuple key gives you "descending by count, ascending by
word" without writing a comparator.

### Common `Vec` methods

**Creating**

| Method | Does |
|---|---|
| `Vec::new()` | empty, no allocation yet |
| `vec![a, b, c]` | from a list |
| `vec![x; n]` | `n` clones of `x` — see the `Clone` trap in the syntax notes |
| `Vec::with_capacity(n)` | empty but pre-allocated; skips regrowth |
| `iter.collect()` | from any iterator |

**Adding and removing**

| Method | Does | Cost / gotcha |
|---|---|---|
| `.push(x)` | append | amortized O(1) |
| `.pop()` | remove last, `Option<T>` | O(1) |
| `.insert(i, x)` | insert at index | O(n) — shifts everything right |
| `.remove(i)` | remove at index, returns it | O(n) — shifts left |
| `.swap_remove(i)` | remove at index | **O(1), but reorders** — moves the last element into the hole |
| `.truncate(n)` | keep first `n` | |
| `.clear()` | remove all | keeps capacity |
| `.extend(iter)` | append many | |
| `.append(&mut other)` | move all of `other` in | leaves `other` empty |
| `.drain(range)` | remove a range, yields them | useful for "take these out and process them" |
| `.retain(\|x\| ..)` | keep only matching | in place, O(n), preserves order |
| `.dedup()` | remove **consecutive** duplicates | **sort first**, see below |

**Accessing**

| Method | Does |
|---|---|
| `v[i]` | panics if out of range |
| `.get(i)` / `.get_mut(i)` | `Option<&T>` / `Option<&mut T>` |
| `.first()` / `.last()` | `Option<&T>` |
| `.len()` / `.is_empty()` / `.capacity()` | |
| `.swap(i, j)` | exchange two elements |

**Searching and ordering**

| Method | Does | Note |
|---|---|---|
| `.contains(&x)` | membership | **O(n)** — this is what `HashSet` fixes |
| `.iter().position(\|x\| ..)` | index of first match | `Option<usize>` |
| `.sort()` | stable sort | needs `Ord` |
| `.sort_unstable()` | faster, order of equals unspecified | prefer when you don't need stability |
| `.sort_by(\|a, b\| ..)` / `.sort_by_key(\|x\| ..)` | custom ordering | `sort_by_key` is usually clearer |
| `.binary_search(&x)` | O(log n) lookup | **requires sorted**; `Err(i)` gives the insertion point |
| `.reverse()` | in place | |

**Viewing in pieces**

| Method | Yields |
|---|---|
| `.iter()` / `.iter_mut()` / `.into_iter()` | `&T` / `&mut T` / `T` |
| `.chunks(n)` | non-overlapping groups: `[40,20],[30]` |
| `.windows(n)` | overlapping runs: `[40,20],[20,30]` |
| `.split_at(i)` | two slices |
| `.concat()` / `.join(sep)` | flatten a `Vec<Vec<T>>` or `Vec<&str>` |

### The `dedup` trap

`dedup` only removes duplicates that are **already adjacent**:

```rust
let mut v = vec![1, 2, 1, 2, 1];
v.dedup();
// [1, 2, 1, 2, 1]   <- nothing removed!

let mut v = vec![1, 2, 1, 2, 1];
v.sort();
v.dedup();
// [1, 2]
```

`sort` then `dedup` is the idiom, and it's O(n log n). If you don't need the
ordering, collecting into a `HashSet` is O(n).

## Slices `&[T]` — the glue you'll under-use at first

A **slice** is a borrowed view into a contiguous run of elements: a pointer plus
a length, 16 bytes, owning nothing.

```rust
fn total(xs: &[i32]) -> i32 { xs.iter().sum() }

let arr = [1, 2, 3];
let vec = vec![1, 2, 3];

total(&arr);        // works
total(&vec);        // also works
total(&vec[1..]);   // and so does a sub-range
```

This is the array/`Vec` counterpart of the `&str` rule from
[Strings](./strings.md): **accept `&[T]` in function parameters, return `Vec<T>`
when you're producing new data.** Writing `fn total(xs: Vec<i32>)` forces every
caller to hand over ownership or clone — the same mistake as taking `String`
where `&str` would do.

### Worked example: one function, three callers

```rust
fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() { return None; }
    Some(xs.iter().sum::<f64>() / xs.len() as f64)
}

let arr = [1.0, 2.0, 3.0, 4.0];
let vc  = vec![10.0, 20.0];

mean(&arr);         // Some(2.5)   — an array
mean(&vc);          // Some(15.0)  — a Vec
mean(&arr[1..3]);   // Some(2.5)   — a sub-range of an array
mean(&[]);          // None
```

Written as `fn mean(xs: Vec<f64>)`, only the second call would work, and it
would consume `vc`.

### Common slice methods

Everything in the `Vec` tables above under **Accessing**, **Searching and
ordering**, and **Viewing in pieces** is really a slice method — `Vec` gets them
by dereferencing to `[T]`. That's why `.sort()`, `.binary_search()`,
`.windows()`, and `.contains()` work identically on `[T; N]`, `Vec<T>`, and
`&mut [T]`.

The distinction: methods that **change the length** (`push`, `pop`, `insert`,
`remove`, `truncate`, `retain`) belong to `Vec` alone, because a slice doesn't
own its storage and can't grow. Everything that only reads or reorders in place
is a slice method.

## `HashSet<T>` — membership, not sequence

```rust
let mut seen = HashSet::new();
seen.insert("apple");

if seen.contains("apple") { /* ... */ }
```

The single reason to choose it: **`contains` is O(1) average** instead of `Vec`'s
O(n) scan. You pay memory and lose ordering to get that.

| Operation | `Vec<T>` | `HashSet<T>` |
|---|---|---|
| `contains` | O(n) | **O(1) average** |
| push / insert | O(1) amortized | O(1) average |
| index by position | O(1) | **not possible** |
| keeps insertion order | yes | **no** |
| allows duplicates | yes | no |

Two mechanics worth knowing.

**`insert` returns `bool`** — `true` if the value was newly added. So the
membership check and the recording step are one operation:

```rust
if !seen.insert(value) {
    // value was already present
}
```

One hash lookup instead of two, and the check can never drift out of sync with
the insert.

**`T` must implement `Hash + Eq`.** The set has to hash a value to find its
bucket and compare for equality on collision. This is why `f64` keys are
rejected: `NaN != NaN` breaks the reflexivity `Eq` requires. Tuples and structs
get these traits structurally or by `#[derive(Hash, Eq, PartialEq)]`.

### Set algebra — the feature most people never find

This is the real reason to reach for `HashSet` beyond deduplication, and it
turns a page of loops into one line.

```rust
let before: HashSet<&str> = ["api", "web", "cache"].into();
let after:  HashSet<&str> = ["api", "web", "worker"].into();

after.difference(&before)            // ["worker"]  — added
before.difference(&after)            // ["cache"]   — removed
before.intersection(&after)          // ["api", "web"] — unchanged
before.symmetric_difference(&after)  // ["cache", "worker"] — changed either way
before.union(&after)                 // all four
```

"What changed between two deploys / two configs / two permission sets" is a
three-line answer, not a nested loop. All five return **iterators**, so you
`.collect()` or iterate them — and since they come from a hash set, they arrive
in no particular order.

There are also three predicates that return `bool` directly:
`is_subset`, `is_superset`, `is_disjoint`.

### Common `HashSet` methods

| Method | Does | Note |
|---|---|---|
| `HashSet::new()` / `::with_capacity(n)` | create | `new` doesn't allocate |
| `.insert(x)` | add | **returns `bool`** — `false` if already present |
| `.remove(&x)` | delete | returns `bool` — whether it was there |
| `.contains(&x)` | membership | O(1) average |
| `.get(&x)` | `Option<&T>` | useful when `T` carries more than the key part |
| `.take(&x)` | remove **and return** it | `Option<T>` |
| `.len()` / `.is_empty()` / `.clear()` | | |
| `.retain(\|x\| ..)` | keep only matching | in place |
| `.extend(iter)` | add many | |
| `.drain()` | remove all, yielding them | keeps capacity |
| `.union` / `.intersection` / `.difference` / `.symmetric_difference` | set algebra | all return iterators |
| `.is_subset` / `.is_superset` / `.is_disjoint` | relations | return `bool` |
| `.iter()` | borrowing iterator | **order is arbitrary and varies per run** |

## `HashMap<K, V>` — the same machine, with values attached

```rust
let mut ages = HashMap::new();
ages.insert("alice", 30);

match ages.get("alice") {
    Some(age) => println!("{age}"),
    None      => println!("unknown"),
}
```

`HashSet<T>` is essentially `HashMap<T, ()>` — a map whose values carry no
information. Everything true of one is true of the other: O(1) average lookup,
`Hash + Eq` on the key, no iteration-order guarantee. Both are 48 bytes on the
stack.

`get` returns `Option<&V>`, not `V`. That's not ceremony — a missing key is a
real possibility and the type makes you handle it, where Python raises `KeyError`
at runtime.

### The `entry` API — learn this one

The pattern you'll reach for constantly is "update if present, insert a default
if not." The naive version does two lookups:

```rust
if !counts.contains_key(&ch) {      // lookup 1
    counts.insert(ch, 0);           // lookup 2
}
*counts.get_mut(&ch).unwrap() += 1; // lookup 3
```

`entry` does it in one, with no `unwrap`:

```rust
let mut counts: HashMap<char, u32> = HashMap::new();
for ch in "hello".chars() {
    *counts.entry(ch).or_insert(0) += 1;
}
// [('e', 1), ('h', 1), ('l', 2), ('o', 1)]
```

`entry(ch)` finds the slot once and hands back a handle to it; `or_insert(0)`
fills it if empty and returns `&mut V` either way. This is Rust's
`collections.Counter` / `defaultdict`, and it's the idiom that most marks
someone as comfortable with the language.

Use `or_insert_with(Vec::new)` when the default is expensive to construct — the
closure only runs if the slot is actually empty.

The four `entry` finishers worth knowing:

| Call | Behaviour |
|---|---|
| `.or_insert(v)` | fill with `v` if empty; returns `&mut V` |
| `.or_insert_with(\|\| ..)` | same, but only builds the default when needed |
| `.or_default()` | fill with `V::default()` — `0`, `""`, empty `Vec` |
| `.and_modify(\|v\| ..)` | mutate **only if already present**; chains before the above |

Chained, they read as a single sentence:

```rust
map.entry(key).and_modify(|v| *v += 1).or_insert(1);
// "increment if present, otherwise start at 1"
```

### `insert` returns the *old* value

A gotcha that differs from `HashSet`:

```rust
m.insert("a", 1);   // None      — no previous value
m.insert("a", 2);   // Some(1)   — the value that got replaced
```

`HashSet::insert` returns `bool`; `HashMap::insert` returns `Option<V>`. Both
tell you "was something already here," but the map hands back what it evicted.

### Common `HashMap` methods

| Method | Does | Note |
|---|---|---|
| `HashMap::new()` / `::with_capacity(n)` | create | |
| `.insert(k, v)` | add or replace | returns `Option<V>`, the **old** value |
| `.get(&k)` / `.get_mut(&k)` | look up | `Option<&V>` / `Option<&mut V>` |
| `.remove(&k)` | delete | returns `Option<V>` |
| `.contains_key(&k)` | membership | prefer `entry` if you'll then insert |
| `.entry(k)` | get-or-insert handle | the idiom above; one lookup |
| `.keys()` / `.values()` / `.values_mut()` | iterate one side | |
| `.iter()` / `.iter_mut()` | yields `(&K, &V)` | **arbitrary order** |
| `.len()` / `.is_empty()` / `.clear()` | | |
| `.retain(\|k, v\| ..)` | keep only matching | closure takes **both** |
| `.extend(iter)` | merge in pairs | later keys overwrite |
| `.get(&k).copied().unwrap_or(d)` | lookup with a default | common shorthand for `Copy` values |

## When you need order back: `BTreeSet` / `BTreeMap`

Same interfaces, different engine: a balanced tree instead of a hash table.
Operations are O(log n) rather than O(1) — and in exchange, iteration is **sorted
by key**, deterministically, every run.

That unlocks something a hash map fundamentally cannot do — **range queries**:

```rust
let bt: BTreeMap<i32, &str> = [(1,"a"), (5,"b"), (9,"c"), (14,"d")].into();
let hits: Vec<_> = bt.range(4..10).collect();
// [(5, "b"), (9, "c")]
```

A hash table scatters keys across buckets by hash, so "all keys between 4 and 10"
would mean scanning everything. A tree keeps them adjacent, so it's a walk.

Note that sorted order is **not** insertion order. If you specifically need
*insertion*-ordered unique items, neither std type does it — reach for the
`indexmap` crate (`IndexSet` / `IndexMap`), or keep a `Vec` for order alongside a
`HashSet` for O(1) membership.

### Worked example: events in a time window

```rust
let log: BTreeMap<u64, &str> = [
    (1000, "start"), (1500, "connect"), (2200, "query"),
    (3100, "error"), (4000, "retry"),
].into();

log.range(1400..3200);      // [(1500,"connect"), (2200,"query"), (3100,"error")]
log.first_key_value();      // Some((1000, "start"))
log.last_key_value();       // Some((4000, "retry"))
```

Timestamps, version numbers, price levels, score thresholds — any key with a
natural ordering where you ask "everything between X and Y" is a `BTreeMap`
signal. Doing this with a `HashMap` means scanning every entry.

### Common `BTreeMap` / `BTreeSet` methods

Each mirrors its hash counterpart: `BTreeMap` has `insert` / `get` / `remove` /
`contains_key` / `entry` / `retain`, and `BTreeSet` has `insert` / `contains` /
`retain` plus the same `union` / `intersection` / `difference` /
`symmetric_difference` set algebra. Two differences apply throughout: every
operation is O(log n) rather than O(1), and iteration is **sorted and
deterministic**. On top of that, both add ordering-only methods:

| Method | Does |
|---|---|
| `.range(a..b)` | all entries with keys in the range |
| `.range_mut(a..b)` | same, mutable values |
| `.first_key_value()` / `.last_key_value()` | smallest / largest entry (`first()` / `last()` on a set) |
| `.pop_first()` / `.pop_last()` | remove and return the extreme entry |
| `.split_off(&k)` | split into two maps at a key |

`pop_first` makes a `BTreeMap` usable as a priority queue when you also need
lookup by key — something `BinaryHeap` can't do.

## Choosing between them

| You need | Use |
|---|---|
| Fixed length, known at compile time | `[T; N]` |
| A growable ordered sequence | `Vec<T>` |
| To *accept* a sequence in a function | `&[T]` |
| Fast "have I seen this?" | `HashSet<T>` |
| Fast key → value lookup | `HashMap<K, V>` |
| Sorted iteration or range queries | `BTreeSet<T>` / `BTreeMap<K, V>` |
| Push/pop at both ends (a queue) | `VecDeque<T>` |
| Repeatedly extract the min/max | `BinaryHeap<T>` |
| Unique **and** insertion-ordered | `indexmap` crate, or `Vec` + `HashSet` together |

Handle sizes, measured — the stack cost before any heap allocation:

```text
[i32; 5]           = 20     (inline, no heap at all)
&[i32]             = 16     (ptr + len)
Vec<i32>           = 24     (ptr + len + capacity)
BTreeSet<i32>      = 24
VecDeque<i32>      = 32
HashSet<i32>       = 48
HashMap<i32,i32>   = 48
```

Two closing rules of thumb:

- **Don't reach for `HashSet` to dedupe a small list.** For a handful of
  elements, a linear scan over a `Vec` beats hashing — the O(1) only wins once
  `n` is big enough to pay back the hashing constant and the cache-unfriendly
  layout.
- **Don't reach for `Vec` when the size is a compile-time constant.** `[T; N]`
  puts the size in the type, costs no allocation, and makes a whole class of
  length bugs unrepresentable.
