use {
    cycle_deep_safe_compare::alt::basic::{
        precheck_interleave_equiv,
        Node,
    },
    std::rc::Rc,
    tests_utils::{
        node_types::rc_pair::{
            Datum,
            DatumAllocator,
        },
        shapes::Leaf,
    },
};


/// New type needed so we can impl the `Node` and `PartialEq` traits on it.
#[derive(Debug)]
struct My(Rc<Datum>);

impl PartialEq for My
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        precheck_interleave_equiv(self, other)
    }
}

impl Node for My
{
    type Edge = Self;
    type Id = *const Datum;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        &*self.0
    }

    fn amount_edges(&self) -> Self::Index
    {
        match &*self.0.0.borrow() {
            None => 0,
            Some((_, _)) => 2,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self::Edge
    {
        match (idx, &*self.0.0.borrow()) {
            (0, Some((a, _))) => My(a.clone()),
            (1, Some((_, b))) => My(b.clone()),
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
    let leaf1 = Leaf::new_in(&DatumAllocator);
    let leaf2 = Leaf::new_in(&DatumAllocator);
    assert_eq!(My(leaf1), My(leaf2));
}


use std::convert::identity;

tests_utils::eq_tests!(identity, DatumAllocator::new, My);
