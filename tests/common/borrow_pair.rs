use {
    graph_safe_compare::utils::RefId,
    tests_utils::node_types::borrow_pair::Inner,
};
pub use {
    graph_safe_compare::Node,
    tests_utils::node_types::borrow_pair::{
        Datum,
        DatumAllocator,
    },
};


#[derive(Copy, Clone, Debug)]
pub struct My<'l>(pub &'l Datum<'l>);

impl<'l> Node for My<'l>
{
    type Cmp = bool;
    type Id = RefId<&'l Datum<'l>>;
    type Index = u32;

    fn id(&self) -> Self::Id
    {
        RefId(self.0)
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Option<Self>
    {
        match (idx, self.0.0.get()) {
            (0, Inner::Pair(a, _)) => Some(My(a)),
            (1, Inner::Pair(_, b)) => Some(My(b)),
            _ => None,
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
