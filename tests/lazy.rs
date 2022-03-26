use {
    graph_safe_compare::{
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

    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Option<Self>
    {
        match (self.0.get_edges(), index) {
            (Some((a, _)), 0) => Some(My(a)),
            (Some((_, b)), 1) => Some(My(b)),
            _ => None,
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
