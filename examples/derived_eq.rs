//! Demonstrate that the usual derived PartialEq algorithm cannot handle cyclic
//! graphs nor very-deep DAGs and that it handles large amounts of shared
//! structure inefficiently.

#![cfg(test)]

use std::rc::Rc;

use tests_utils::{
    node_types::rc_pair::Datum,
    shapes::PairChainMaker,
    DEGENERATE_TEST_DEPTH,
    LONG_LIST_TEST_LENGTH,
};

#[test]
#[ignore]
fn long_list_stack_overflow()
{
    let list1: Rc<Datum> = PairChainMaker::new(LONG_LIST_TEST_LENGTH).list();
    let list2: Rc<Datum> = PairChainMaker::new(LONG_LIST_TEST_LENGTH).list();
    assert!(list1 == list2);
}

// TODO: Change this into a #[bench] benchmark?
#[test]
#[ignore]
fn degenerate_dag_slow()
{
    let ddag1: Rc<Datum> = PairChainMaker::new(DEGENERATE_TEST_DEPTH).degenerate_dag();
    let ddag2: Rc<Datum> = PairChainMaker::new(DEGENERATE_TEST_DEPTH).degenerate_dag();
    assert!(ddag1 == ddag2);
}

#[test]
#[ignore]
fn degenerate_cyclic_stack_overflow()
{
    let dcyc1: Rc<Datum> = PairChainMaker::new(1).degenerate_cyclic();
    let dcyc2: Rc<Datum> = PairChainMaker::new(1).degenerate_cyclic();
    assert!(dcyc1 == dcyc2);
}
