//! Things useful to both unit tests and integration tests.
//! TODO: And benchmarks?

pub mod node_types
{
    pub mod borrow_pair;
    pub mod rc_pair;
}

pub mod shapes;


pub const LONG_LIST_TEST_LENGTH: usize =
    if cfg!(debug_assertions) { 1_000_000 } else { 10_000_000 };
pub const DEGENERATE_TEST_DEPTH: usize = if cfg!(debug_assertions) { 28 } else { 33 };
