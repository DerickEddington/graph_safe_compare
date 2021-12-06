use {
    cycle_deep_safe_compare::alt::basic::{
        precheck_interleave_equiv,
        Node,
    },
    std::{
        cell::RefCell,
        rc::Rc,
    },
    tests_utils::node_types::diff_edge::{
        Datum1,
        Datum2,
        Datum3,
        Datum4,
    },
};


/// New type needed so we can impl the `Node` and `PartialEq` traits on it.
#[derive(Clone, Debug)]
enum My
{
    A(My1),
    B(My2),
    C(My3),
}
#[derive(Clone, Debug)]
struct My1(Rc<Datum1>);
#[derive(Clone, Debug)]
struct My2(Rc<Datum2>);
#[derive(Clone, Debug)]
struct My3(Rc<Datum3>);

impl PartialEq for My
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        precheck_interleave_equiv(self, other)
    }
}

impl Node for My
{
    type Id = *const ();
    type Index = u16;

    fn id(&self) -> Self::Id
    {
        match self {
            My::A(a) => a.id() as _,
            My::B(b) => b.id() as _,
            My::C(c) => c.id() as _,
        }
    }

    fn amount_edges(&self) -> Self::Index
    {
        match self {
            My::A(a) => a.amount_edges(),
            My::B(b) => b.amount_edges(),
            My::C(c) => c.amount_edges(),
        }
    }

    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Self
    {
        match self {
            My::A(a) => My::B(a.get_edge(index)),
            My::B(b) => My::C(b.get_edge(index)),
            My::C(c) => My::A(c.get_edge(index)),
        }
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> bool
    {
        match (self, other) {
            (My::A(a1), My::A(a2)) => a1.equiv_modulo_edges(a2),
            (My::B(b1), My::B(b2)) => b1.equiv_modulo_edges(b2),
            (My::C(c1), My::C(c2)) => c1.equiv_modulo_edges(c2),
            _ => false,
        }
    }
}

impl My1
{
    fn id(&self) -> *const Datum1
    {
        &*self.0
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
    fn id(&self) -> *const Datum2
    {
        &*self.0
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
        // Note: Comparing the variants is not strictly needed, since their
        // amount of edges differ, but this exercises this method a little
        // differently.
        matches!(
            (&*self.0, &*other.0),
            (Datum2::Double(_, _), Datum2::Double(_, _))
                | (Datum2::Triple(_, _, _), Datum2::Triple(_, _, _))
        )
    }
}

impl My3
{
    fn id(&self) -> *const Datum3
    {
        &*self.0
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


#[test]
fn rudimentary()
{
    let leaf1 = Datum1 { child: None };
    let leaf2 = Datum1 { child: None };
    assert_eq!(My::A(My1(Rc::new(leaf1))), My::A(My1(Rc::new(leaf2))));
}


#[test]
fn cyclic()
{
    fn make_shape() -> My
    {
        let shape = My::A(My1(Rc::new(Datum1 {
            child: Some(Rc::new(Datum2::Triple(
                Rc::new(Datum3(RefCell::new(Datum4::End))),
                Rc::new(Datum3(RefCell::new(Datum4::End))),
                Rc::new(Datum3(RefCell::new(Datum4::End))),
            ))),
        })));
        if let My::A(my1) = &shape {
            if let Some(d2) = &my1.0.child {
                if let Datum2::Triple(d3a, _d3b, d3c) = &**d2 {
                    *d3a.0.borrow_mut() = Datum4::Link(Rc::clone(&my1.0));
                    *d3c.0.borrow_mut() = Datum4::Link(Rc::clone(&my1.0));
                    return shape;
                }
            }
        }
        unreachable!();
    }

    assert_eq!(make_shape(), make_shape());
}
