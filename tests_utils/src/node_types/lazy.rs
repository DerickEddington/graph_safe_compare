use {
    crate::shapes::{
        Allocator,
        Leaf,
        Pair,
    },
    std::{
        cell::{
            RefCell,
            RefMut,
        },
        hash::Hash,
        ops::Sub,
        ptr,
        rc::Rc,
    },
};


#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Shape
{
    Leaf,
    List,
    InvertedList,
    Vee,
    DegenerateDAG,
    DegenerateCyclic,
}


#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Id
{
    pub alloc: Rc<DatumAllocator>,
    pub gen:   usize,
    pub num:   usize,
}

impl Sub<usize> for &Id
{
    type Output = Id;

    fn sub(
        self,
        rhs: usize,
    ) -> Self::Output
    {
        Id { num: self.num - rhs, ..self.clone() }
    }
}


#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Inner
{
    pub id:    Id,
    pub shape: Shape,
    pub depth: usize,
}

impl Inner
{
    fn get_edges(&self) -> Option<(Self, Self)>
    {
        use Shape::*;

        (self.depth >= 1).then(|| {
            let (left, right);
            let depth = self.depth - 1;

            match self.shape {
                List => {
                    left = Inner { id: &self.id - 1, shape: Leaf, depth: 0 };
                    right = Inner {
                        id: &left.id - 1,
                        shape: if depth >= 1 { List } else { Leaf },
                        depth,
                    };
                },
                InvertedList => {
                    let amount_nodes = 2 * self.depth + 1;
                    debug_assert_eq!(self.id.num, amount_nodes - 1);

                    right = Inner { id: &self.id - 1, shape: Leaf, depth: 0 };
                    left = Inner {
                        id: &right.id - 1,
                        shape: if depth >= 1 { InvertedList } else { Leaf },
                        depth,
                    };
                },
                Vee => {
                    let side_amount_nodes = 2 * depth + 1;
                    let total_amount_nodes = 2 * side_amount_nodes + 1;

                    debug_assert_eq!(self.id.num, total_amount_nodes - 1);

                    right = Inner {
                        id: &self.id - 1,
                        shape: if depth >= 1 { List } else { Leaf },
                        depth,
                    };
                    left = Inner {
                        id: &right.id - side_amount_nodes,
                        shape: if depth >= 1 { InvertedList } else { Leaf },
                        depth,
                    };
                    debug_assert_eq!(left.id.num, side_amount_nodes - 1);
                },
                DegenerateDAG => {
                    let amount_nodes = self.depth + 1;
                    debug_assert_eq!(self.id.num, amount_nodes - 1);

                    left = Inner {
                        id: &self.id - 1,
                        shape: if depth >= 1 { DegenerateDAG } else { Leaf },
                        depth,
                    };
                    right = left.clone();
                },
                DegenerateCyclic => {
                    let id = if self.id.num == 0 {
                        Id { num: depth, ..self.id.clone() } // cycle
                    }
                    else {
                        &self.id - 1
                    };
                    left = Inner { id, shape: DegenerateCyclic, depth: self.depth };
                    right = left.clone();
                },
                Leaf => unreachable!(),
            }

            (left, right)
        })
    }

    pub fn tail(&self) -> Tail
    {
        macro_rules! descend {
            ($next_vertebrae:expr) => {{
                let next_vertebrae: &dyn Fn(&Self) -> Option<Self> = &($next_vertebrae);
                let mut spine = self.clone();
                while let Some(next) = next_vertebrae(&spine) {
                    spine = next;
                }
                spine
            }};
        }

        match self.shape {
            Shape::Leaf => Tail::Single(self.clone()),
            Shape::List => Tail::Single(descend!(|i| i.get_edges().map(|(_, n)| n))),
            Shape::InvertedList => Tail::Single(descend!(|i| i.get_edges().map(|(n, _)| n))),
            Shape::Vee => {
                let (left, right) = self.get_edges().expect("not pair");
                if let (Tail::Single(left), Tail::Single(right)) = (left.tail(), right.tail()) {
                    Tail::Double(left, right)
                }
                else {
                    unreachable!()
                }
            },
            Shape::DegenerateDAG => Tail::Single(descend!(|i| {
                i.get_edges().map(|(l, r)| {
                    debug_assert_eq!(l, r);
                    l
                })
            })),
            Shape::DegenerateCyclic => Tail::Single(descend!(|i| {
                i.get_edges().and_then(|(l, r)| {
                    debug_assert_eq!(l, r);
                    (i.id.num >= 1).then(|| l)
                })
            })),
        }
    }
}


