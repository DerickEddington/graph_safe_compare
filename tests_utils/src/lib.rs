//! Things useful to both unit tests and integration tests.
//! TODO: And benchmarks?

pub mod cases
{
    use std::mem::ManuallyDrop;

    /// Because dropping deep graphs can cause stack overflows, disable dropping for select test
    /// data so we know that any stack overflows that happen were not caused by dropping.  This
    /// does leak the memory of these test data, which is acceptable.
    #[derive(PartialEq, Eq, Debug)]
    pub struct NoDrop<T>(pub ManuallyDrop<T>);

    pub mod eq;
}

pub mod node_types
{
    pub mod borrow_pair;
    pub mod rc_pair;
    pub mod dyn_pair;
    pub mod diff_edge;
    pub mod diff_index;
}

pub mod shapes;


pub const LONG_LIST_TEST_LENGTH: u32 = if cfg!(debug_assertions) { 1_000_000 } else { 10_000_000 };
pub const DEGENERATE_TEST_DEPTH: u32 = if cfg!(debug_assertions) { 28 } else { 33 };
