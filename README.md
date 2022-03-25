# graph_safe_compare

Equivalence predicate that can handle cyclic, shared, and very-deep graphs.
Implements the algorithm described in the paper [Efficient Nondestructive
Equality Checking for Trees and
Graphs](https://michaeldadams.org/papers/efficient_equality/).  Has enhancements
to support recursion without using the call-stack to support graphs with great
depth and to support multi-way comparison to support giving an ordering to
graphs.

## Motivation

With Rust, it is common to `#[derive(PartialEq)]` for types so that values can
be compared.  However, such derived implementations cannot handle cyclic nor
very-deep inputs and will cause stack overflows when given them, and will
execute inefficiently when given inputs that have much shared structure.

This crate provides functions that are safe and efficient for general shapes of
graphs, that can be used as `PartialEq` `impl`ementations.

## Examples

<details><summary>Degenerate D.A.G. Shape</summary>

A chain where each pair of `left` and `right` edges of a `My::Branch` reference
the same next `Rc<My>` node.  Without shared-structure detection, it would be
traversed like a perfect binary tree with `2^(depth+1)-2` recursions, but with
the shared-structure detection of this crate, it is traversed with only
`2*depth` recursions.

```rust
use graph_safe_compare::{robust, utils::RefId, Node};
use std::rc::Rc;
use My::*;

#[derive(Eq)]
enum My {
    Leaf { val: i32 },
    Branch { left: Rc<Self>, right: Rc<Self> },
}

impl My {
    fn new_degenerate_shared_structure(depth: usize) -> Self {
        let next = Leaf { val: 1 };
        (0..depth).fold(next, |next, _| {
            let next = Rc::new(next);
            Branch { left: Rc::clone(&next), right: next }
        })
    }
}

impl PartialEq for My {
    fn eq(&self, other: &Self) -> bool { robust::equiv(self, other) }
}

impl Node for &My {
    type Cmp = bool;
    type Id = RefId<Self>;
    type Index = usize;

    fn id(&self) -> Self::Id { RefId(*self) }

    fn amount_edges(&self) -> Self::Index {
        match self {
            Leaf { .. } => 0,
            Branch { .. } => 2,
        }
    }

    fn get_edge(&self, index: &Self::Index) -> Self {
        match (self, index) {
            (Branch { left, .. }, 0) => left,
            (Branch { right, .. }, 1) => right,
            _ => unreachable!(),
        }
    }

    fn equiv_modulo_edges(&self, other: &Self) -> Self::Cmp {
        match (self, other) {
            (Leaf { val: v1 }, Leaf { val: v2 }) => v1 == v2,
            (Branch { .. }, Branch { .. }) => true,
            _ => false,
        }
    }
}

fn main() {
    // A depth that is fast with the `robust` variant of this crate, but that
    // would be infeasible and either take forever, due to the great degree of
    // shared structure, or cause stack overflow, due to the great depth, if
    // another variant were used.
    let depth = 1_000_000;
    let a = My::new_degenerate_shared_structure(depth);
    let b = My::new_degenerate_shared_structure(depth);
    assert!(a == b);

    // Prevent running the drop destructor, to avoid the stack overflow it would
    // cause due to the great depth.  (A real implementation would need a `Drop`
    // designed to properly avoid that.)
    std::mem::forget((a, b));
}
```
</details>

<details><summary>Cyclic Shape</summary>

A very-simple cycle.  Without shared-structure detection, it would infinitely
recurse and overflow the stack, but with the shared-structure detection of this
crate, it does not and it completes efficiently.

The types involved are more complicated, to be able to construct cycles.

```rust
use graph_safe_compare::{cycle_safe, utils::RefId, Node};
use std::{cell::{Ref, RefCell}, rc::Rc};
use Inner::*;

#[derive(Clone)]
struct My(Rc<RefCell<Inner>>);

enum Inner {
    Leaf { val: i32 },
    Branch { left: My, right: My },
}

impl My {
    fn leaf(val: i32) -> Self { My(Rc::new(RefCell::new(Leaf { val }))) }

    fn set_branch(&self, left: My, right: My) {
        *self.0.borrow_mut() = Branch { left, right };
    }

    fn new_cyclic_structure() -> Self {
        let cyc = My::leaf(0);
        cyc.set_branch(My::leaf(1), cyc.clone());
        cyc
    }

    fn inner(&self) -> Ref<'_, Inner> { self.0.borrow() }
}

impl PartialEq for My {
    fn eq(&self, other: &Self) -> bool {
        cycle_safe::equiv(self.clone(), other.clone())
    }
}
impl Eq for My {}

impl Node for My {
    type Cmp = bool;
    type Id = RefId<Rc<RefCell<Inner>>>;
    type Index = u32;

    fn id(&self) -> Self::Id { RefId(Rc::clone(&self.0)) }

    fn amount_edges(&self) -> Self::Index {
        match &*self.inner() {
            Leaf { .. } => 0,
            Branch { .. } => 2,
        }
    }

    fn get_edge(&self, index: &Self::Index) -> Self {
        match (index, &*self.inner()) {
            (0, Branch { left, .. }) => left.clone(),
            (1, Branch { right, .. }) => right.clone(),
            _ => unreachable!(),
        }
    }

    fn equiv_modulo_edges(&self, other: &Self) -> Self::Cmp {
        match (&*self.inner(), &*other.inner()) {
            (Leaf { val: v1 }, Leaf { val: v2 }) => v1 == v2,
            (Branch { .. }, Branch { .. }) => true,
            _ => false,
        }
    }
}

fn main() {
    let a = My::new_cyclic_structure();
    let b = My::new_cyclic_structure();
    assert!(a == b);

    // (A real implementation would need to break the cycles, to allow them to
    // be dropped.)
}
````
</details>

<details><summary>Multi-way Comparison for Ordering</summary>

```rust
use graph_safe_compare::{basic, utils::RefId, Node};
use std::cmp::Ordering;

#[derive(Eq)]
struct My(Vec<i32>);

impl Ord for My {
    fn cmp(&self, other: &Self) -> Ordering { basic::equiv(self, other) }
}
impl PartialOrd for My {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for My {
    fn eq(&self, other: &Self) -> bool { self.cmp(other).is_eq() }
}

impl Node for &My {
    type Cmp = Ordering;
    type Id = RefId<Self>;
    type Index = u8;

    fn id(&self) -> Self::Id { RefId(*self) }

    fn amount_edges(&self) -> Self::Index { 0 }

    fn get_edge(&self, _: &Self::Index) -> Self { unreachable!() }

    fn equiv_modulo_edges(&self, other: &Self) -> Self::Cmp {
        self.0.iter().cmp(other.0.iter())
    }
}

fn main() {
    let mut array = [My(vec![1, 2, 3]), My(vec![3]), My(vec![1, 2])];
    array.sort();
    assert!(array == [My(vec![1, 2]), My(vec![1, 2, 3]), My(vec![3])])
}
````
</details>

## Design

- No `unsafe` code.

- No `panic`s.

- Very minimal dependencies.

- Organized into modules that provide variations of the algorithm for different
possible shapes of graphs.  Applications with graph shapes that are limited can
benefit from using a variation that only supports what is needed and avoids the
overhead that other variations involve.  E.g. when only shallow cyclic shapes
are possible, the functions provided by the `cycle_safe` module are sufficient,
or e.g. when only acyclic deep shapes are possible, the `deep_safe` module is
sufficient, or e.g. when deep cyclic shapes are possible then the `robust`
module can be used.

- A `generic` module exposes the generic API (which the other modules build on)
that enables customizing the parameters (both types and constants) of the
algorithm to make custom variations.

- The generic API supports fallible `Result`s with custom error types, which can
be used to achieve custom limiting, e.g. of memory-usage or execution-time.

### `no_std` support

While the support for cyclic and deep graphs requires dynamic memory allocations
internally, this can be provided without the `std` or `alloc` crates.  The
generic API of this crate is designed for custom provision of the needed dynamic
data structures.  When built without its `"std"` feature, this crate is
`no_std`.

## Documentation

The source-code has many doc comments, which are rendered as the API
documentation.

View online at: <http://docs.rs/graph_safe_compare>

Or, you can generate them yourself and view locally by doing:

```shell
cargo doc --open
```

## Tests

There are unit tests and integration tests, which can be run by doing:

```shell
cargo test --workspace
```

The `ignored` tests can be run to demonstrate the limitations of variations that
do not support some shapes, and are expected to either cause stack overflow
crashes or to take a very long time.

There is a package that tests using the crate as `no_std`, which can be run by
doing:

```shell
cd test_no_std
cargo build --features graph_safe_compare/wyrng
```

## Benchmarks

There are benchmarks of the variations, that use a node type with very little
overhead, which can be run by doing:

```shell
cargo +nightly bench --profile bench-max-optim
```
