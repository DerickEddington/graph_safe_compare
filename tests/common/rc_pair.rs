use cycle_deep_safe_compare::utils::RefId;
pub use {
    cycle_deep_safe_compare::Node,
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
    ) -> Self
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
    ) -> Self::Cmp
    {
        Ordering::Equal
    }
}
