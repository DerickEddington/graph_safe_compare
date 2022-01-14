use {
    cycle_deep_safe_compare::{
        Cmp,
        Node,
    },
    std::{
        cmp::Ordering,
        convert::identity,
    },
    tests_utils::node_types::lazy::{
        Datum,
        DatumAllocator,
        Id,
    },
};


#[derive(Clone, Debug)]
struct My(Datum);

impl Node for My
{
    type Cmp = Ordering;
    type Id = Id;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        self.0.inner().id.clone()
    }

    fn amount_edges(&self) -> Self::Index
    {
        match self.0.get_edges() {
            Some((_, _)) => 2,
            None => 0,
        }
    }

    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Self
    {
        match (self.0.get_edges(), index) {
            (Some((a, _)), 0) => My(a),
            (Some((_, b)), 1) => My(b),
            _ => panic!("invalid"),
        }
    }

    fn equiv_modulo_edges(
        &self,
        _other: &Self,
    ) -> Self::Cmp
    {
        Cmp::new_equiv()
    }
}


tests_utils::eq_variations_tests!(My, Datum, identity, DatumAllocator::new);
