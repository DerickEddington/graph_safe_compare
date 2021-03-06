use {
    graph_safe_compare::{
        utils::RefId,
        Node,
    },
    std::{
        any::Any,
        cell::RefCell,
        rc::Rc,
    },
    tests_utils::node_types::dyn_pair::{
        Datum1,
        Datum2,
        DatumAllocator,
        DatumRef,
        DowncastDatum,
    },
};


#[derive(Clone, Debug)]
struct My(DatumRef);

impl Node for My
{
    type Cmp = bool;
    type Id = RefId<Rc<RefCell<dyn Any>>>;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        RefId(Rc::clone(&self.0.0))
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Option<Self>
    {
        match self.0.downcast() {
            DowncastDatum::Datum1(rd1) => match (idx, &*rd1) {
                (0, Datum1::Double(a, _)) => Some(My(a.clone())),
                (1, Datum1::Double(_, b)) => Some(My(b.clone())),
                _ => None,
            },
            DowncastDatum::Datum2Int32(rd2) => match (idx, &*rd2) {
                (0, Datum2::Two(a, _)) => Some(My(a.clone())),
                (1, Datum2::Two(_, b)) => Some(My(b.clone())),
                (0, Datum2::Four(a, _, _, _)) => Some(My(a.clone())),
                (1, Datum2::Four(_, b, _, _)) => Some(My(b.clone())),
                (2, Datum2::Four(_, _, c, _)) => Some(My(c.clone())),
                (3, Datum2::Four(_, _, _, d)) => Some(My(d.clone())),
                _ => None,
            },
            DowncastDatum::Datum2Char(rd2) => match (idx, &*rd2) {
                (0, Datum2::Two(a, _)) => Some(My(a.clone())),
                (1, Datum2::Two(_, b)) => Some(My(b.clone())),
                (0, Datum2::Four(a, _, _, _)) => Some(My(a.clone())),
                (1, Datum2::Four(_, b, _, _)) => Some(My(b.clone())),
                (2, Datum2::Four(_, _, c, _)) => Some(My(c.clone())),
                (3, Datum2::Four(_, _, _, d)) => Some(My(d.clone())),
                _ => None,
            },
        }
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> bool
    {
        matches!(
            (self.0.downcast(), other.0.downcast()),
            (DowncastDatum::Datum1(_), DowncastDatum::Datum1(_))
                | (DowncastDatum::Datum2Int32(_), DowncastDatum::Datum2Int32(_))
                | (DowncastDatum::Datum2Char(_), DowncastDatum::Datum2Char(_))
        )
    }
}


use std::convert::identity;

tests_utils::eq_variations_tests!(My, DatumRef, identity, DatumAllocator::new);
