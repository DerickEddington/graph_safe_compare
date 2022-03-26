use graph_safe_compare::utils::RefId;
pub use {
    graph_safe_compare::Node,
    std::{
        cmp::Ordering,
        rc::Rc,
    },
    tests_utils::node_types::rc_pair::{
        Datum,
        DatumAllocator,
    },
};


#[derive(Clone, Debug)]
pub struct My(pub Rc<Datum>);

impl Node for My
{
    type Cmp = Ordering;
    type Id = RefId<Rc<Datum>>;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        RefId(Rc::clone(&self.0))
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Option<Self>
    {
        match (idx, &*self.0.0.borrow()) {
            (0, Some((a, _))) => Some(My(a.clone())),
            (1, Some((_, b))) => Some(My(b.clone())),
            _ => None,
        }
    }

    fn equiv_modulo_edges(
        &self,
        _other: &Self,
    ) -> Self::Cmp
    {
        Ordering::Equal
    }
}