#[derive(Debug)]
pub enum Tail
{
    Single(Inner),
    Double(Inner, Inner),
}

impl PartialEq<Inner> for Tail
{
    fn eq(
        &self,
        other: &Inner,
    ) -> bool
    {
        match self {
            Tail::Single(single) => single == other,
            Tail::Double(left, right) => left == other && right == other,
        }
    }
}


/// Generates edges lazily and uses little memory itself.
#[derive(Eq, Clone, Debug)]
pub struct Datum(RefCell<Inner>);

impl Datum
{
    #[inline]
    pub fn inner(&self) -> RefMut<'_, Inner>
    {
        self.0.borrow_mut()
    }

    #[inline]
    pub fn get_edges(&self) -> Option<(Self, Self)>
    {
        self.inner()
            .get_edges()
            .map(|(left, right)| (Self(RefCell::new(left)), Self(RefCell::new(right))))
    }
}

/// This `PartialEq` does not implement a `graph_safe_compare` algorithm and is only used for
/// having an intentionally-naive algorithm that acts as if the shapes exist.  When
/// `graph_safe_compare` algorithms are tested against this type, their functions must be
/// called directly.
impl PartialEq for Datum
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        let self_shape = self.inner().shape; // (Drop temporary.)
        let other_shape = other.inner().shape; // (Drop temporary.)
        self_shape == other_shape
            && match (self.get_edges(), other.get_edges()) {
                (Some(self_edges), Some(other_edges)) =>
                    self_edges.0 == other_edges.0 && self_edges.1 == other_edges.1,
                (None, None) => true,
                _ => false,
            }
    }
}


impl Pair for Datum
{
    /// Must only be used by `PairChainMaker`.
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        use Shape::*;

        let mut inner = self.inner();
        let alloc = Rc::clone(&inner.id.alloc);
        let (a, b) = (a.0.into_inner(), b.0.into_inner());

        debug_assert_eq!(inner.shape, Leaf);
        debug_assert_eq!(inner.depth, 0);
        debug_assert_eq!(alloc, a.id.alloc);
        debug_assert_eq!(a.id.alloc, b.id.alloc);

