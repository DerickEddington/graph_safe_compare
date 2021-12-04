//! Things useful to both unit tests and integration tests.
//! TODO: And benchmarks?

pub mod cases
{
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
