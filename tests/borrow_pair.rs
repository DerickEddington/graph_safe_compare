use {
    cycle_deep_safe_compare::alt::basic::{
        precheck_interleave_equiv,
        Node,
    },
    tests_utils::{
        node_types::borrow_pair::{
            Datum,
            DatumAllocator,
            Inner,
        },
        shapes::Leaf,
    },
};


/// New type needed so we can impl the `Node` and `PartialEq` traits on it.
#[derive(Debug)]
struct My<'l>(&'l Datum<'l>);

impl<'l> PartialEq for My<'l>
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        precheck_interleave_equiv(self, other)
    }
}

impl<'l> Node for My<'l>
{
    type Edge = Self;
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
    ) -> Self::Edge
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


#[test]
fn rudimentary()
{
    let alloc = DatumAllocator::new(2);
    let leaf1 = Leaf::new_in(&&alloc);
    let leaf2 = Leaf::new_in(&&alloc);
    assert_eq!(My(leaf1), My(leaf2));
}


tests_utils::eq_tests!(&, DatumAllocator::new, My);