        let new = match (a.shape, b.shape) {
            (Leaf, Leaf) => {
                debug_assert_eq!(a.id.num, 0);
                debug_assert_eq!(a.depth, 0);
                debug_assert_eq!(b.depth, 0);

                if a.id == b.id {
                    if inner.id == a.id {
                        Inner { id: inner.id.clone(), shape: DegenerateCyclic, depth: 1 }
                    }
                    else {
                        Inner {
                            id:    Id { alloc, gen: a.id.gen, num: 1 },
                            shape: DegenerateDAG,
                            depth: 1,
                        }
                    }
                }
                else if a.id.gen > b.id.gen {
                    Inner { id: Id { alloc, gen: b.id.gen, num: 2 }, shape: List, depth: 1 }
                }
                else if a.id.gen < b.id.gen {
                    Inner {
                        id:    Id { alloc, gen: a.id.gen, num: 2 },
                        shape: InvertedList,
                        depth: 1,
                    }
                }
                else if inner.id.gen == a.id.gen && inner.id.gen == b.id.gen {
                    let side_amount_nodes = 1;
                    let total_amount_nodes = 2 * side_amount_nodes + 1;
                    Inner {
                        id:    Id { alloc, gen: inner.id.gen, num: total_amount_nodes - 1 },
                        shape: Vee,
                        depth: 1,
                    }
                }
                else {
                    unreachable!()
                }
            },
            (Leaf, List) => {
                debug_assert_eq!(a.id.num, 0);
                debug_assert_eq!(a.depth, 0);
                debug_assert!(a.id.gen > b.id.gen);
                Inner {
                    id:    Id { alloc, gen: b.id.gen, num: b.id.num + 2 },
                    shape: List,
                    depth: b.depth + 1,
                }
            },
            (InvertedList, Leaf) => {
                debug_assert_eq!(b.id.num, 0);
                debug_assert_eq!(b.depth, 0);
                debug_assert!(a.id.gen < b.id.gen);
                Inner {
                    id:    Id { alloc, gen: a.id.gen, num: a.id.num + 2 },
                    shape: InvertedList,
                    depth: a.depth + 1,
                }
            },
            (InvertedList, List) => {
                debug_assert_eq!(a.depth, b.depth);
                debug_assert!(a.id.gen < b.id.gen);
                debug_assert_eq!(a.id.num, 2 * a.depth);
                debug_assert_eq!(b.id.num, 2 * b.depth);

                let side_amount_nodes = 2 * a.depth + 1;
                let total_amount_nodes = 2 * side_amount_nodes + 1;
                let num = total_amount_nodes - 1; // zero based
                Inner { id: Id { alloc, gen: a.id.gen, num }, shape: Vee, depth: a.depth + 1 }
            },
            (DegenerateDAG, DegenerateDAG) => {
                debug_assert_eq!(a, b);

                if inner.id.gen == a.id.gen {
                    Inner {
                        id:    Id { alloc, gen: inner.id.gen, num: 0 },
                        shape: DegenerateCyclic,
                        depth: a.depth + 1,
                    }
                }
                else {
                    Inner {
                        id:    Id { alloc, gen: a.id.gen, num: a.id.num + 1 },
                        shape: DegenerateDAG,
                        depth: a.depth + 1,
                    }
                }
            },
            (DegenerateCyclic, DegenerateCyclic) => {
                debug_assert_eq!(a, b);

                Inner {
                    id:    Id { alloc: a.id.alloc, gen: a.id.gen, num: 0 },
                    shape: DegenerateCyclic,
                    depth: a.depth,
                }
            },
            _ => unreachable!(),
        };
        *inner = new;
    }

    fn take(&self) -> Option<(Self, Self)>
    {
        let result = self.get_edges();
        let new = self.inner().clone();
        *self.inner() = Inner { shape: Shape::Leaf, depth: 0, ..new }; // Keep same ID.
        result
    }

    /// Must only be used by `PairChainMaker::vee`.
    fn into_vee_tails_for_head(
        left_tail: Self,
        right_tail: Self,
        head: &Self,
    ) -> (Self, Self)
    {
        let head = head.inner();
        debug_assert_eq!(head.shape, Shape::Vee);
        debug_assert!(head.depth >= 1);
        debug_assert!(head.id.num >= 1);

        macro_rules! assert_tail {
            ($tail:expr) => {{
                let tail = $tail;
                debug_assert_eq!(tail.shape, Shape::Leaf);
                debug_assert_eq!(tail.id.num, 0);
                debug_assert_eq!(tail.id.alloc, head.id.alloc);
                debug_assert!(tail.id.num < head.id.num);
                tail
            }};
        }

        if let Shape::Vee = head.shape {
            let mut left_tail = assert_tail!(left_tail.inner());
            let mut right_tail = assert_tail!(right_tail.inner());
            debug_assert!(left_tail.id.gen < right_tail.id.gen);

            let side_depth = head.depth - 1;
            let side_amount_nodes = 2 * side_depth + 1;
            right_tail.id = Id { num: side_amount_nodes, ..head.id.clone() };
            left_tail.id = Id { num: 0, ..head.id.clone() };
        }
        else {
            unreachable!();
        }

        (left_tail, right_tail)
    }

    fn needs_cycle_deep_safe_drop() -> bool
    {
        false
    }
}

impl Leaf for Datum
{
    type Alloc = Rc<DatumAllocator>;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        let mut next_gen = alloc.next_gen.borrow_mut();
        let gen = *next_gen;
        *next_gen += 1;
        Self(RefCell::new(Inner {
            id:    Id { alloc: Rc::clone(alloc), gen, num: 0 },
            shape: Shape::Leaf,
            depth: 0,
        }))
    }
}


#[derive(Default, Eq, Debug)]
pub struct DatumAllocator
{
    next_gen: RefCell<usize>,
}

impl DatumAllocator
{
    pub fn new(_size: u32) -> Rc<Self>
    {
        Rc::new(Self::default())
    }
}

impl Allocator<Datum> for Rc<DatumAllocator>
{
    fn alloc(&self) -> Datum
    {
        Leaf::new_in(self)
    }
}

impl PartialEq for DatumAllocator
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        ptr::eq(self, other)
    }
}

impl Hash for DatumAllocator
{
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    )
    {
        ptr::hash(self, state)
    }
}


#[cfg(test)]
mod tests
{
    use {
        super::*,
        crate::{
            shapes::PairChainMaker,
            sizes,
        },
        std::cmp::max,
        Shape::*,
    };

