use {
    cycle_deep_safe_compare::Node,
    std::any::Any,
    tests_utils::{
        node_types::dyn_pair::{
            Datum1,
            Datum2,
            DatumAllocator,
            DatumRef,
            DowncastDatum,
        },
        shapes::Leaf,
    },
};


#[derive(Clone, Debug)]
struct My(DatumRef);

impl Node for My
{
    type Cmp = bool;
    type Id = *const dyn Any;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        &*self.0.0.borrow()
    }

    fn amount_edges(&self) -> Self::Index
    {
        match self.0.downcast() {
            DowncastDatum::Datum1(rd1) => match *rd1 {
                Datum1::Empty => 0,
                Datum1::Double(_, _) => 2,
            },
            DowncastDatum::Datum2Int32(rd2) => match *rd2 {
                Datum2::Value(_) => 0,
                Datum2::Two(_, _) => 2,
                Datum2::Four(_, _, _, _) => 4,
            },
            DowncastDatum::Datum2Char(rd2) => match *rd2 {
                Datum2::Value(_) => 0,
                Datum2::Two(_, _) => 2,
                Datum2::Four(_, _, _, _) => 4,
            },
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self
    {
        match self.0.downcast() {
            DowncastDatum::Datum1(rd1) => match (idx, &*rd1) {
                (0, Datum1::Double(a, _)) => My(a.clone()),
                (1, Datum1::Double(_, b)) => My(b.clone()),
                _ => panic!("invalid"),
            },
            DowncastDatum::Datum2Int32(rd2) => match (idx, &*rd2) {
                (0, Datum2::Two(a, _)) => My(a.clone()),
                (1, Datum2::Two(_, b)) => My(b.clone()),
                (0, Datum2::Four(a, _, _, _)) => My(a.clone()),
                (1, Datum2::Four(_, b, _, _)) => My(b.clone()),
                (2, Datum2::Four(_, _, c, _)) => My(c.clone()),
                (3, Datum2::Four(_, _, _, d)) => My(d.clone()),
                _ => panic!("invalid"),
            },
            DowncastDatum::Datum2Char(rd2) => match (idx, &*rd2) {
                (0, Datum2::Two(a, _)) => My(a.clone()),
                (1, Datum2::Two(_, b)) => My(b.clone()),
                (0, Datum2::Four(a, _, _, _)) => My(a.clone()),
                (1, Datum2::Four(_, b, _, _)) => My(b.clone()),
                (2, Datum2::Four(_, _, c, _)) => My(c.clone()),
                (3, Datum2::Four(_, _, _, d)) => My(d.clone()),
                _ => panic!("invalid"),
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
