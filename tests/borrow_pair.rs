// For the expansion of `tests_utils::eq_variations_tests!` below.  It is ok for the `&Datum` to
// be passed pointlessly to the `drop` in the general `tests_utils::eq_case!` macro, because the
// `Datum` will instead be dropped along with their allocator.
#![allow(clippy::drop_copy)]

mod common
{
    pub mod borrow_pair;
}
use common::borrow_pair::*;


tests_utils::eq_variations_tests!(My<'l>, &'l Datum<'l>, &, DatumAllocator::new);
