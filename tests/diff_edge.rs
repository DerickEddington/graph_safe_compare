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
struct My1(Rc<Datum1>);
#[derive(Clone, Debug)]
struct My2(Rc<Datum2>);
#[derive(Clone, Debug)]
struct My3(Rc<Datum3>);

impl PartialEq for My1
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        precheck_interleave_equiv(self, other)
    }
}

impl Node for My1
{
    type Edge = My2;
    type Id = *const ();
    type Index = u16;

    fn id(&self) -> Self::Id
    {
        &*self.0 as *const Datum1 as *const _
    }

    fn amount_edges(&self) -> Self::Index
    {
        match &*self.0 {
            Datum1 { child: None } => 0,
            Datum1 { child: Some(_) } => 1,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self::Edge
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

impl Node for My2
{
    type Edge = My3;
    type Id = *const ();
    type Index = u16;

    fn id(&self) -> Self::Id
    {
        &*self.0 as *const Datum2 as *const _
    }

    fn amount_edges(&self) -> Self::Index
    {
        match &*self.0 {
            Datum2::Double(_, _) => 2,
            Datum2::Triple(_, _, _) => 3,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self::Edge
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

impl Node for My3
{
    type Edge = My1;
    type Id = *const ();
    type Index = u16;

    fn id(&self) -> Self::Id
    {
        &*self.0 as *const Datum3 as *const _
    }

    fn amount_edges(&self) -> Self::Index
    {
        match &*self.0.0.borrow() {
            Datum4::End => 0,
            Datum4::Link(_) => 1,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self::Edge
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
    assert_eq!(My1(Rc::new(leaf1)), My1(Rc::new(leaf2)));
}


#[test]
fn cyclic()
{
    fn make_shape() -> My1
    {
        let shape = My1(Rc::new(Datum1 {
            child: Some(Rc::new(Datum2::Triple(
                Rc::new(Datum3(RefCell::new(Datum4::End))),
                Rc::new(Datum3(RefCell::new(Datum4::End))),
                Rc::new(Datum3(RefCell::new(Datum4::End))),
            ))),
        }));
        if let Some(d2) = &shape.0.child {
            if let Datum2::Triple(d3a, _d3b, d3c) = &**d2 {
                *d3a.0.borrow_mut() = Datum4::Link(Rc::clone(&shape.0));
                *d3c.0.borrow_mut() = Datum4::Link(Rc::clone(&shape.0));
                return shape;
            }
        }
        unreachable!();
    }

    assert_eq!(make_shape(), make_shape());
}
