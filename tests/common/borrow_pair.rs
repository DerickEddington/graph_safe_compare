use {
    cycle_deep_safe_compare::utils::RefId,
    tests_utils::node_types::borrow_pair::Inner,
};
pub use {
    cycle_deep_safe_compare::Node,
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