    // #[test]
    // fn size()
    // {
    //     dbg!(std::mem::size_of::<Datum>());
    // }

    #[rustfmt::skip::macros(case, id)]
    #[allow(clippy::redundant_clone)]
    mod make
    {
        use {
            super::*,
            to_inner::*,
        };

        mod to_inner
        {
            use super::*;

            pub(super) trait ToInner
            {
                type Inner;
                fn to_inner(&self) -> Self::Inner;
            }

            impl ToInner for Datum
            {
                type Inner = Inner;

                fn to_inner(&self) -> Inner
                {
                    self.inner().clone()
                }
            }

            impl ToInner for (Datum, Datum)
            {
                type Inner = InnerPair;

                fn to_inner(&self) -> InnerPair
                {
                    InnerPair(self.0.to_inner(), self.1.to_inner())
                }
            }

            #[derive(PartialEq, Eq, Debug)]
            pub(super) struct InnerPair(pub(super) Inner, pub(super) Inner);

            impl PartialEq<InnerPair> for Inner
            {
                fn eq(
                    &self,
                    other: &InnerPair,
                ) -> bool
                {
                    *self == other.0 && *self == other.1
                }
            }

            impl PartialEq<InnerPair> for Tail
            {
                fn eq(
                    &self,
                    other: &InnerPair,
                ) -> bool
                {
                    match self {
                        Tail::Single(single) => single == &other.0 && single == &other.1,
                        Tail::Double(left, right) => left == &other.0 && right == &other.1,
                    }
                }
            }
        }


        macro_rules! case_with {
            (
                $alloc:expr, $depth:expr,degenerate_cyclic, $expect_head:expr, $expect_tail:expr
            ) => {
                case_with!($alloc, $depth, degenerate_cyclic, 1, $expect_head, $expect_tail);
            };
            ($alloc:expr, $depth:expr, $shape:ident, $expect_head:expr, $expect_tail:expr) => {
                case_with!($alloc, $depth, $shape, 0, $expect_head, $expect_tail);
            };
            (
                $alloc:expr,
                $depth:expr,
                $shape:ident,
                $same_at_depth:expr,
                $expect_head:expr,
                $expect_tail:expr
            ) => {{
                let alloc = Rc::clone(&$alloc);
                let depth = $depth;
                let same_at_depth = $same_at_depth;

                let (head, tail) = PairChainMaker::new_with(depth, alloc).$shape();
                let (head, tail) = (head.to_inner(), tail.to_inner());

                if depth > same_at_depth {
                    assert_ne!(head, tail);
                }
                else {
                    assert_eq!(head, tail);
                }
                assert_eq!(head, $expect_head);
                assert_eq!(tail, $expect_tail);
                assert_eq!(head.tail(), tail);
            }};
        }

        #[test]
        fn depth0()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            macro_rules! case {
                ($shape:ident, $expect_head:expr, $expect_tail:expr) => {
                    case_with!(alloc, 0, $shape, $expect_head, $expect_tail);
                };
            }
            macro_rules! id {
                ($gen:expr, $num:expr) => {
                    Id { gen: $gen, num: $num, alloc: Rc::clone(&alloc) }
                };
            }

