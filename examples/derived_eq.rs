//! Demonstrate that the usual derived PartialEq algorithm cannot handle cyclic graphs nor
//! very-deep DAGs and that it handles large amounts of shared structure inefficiently.

#![cfg(test)]

use std::convert::identity;
use tests_utils::node_types::rc_pair::DatumAllocator;

tests_utils::eq_shapes_tests!(identity, DatumAllocator::new, identity,
                              #[ignore], #[ignore]);
