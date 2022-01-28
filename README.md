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

## Example

```rust
use graph_safe_compare::{robust::equiv, Node, utils::RefId};

#[derive(Eq)]
enum My {
    Leaf {
        val: i32,
    },
    Branch {
        left: Box<Self>,
        right: Box<Self>,
    },
}

impl PartialEq for My {
    fn eq(&self, other: &Self) -> bool {
        equiv(self, other)
    }
}

impl Node for &My {
    type Cmp = bool;
    type Id = RefId<Self>;
    type Index = usize;

    fn id(&self) -> Self::Id {
        RefId(*self)
    }

    fn amount_edges(&self) -> Self::Index {
        match self {
            My::Leaf { .. } => 0,
            My::Branch { .. } => 2,
        }
    }

    fn get_edge(&self, index: &Self::Index) -> Self {
        match (self, index) {
            (My::Branch { left, .. }, 0) => left,
            (My::Branch { right, .. }, 1) => right,
            _ => unreachable!(),
        }
    }

    fn equiv_modulo_edges(&self, other: &Self) -> Self::Cmp {
        match (self, other) {
            (My::Leaf { val: v1 }, My::Leaf { val: v2 }) => v1 == v2,
            (My::Branch { .. }, My::Branch { .. }) => true,
            _ => false,
        }
    }
}

let a = Box::new(My::Branch {
    left: Box::new(My::Leaf { val: 1 }),
    right: Box::new(My::Leaf { val: 2 }),
});
let b = Box::new(My::Branch {
    left: Box::new(My::Leaf { val: 1 }),
    right: Box::new(My::Leaf { val: 2 }),
});
assert!(a == b);
```

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