            case!(
                list,
                Inner { id: id!(0, 0), shape: Leaf, depth: 0 },
                Inner { id: id!(0, 0), shape: Leaf, depth: 0 }
            );
            case!(
                inverted_list,
                Inner { id: id!(1, 0), shape: Leaf, depth: 0 },
                Inner { id: id!(1, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_dag,
                Inner { id: id!(2, 0), shape: Leaf, depth: 0 },
                Inner { id: id!(2, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_cyclic,
                Inner { id: id!(3, 0), shape: Leaf, depth: 0 },
                Inner { id: id!(3, 0), shape: Leaf, depth: 0 }
            );
            case!(
                vee,
                Inner { id: id!(4, 0), shape: Leaf, depth: 0 },
                InnerPair(Inner { id: id!(4, 0), shape: Leaf, depth: 0 },
                          Inner { id: id!(4, 0), shape: Leaf, depth: 0 })
            );
        }

        #[test]
        fn depth1()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            macro_rules! case {
                ($shape:ident, $expect_head:expr, $expect_tail:expr) => {
                    case_with!(alloc, 1, $shape, $expect_head, $expect_tail);
                };
            }
            macro_rules! id {
                ($gen:expr, $num:expr) => {
                    Id { gen: $gen, num: $num, alloc: Rc::clone(&alloc) }
                };
            }

            case!(
                list,
                Inner { id: id!(0, 2), shape: List, depth: 1 },
                Inner { id: id!(0, 0), shape: Leaf, depth: 0 }
            );
            case!(
                inverted_list,
                Inner { id: id!(3, 2), shape: InvertedList, depth: 1 },
                Inner { id: id!(3, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_dag,
                Inner { id: id!(6, 1), shape: DegenerateDAG, depth: 1 },
                Inner { id: id!(6, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_cyclic,
                Inner { id: id!(8, 0), shape: DegenerateCyclic, depth: 1 },
                Inner { id: id!(8, 0), shape: DegenerateCyclic, depth: 1 }
            );
            case!(
                vee,
                Inner { id: id!(9, 2), shape: Vee, depth: 1 },
                InnerPair(Inner { id: id!(9, 0), shape: Leaf, depth: 0 },
                          Inner { id: id!(9, 1), shape: Leaf, depth: 0 })
            );
        }

        #[test]
        fn depth2()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            macro_rules! case {
                ($shape:ident, $expect_head:expr, $expect_tail:expr) => {
                    case_with!(alloc, 2, $shape, $expect_head, $expect_tail);
                };
            }
            macro_rules! id {
                ($gen:expr, $num:expr) => {
                    Id { gen: $gen, num: $num, alloc: Rc::clone(&alloc) }
                };
            }

            case!(
                list,
                Inner { id: id!(0, 4), shape: List, depth: 2 },
                Inner { id: id!(0, 0), shape: Leaf, depth: 0 }
            );
            case!(
                inverted_list,
                Inner { id: id!(5, 4), shape: InvertedList, depth: 2 },
                Inner { id: id!(5, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_dag,
                Inner { id: id!(10, 2), shape: DegenerateDAG, depth: 2 },
                Inner { id: id!(10, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_cyclic,
                Inner { id: id!(13, 1), shape: DegenerateCyclic, depth: 2 },
                Inner { id: id!(13, 0), shape: DegenerateCyclic, depth: 2 }
            );
            case!(
                vee,
                Inner { id: id!(15, 6), shape: Vee, depth: 2 },
                InnerPair(Inner { id: id!(15, 0), shape: Leaf, depth: 0 },
                          Inner { id: id!(15, 3), shape: Leaf, depth: 0 })
            );
        }

        #[test]
        fn depth3()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            macro_rules! case {
                ($shape:ident, $expect_head:expr, $expect_tail:expr) => {
                    case_with!(alloc, 3, $shape, $expect_head, $expect_tail);
                };
            }
            macro_rules! id {
                ($gen:expr, $num:expr) => {
                    Id { gen: $gen, num: $num, alloc: Rc::clone(&alloc) }
                };
            }

            case!(
                list,
                Inner { id: id!(0, 6), shape: List, depth: 3 },
                Inner { id: id!(0, 0), shape: Leaf, depth: 0 }
            );
            case!(
                inverted_list,
                Inner { id: id!(7, 6), shape: InvertedList, depth: 3 },
                Inner { id: id!(7, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_dag,
                Inner { id: id!(14, 3), shape: DegenerateDAG, depth: 3 },
                Inner { id: id!(14, 0), shape: Leaf, depth: 0 }
            );
            case!(
                degenerate_cyclic,
                Inner { id: id!(18, 2), shape: DegenerateCyclic, depth: 3 },
                Inner { id: id!(18, 0), shape: DegenerateCyclic, depth: 3 }
            );
            case!(
                vee,
                Inner { id: id!(21, 10), shape: Vee, depth: 3 },
                InnerPair(Inner { id: id!(21, 0), shape: Leaf, depth: 0 },
                          Inner { id: id!(21, 5), shape: Leaf, depth: 0 })
            );
        }
    }

    mod derived_eq
    {
        #![allow(clippy::eq_op)]

        use super::*;

        macro_rules! make {
            ($shape:ident, $depth:expr) => {{
                let (head, _tail): (Datum, _) = PairChainMaker::new($depth).$shape();
                head
            }};
            ($alloc:expr, $shape:ident, $depth:expr) => {{
                let alloc = Rc::clone(&$alloc);
                let (head, _tail): (Datum, _) = PairChainMaker::new_with($depth, alloc).$shape();
                head
            }};
        }
        macro_rules! case {
            ($shape:ident, $depth:expr, $alloc:expr) => {{
                let depth = $depth;
                let alloc = Rc::clone(&$alloc);
                let (a, b) = (make!($shape, depth), make!($shape, depth));
                let (c, d) = (make!(alloc, $shape, depth), make!(alloc, $shape, depth));
                assert_eq!(a, a);
                assert_eq!(b, b);
                assert_eq!(a, b);
                assert_eq!(b, a);
                assert_eq!(c, c);
                assert_eq!(d, d);
                assert_eq!(c, d);
                assert_eq!(d, c);
                assert_eq!(a, c);
                assert_eq!(a, d);
                assert_eq!(b, c);
                assert_eq!(b, d);
                assert_eq!(c, a);
                assert_eq!(d, a);
                assert_eq!(c, b);
                assert_eq!(d, b);
            }};
        }

        fn len() -> u32
        {
            max(100, sizes::long_list_length() / 1000)
        }

        fn long_depth() -> u32
        {
            max(17, sizes::degenerate_depth().saturating_sub(5))
        }

        #[test]
        fn list()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            case!(list, 0, alloc);
            case!(list, 1, alloc);
            case!(list, 2, alloc);
            case!(list, 3, alloc);
            case!(list, len(), alloc);
        }

        #[test]
        fn inverted_list()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            case!(inverted_list, 0, alloc);
            case!(inverted_list, 1, alloc);
            case!(inverted_list, 2, alloc);
            case!(inverted_list, 3, alloc);
            case!(inverted_list, len(), alloc);
        }

        #[test]
        fn vee()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            case!(vee, 0, alloc);
            case!(vee, 1, alloc);
            case!(vee, 2, alloc);
            case!(vee, 3, alloc);
            case!(vee, len() / 2, alloc);
        }

        #[test]
        fn degenerate_dag()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            case!(degenerate_dag, 0, alloc);
            case!(degenerate_dag, 1, alloc);
            case!(degenerate_dag, 2, alloc);
            case!(degenerate_dag, 3, alloc);
            case!(degenerate_dag, 15, alloc);
        }

        #[test]
        fn degenerate_cyclic()
        {
            let alloc: Rc<DatumAllocator> = Default::default();

            case!(degenerate_cyclic, 0, alloc);
        }

        mod long
        {
            use super::*;

            #[test]
            #[ignore]
            fn degenerate_dag()
            {
                let alloc: Rc<DatumAllocator> = Default::default();
                case!(degenerate_dag, long_depth(), alloc);
            }
        }

        mod stack_overflow
        {
            use super::*;

            #[test]
            #[ignore]
            fn degenerate_cyclic1()
            {
                let alloc: Rc<DatumAllocator> = Default::default();

                case!(degenerate_cyclic, 1, alloc);
            }

            #[test]
            #[ignore]
            fn degenerate_cyclic2()
            {
                let alloc: Rc<DatumAllocator> = Default::default();

                case!(degenerate_cyclic, 2, alloc);
            }

            #[test]
            #[ignore]
            fn degenerate_cyclic3()
            {
                let alloc: Rc<DatumAllocator> = Default::default();

                case!(degenerate_cyclic, 3, alloc);
            }

            mod long
            {
                use super::*;

                #[test]
                #[ignore]
                fn list()
                {
                    let alloc: Rc<DatumAllocator> = Default::default();
                    case!(list, sizes::long_list_length(), alloc);
                }

                #[test]
                #[ignore]
                fn inverted_list()
                {
                    let alloc: Rc<DatumAllocator> = Default::default();
                    case!(inverted_list, sizes::long_list_length(), alloc);
                }

                #[test]
                #[ignore]
                fn vee()
                {
                    let alloc: Rc<DatumAllocator> = Default::default();
                    case!(vee, sizes::long_list_length(), alloc);
                }

                #[test]
                #[ignore]
                fn degenerate_dag()
                {
                    let alloc: Rc<DatumAllocator> = Default::default();
                    case!(degenerate_dag, sizes::long_list_length(), alloc);
                }

                #[test]
                #[ignore]
                fn degenerate_cyclic()
                {
                    let alloc: Rc<DatumAllocator> = Default::default();

                    case!(degenerate_cyclic, sizes::degenerate_depth(), alloc);
                }

                #[test]
                #[ignore]
                fn degenerate_cyclic_very_long()
                {
                    let alloc: Rc<DatumAllocator> = Default::default();
                    case!(degenerate_cyclic, sizes::long_list_length(), alloc);
                }
            }
        }
    }
}
