use std::rc::Rc;

pub(crate) const LONG_LIST_TEST_LENGTH: usize = 1_000_000;
pub(crate) const DEGEN_DAG_TEST_DEPTH: u8 = if cfg!(debug_assertions) { 28 } else { 33 };

pub(crate) trait Pair: Clone
{
    fn new(
        a: Self,
        b: Self,
    ) -> Self;

    fn set(
        &self,
        a: Self,
        b: Self,
    );
}

pub(crate) trait Leaf: Clone
{
    fn new() -> Self;
}

pub(crate) fn make_list<DR: Pair + Leaf>(mut len: usize) -> DR
{
    let mut head = Leaf::new();

    while len >= 1
    {
        len = len.saturating_sub(1);
        head = Pair::new(Leaf::new(), head);
    }

    head
}

fn make_degenerate<DR: Pair + Leaf>(mut depth: u8) -> (DR, DR)
{
    let tail: DR = Leaf::new();
    let mut head = tail.clone();

    while depth >= 1
    {
        depth = depth.saturating_sub(1);
        head = Pair::new(head.clone(), head.clone());
    }

    (head, tail)
}

pub(crate) fn make_degenerate_dag<DR: Pair + Leaf>(depth: u8) -> DR
{
    make_degenerate(depth).0
}

pub(crate) fn make_degenerate_cycle<DR: Pair + Leaf>(depth: u8) -> DR
{
    let (head, tail): (DR, DR) = make_degenerate(depth);

    Pair::set(&tail, head.clone(), head.clone());

    head
}

mod derived_eq
{
    use std::cell::RefCell;

    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    struct My(RefCell<Option<(Rc<Self>, Rc<Self>)>>);

    impl Pair for Rc<My>
    {
        fn new(
            a: Self,
            b: Self,
        ) -> Self
        {
            Rc::new(My(RefCell::new(Some((a, b)))))
        }

        fn set(
            &self,
            a: Self,
            b: Self,
        )
        {
            *self.0.borrow_mut() = Some((a, b));
        }
    }

    impl Leaf for Rc<My>
    {
        fn new() -> Self
        {
            Rc::new(My(RefCell::new(None)))
        }
    }

    #[test]
    #[ignore]
    fn long_list_stack_overflow()
    {
        let list1 = make_list::<Rc<My>>(LONG_LIST_TEST_LENGTH);
        let list2 = make_list::<Rc<My>>(LONG_LIST_TEST_LENGTH);
        assert!(list1 == list2);
    }

    // TODO: Change this into a #[bench] benchmark?
    #[test]
    #[ignore]
    fn degenerate_dag_slow()
    {
        let ddag1 = make_degenerate_dag::<Rc<My>>(DEGEN_DAG_TEST_DEPTH);
        let ddag2 = make_degenerate_dag::<Rc<My>>(DEGEN_DAG_TEST_DEPTH);
        // dbg!(&ddag1);
        assert!(ddag1 == ddag2);
    }

    #[test]
    #[ignore]
    fn degenerate_cycle_stack_overflow()
    {
        let dcyc1 = make_degenerate_cycle::<Rc<My>>(1);
        let dcyc2 = make_degenerate_cycle::<Rc<My>>(1);
        assert!(dcyc1 == dcyc2);
    }
}
