use {
    graph_safe_compare::Node,
    std::{
        cell::Cell,
        rc::Rc,
    },
    tests_utils::{
        node_types::{
            lazy,
            lazy::Shape,
        },
        shapes::{
            Allocator,
            Leaf,
            Pair,
        },
    },
};


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Side
{
    Top,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Datum
{
    lazy_datum: lazy::Datum,
    side:       Cell<Option<Side>>,
}

impl Datum
{
    fn new_sides(
        ancestor: &Self,
        left: lazy::Datum,
        right: lazy::Datum,
    ) -> (Self, Self)
    {
        use Side::*;

        let (left_side, right_side) = match ancestor.side.get() {
            Some(Top) => (Some(Left), Some(Right)),
            side => (side, side),
        };
        let left = Self { lazy_datum: left, side: Cell::new(left_side) };
        let right = Self { lazy_datum: right, side: Cell::new(right_side) };
        (left, right)
    }
}

impl Leaf for Datum
{
    type Alloc = DatumAllocator;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        Self { lazy_datum: Leaf::new_in(&alloc.0), side: Cell::new(None) }
    }
}

impl Pair for Datum
{
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        Pair::set(&self.lazy_datum, a.lazy_datum, b.lazy_datum);

        if let Shape::Vee = self.lazy_datum.inner().shape {
            self.side.set(Some(Side::Top));
        }
    }

    fn take(&self) -> Option<(Self, Self)>
    {
        Pair::take(&self.lazy_datum).map(|(a, b)| Self::new_sides(self, a, b))
    }

    fn into_vee_tails_for_head(
        left_tail: Self,
        right_tail: Self,
        head: &Self,
    ) -> (Self, Self)
    {
        let (l, r) = lazy::Datum::into_vee_tails_for_head(
            left_tail.lazy_datum,
            right_tail.lazy_datum,
            &head.lazy_datum,
        );
        Self::new_sides(head, l, r)
    }

    fn needs_cycle_deep_safe_drop() -> bool
    {
        lazy::Datum::needs_cycle_deep_safe_drop()
    }
}


#[derive(Clone, Debug)]
struct My(Datum);

impl Node for My
{
    type Cmp = bool;
    type Id = lazy::Id;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        self.0.lazy_datum.inner().id.clone()
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Option<Self>
    {
        use {
            Shape::*,
            Side::*,
        };

        let side = self.0.side.get();
        let shape = self.0.lazy_datum.inner().shape;
        let edges = self.0.lazy_datum.get_edges();

        let (side, descendent) = match (side, shape, idx, edges) {
            (None, Leaf, 0, None) => return None,
            // The top has tails on both sides, so the order in which they're given doesn't matter
            // but it must be consistent.
            (Some(Top), Vee, 0, Some((a, _))) => (Left, a),
            (Some(Top), Vee, 1, Some((_, b))) => (Right, b),
            (Some(Top), Vee, 2, Some(_)) => return None,
            // The left side has a tail on the left, so give that last.
            (Some(Left), InvertedList, 0, Some((_, b))) => (Left, b),
            (Some(Left), InvertedList, 1, Some((a, _))) => (Left, a),
            (Some(Left), InvertedList, 2, Some(_)) => return None,
            (Some(Left), Leaf, 0, None) => return None,
            // The right side has a tail on the right, so give that last.
            (Some(Right), List, 0, Some((a, _))) => (Right, a),
            (Some(Right), List, 1, Some((_, b))) => (Right, b),
            (Some(Right), List, 2, Some(_)) => return None,
            (Some(Right), Leaf, 0, None) => return None,
            _ => unreachable!(),
        };
        Some(My(Datum { lazy_datum: descendent, side: Cell::new(Some(side)) }))
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> Self::Cmp
    {
        fn parts(datum: &Datum) -> (Option<Side>, Shape, usize)
        {
            let inner = datum.lazy_datum.inner();
            (datum.side.get(), inner.shape, inner.depth)
        }

        parts(&self.0) == parts(&other.0)
    }
}


#[derive(Default, Clone)]
pub struct DatumAllocator(Rc<lazy::DatumAllocator>);

impl Allocator<Datum> for DatumAllocator
{
    fn alloc(&self) -> Datum
    {
        Datum { lazy_datum: self.0.alloc(), side: Cell::new(None) }
    }
}

impl DatumAllocator
{
    fn new(_size: u32) -> Self
    {
        Self::default()
    }
}


fn list_node_count(len: u32) -> u32
{
    2 * len + 1
}


#[cfg(test)]
mod wide_safe
{
    use {
        super::*,
        std::convert::identity,
    };

    mod recursion_stack
    {
        tests_utils::eq_variation_mod_body!(
            graph_safe_compare::wide_safe::equiv,
            My,
            Datum,
            identity,
            DatumAllocator::new
        );

        #[test]
        fn vee()
        {
            let total_depth = tests_utils::sizes::long_list_length() / 2;
            let side_depth = total_depth.checked_sub(1);
            let side_node_count = side_depth.map(list_node_count);
            let total_node_count = 1 + 2 * side_node_count.unwrap_or(0);

            // Just to appease `cycle_deep_safe_drop`.  This is fine, since our shape is not
            // cyclic.
            let dummy_tail: Datum = Leaf::new();

            tests_utils::eq_case!(
                identity,
                DatumAllocator::new,
                total_node_count,
                total_depth,
                vee,
                MyEq::new,
                (|(head, (_left_tail, _right_tail))| (head, dummy_tail.clone()))
            );
        }
    }
}
