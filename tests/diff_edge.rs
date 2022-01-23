use {
    cycle_deep_safe_compare::{
        utils::RefId,
        Node,
    },
    std::{
        cell::RefCell,
        rc::Rc,
    },
    tests_utils::{
        node_types::diff_edge::{
            Datum1,
            Datum2,
            Datum3,
            Datum4,
        },
        shapes::{
            Allocator,
            Leaf,
            Pair,
        },
    },
};


#[derive(Clone, Debug)]
struct My(Kind);

// Note that these derived PartialEq implementations do not do a `cycle_deep_safe_compare`
// algorithm and are only used for demonstrating the limitations of the derived algorithm.  When
// `cycle_deep_safe_compare` algorithms are tested against this type, their functions must be
// called directly.

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Kind
{
    A(My1),
    B(My2),
    C(My3),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct My1(Rc<Datum1>);
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct My2(Rc<Datum2>);
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct My3(Rc<Datum3>);

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Id
{
    Id1(RefId<Rc<Datum1>>),
    Id2(RefId<Rc<Datum2>>),
    Id3(RefId<Rc<Datum3>>),
}

impl Node for My
{
    type Cmp = bool;
    type Id = Id;
    type Index = u16;

    fn id(&self) -> Self::Id
    {
        match self {
            My(Kind::A(a)) => Id::Id1(a.id()),
            My(Kind::B(b)) => Id::Id2(b.id()),
            My(Kind::C(c)) => Id::Id3(c.id()),
        }
    }

    fn amount_edges(&self) -> Self::Index
    {
        match self {
            My(Kind::A(a)) => a.amount_edges(),
            My(Kind::B(b)) => b.amount_edges(),
            My(Kind::C(c)) => c.amount_edges(),
        }
    }

    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Self
    {
        match self {
            My(Kind::A(a)) => My(Kind::B(a.get_edge(index))),
            My(Kind::B(b)) => My(Kind::C(b.get_edge(index))),
            My(Kind::C(c)) => My(Kind::A(c.get_edge(index))),
        }
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> bool
    {
        match (self, other) {
            (My(Kind::A(a1)), My(Kind::A(a2))) => a1.equiv_modulo_edges(a2),
            (My(Kind::B(b1)), My(Kind::B(b2))) => b1.equiv_modulo_edges(b2),
            (My(Kind::C(c1)), My(Kind::C(c2))) => c1.equiv_modulo_edges(c2),
            _ => false,
        }
    }
}

impl My1
{
    fn id(&self) -> RefId<Rc<Datum1>>
    {
        RefId(Rc::clone(&self.0))
    }

    fn amount_edges(&self) -> u16
    {
        match &*self.0 {
            Datum1 { child: None } => 0,
            Datum1 { child: Some(_) } => 1,
        }
    }

    fn get_edge(
        &self,
        idx: &u16,
    ) -> My2
    {
        match (idx, &*self.0) {
            (0, Datum1 { child: Some(d2) }) => My2(Rc::clone(d2)),
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

impl My2
{
    fn id(&self) -> RefId<Rc<Datum2>>
    {
        RefId(Rc::clone(&self.0))
    }

    fn amount_edges(&self) -> u16
    {
        match &*self.0 {
            Datum2::Double(_, _) => 2,
            Datum2::Triple(_, _, _) => 3,
        }
    }

    fn get_edge(
        &self,
        idx: &u16,
    ) -> My3
    {
        match (idx, &*self.0) {
            (0, Datum2::Double(d2a, _)) => My3(Rc::clone(d2a)),
            (1, Datum2::Double(_, d2b)) => My3(Rc::clone(d2b)),
            (0, Datum2::Triple(d2a, _, _)) => My3(Rc::clone(d2a)),
            (1, Datum2::Triple(_, d2b, _)) => My3(Rc::clone(d2b)),
            (2, Datum2::Triple(_, _, d2c)) => My3(Rc::clone(d2c)),
            _ => panic!("invalid"),
        }
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> bool
    {
        // Note: Comparing the variants is not strictly needed, since their amount of edges
        // differ, but this exercises this method a little differently.
        matches!(
            (&*self.0, &*other.0),
            (Datum2::Double(_, _), Datum2::Double(_, _))
                | (Datum2::Triple(_, _, _), Datum2::Triple(_, _, _))
        )
    }
}

impl My3
{
    fn id(&self) -> RefId<Rc<Datum3>>
    {
        RefId(Rc::clone(&self.0))
    }

    fn amount_edges(&self) -> u16
    {
        match &*self.0.0.borrow() {
            Datum4::End => 0,
            Datum4::Link(_) => 1,
        }
    }

    fn get_edge(
        &self,
        idx: &u16,
    ) -> My1
    {
        match (idx, &*self.0.0.borrow()) {
            (0, Datum4::Link(d1)) => My1(Rc::clone(d1)),
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


impl Leaf for Kind
{
    type Alloc = KindAllocator;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        alloc.alloc()
    }
}

impl Pair for Kind
{
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        match (self, a, b) {
            (Kind::C(My3(d3)), Kind::C(My3(a)), Kind::C(My3(b))) => {
                let pair = Datum2::Double(a, b);
                let d1 = Datum1 { child: Some(Rc::new(pair)) };
                *d3.0.borrow_mut() = Datum4::Link(Rc::new(d1));
            },
            _ => panic!("unsupported"),
        }
    }

    fn take(&self) -> Option<(Self, Self)>
    {
        if let Kind::C(My3(d3)) = self {
            let val = d3.0.replace(Datum4::End);
            match val {
                Datum4::Link(d1) => match &d1.child {
                    Some(d2) => match &**d2 {
                        Datum2::Double(a, b) =>
                            Some((Kind::C(My3(Rc::clone(a))), Kind::C(My3(Rc::clone(b))))),
                        Datum2::Triple(_, _, _) => unreachable!(),
                    },
                    None => unreachable!(),
                },
                Datum4::End => None,
            }
        }
        else {
            unreachable!()
        }
    }
}

pub struct KindAllocator;

impl KindAllocator
{
    pub fn new(_size: u32) -> Self
    {
        Self
    }
}

impl Allocator<Kind> for KindAllocator
{
    fn alloc(&self) -> Kind
    {
        Kind::C(My3(Rc::new(Datum3(RefCell::new(Datum4::End)))))
    }
}


use std::convert::identity;

tests_utils::eq_variations_tests!(My, Kind, identity, KindAllocator::new);
