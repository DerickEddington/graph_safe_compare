// For the expansion of `tests_utils::eq_variations_tests!` below.  It is ok for the `&Datum` to
// be passed pointlessly to the `drop` in the general `tests_utils::eq_case!` macro, because the
// `Datum` will instead be dropped along with their allocator.
#![allow(clippy::drop_copy)]

use {
    cycle_deep_safe_compare::Node,
    tests_utils::{
        node_types::borrow_pair::{
            Datum,
            DatumAllocator,
            Inner,
        },
        shapes::Leaf,
    },
};


#[derive(Debug)]
struct My<'l>(&'l Datum<'l>);

impl<'l> Node for My<'l>
{
    type Cmp = bool;
    type Id = *const Datum<'l>;
    type Index = u32;

    fn id(&self) -> Self::Id
    {
        self.0
    }

    fn amount_edges(&self) -> Self::Index
    {
        match &*self.0.0.borrow() {
            Inner::Leaf => 0,
            Inner::Pair(_, _) => 2,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self
    {
        match (idx, &*self.0.0.borrow()) {
            (0, Inner::Pair(a, _)) => My(a),
            (1, Inner::Pair(_, b)) => My(b),
            _ => panic!("invalid"),
        }
    }

    fn equiv_modulo_edges(
        &self,
        _other: &Self,
    ) -> bool
    {
        true
    }
}


tests_utils::eq_variations_tests!(My<'l>, &'l Datum<'l>, &, DatumAllocator::new);
