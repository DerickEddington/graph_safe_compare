use cycle_deep_safe_compare::alt::basic::{
    precheck_interleave_equiv,
    Node,
};
use tests_utils::{
    node_types::borrow_pair::{
        Datum,
        DatumAllocator,
        Inner,
    },
    shapes::{
        Leaf,
        PairChainMaker,
    },
    DEGENERATE_TEST_DEPTH,
    LONG_LIST_TEST_LENGTH,
};


/// New type needed so we can impl the `Node` and `PartialEq` traits on it.
#[derive(Debug)]
struct My<'l>(&'l Datum<'l>);

impl<'l> PartialEq for My<'l>
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        precheck_interleave_equiv(self, other)
    }
}

impl<'l> Node for My<'l>
{
    type Edge = Self;
    type Id = *const Datum<'l>;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        self.0
    }

    fn amount_edges(&self) -> Self::Index
    {
        match &*self.0.0.borrow()
        {
            Inner::Leaf => 0,
            Inner::Pair(_, _) => 2,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self::Edge
    {
        match (idx, &*self.0.0.borrow())
        {
            (0, Inner::Pair(a, _)) => My(a),
            (1, Inner::Pair(_, b)) => My(b),
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
    let alloc = DatumAllocator::new(2);
    let leaf1 = Leaf::new_in(&&alloc);
    let leaf2 = Leaf::new_in(&&alloc);
    assert_eq!(My(leaf1), My(leaf2));
}

mod degenerate
{
    use super::*;

    #[test]
    fn dag_fast()
    {
        let alloc1 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
        let make1 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc1);
        let ddag1 = make1.degenerate_dag();

        let alloc2 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
        let make2 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc2);
        let ddag2 = make2.degenerate_dag();

        assert_eq!(My(ddag1), My(ddag2));
    }

    #[test]
    fn cyclic_works_and_fast()
    {
        let alloc1 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
        let make1 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc1);
        let dcyc1 = make1.degenerate_cyclic();

        let alloc2 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
        let make2 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc2);
        let dcyc2 = make2.degenerate_cyclic();

        assert_eq!(My(dcyc1), My(dcyc2));
    }

    mod long
    {
        use super::*;

        #[test]
        #[ignore]
        fn dag_stack_overflow()
        {
            const DEPTH: usize = 1000 * DEGENERATE_TEST_DEPTH;

            let alloc1 = DatumAllocator::new(DEPTH + 1);
            let make1 = PairChainMaker::new_with(DEPTH, &alloc1);
            let ddag1 = make1.degenerate_dag();

            let alloc2 = DatumAllocator::new(DEPTH + 1);
            let make2 = PairChainMaker::new_with(DEPTH, &alloc2);
            let ddag2 = make2.degenerate_dag();

            assert_eq!(My(ddag1), My(ddag2));
        }

        #[test]
        #[ignore]
        fn cyclic_stack_overflow()
        {
            const DEPTH: usize = 1000 * DEGENERATE_TEST_DEPTH;

            let alloc1 = DatumAllocator::new(DEPTH + 1);
            let make1 = PairChainMaker::new_with(DEPTH, &alloc1);
            let dcyc1 = make1.degenerate_cyclic();

            let alloc2 = DatumAllocator::new(DEPTH + 1);
            let make2 = PairChainMaker::new_with(DEPTH, &alloc2);
            let dcyc2 = make2.degenerate_cyclic();

            assert_eq!(My(dcyc1), My(dcyc2));
        }
    }

    mod derived_eq
    {
        use super::*;

        #[test]
        #[ignore]
        fn dag_slow()
        {
            let alloc1 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
            let make1 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc1);
            let ddag1 = make1.degenerate_dag();

            let alloc2 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
            let make2 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc2);
            let ddag2 = make2.degenerate_dag();

            assert_eq!(ddag1, ddag2);
        }

        #[test]
        #[ignore]
        fn cyclic_stack_overflow()
        {
            let alloc1 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
            let make1 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc1);
            let dcyc1 = make1.degenerate_cyclic();

            let alloc2 = DatumAllocator::new(DEGENERATE_TEST_DEPTH + 1);
            let make2 = PairChainMaker::new_with(DEGENERATE_TEST_DEPTH, &alloc2);
            let dcyc2 = make2.degenerate_cyclic();

            assert_eq!(dcyc1, dcyc2);
        }
    }
}

mod long_list
{
    use super::*;

    #[test]
    #[ignore]
    fn stack_overflow()
    {
        let alloc1 = DatumAllocator::new(2 * LONG_LIST_TEST_LENGTH + 1);
        let make1 = PairChainMaker::new_with(LONG_LIST_TEST_LENGTH, &alloc1);
        let list1 = make1.list();

        let alloc2 = DatumAllocator::new(2 * LONG_LIST_TEST_LENGTH + 1);
        let make2 = PairChainMaker::new_with(LONG_LIST_TEST_LENGTH, &alloc2);
        let list2 = make2.list();

        assert_eq!(My(list1), My(list2));
    }

    #[test]
    #[ignore]
    fn inverted_stack_overflow()
    {
        let alloc1 = DatumAllocator::new(2 * LONG_LIST_TEST_LENGTH + 1);
        let make1 = PairChainMaker::new_with(LONG_LIST_TEST_LENGTH, &alloc1);
        let ilist1 = make1.inverted_list();

        let alloc2 = DatumAllocator::new(2 * LONG_LIST_TEST_LENGTH + 1);
        let make2 = PairChainMaker::new_with(LONG_LIST_TEST_LENGTH, &alloc2);
        let ilist2 = make2.inverted_list();

        assert_eq!(My(ilist1), My(ilist2));
    }
}
