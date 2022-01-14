mod common
{
    pub mod rc_pair;
}
use {
    common::rc_pair::*,
    std::convert::identity,
};


// #[test]
// fn size()
// {
//     dbg!(std::mem::size_of::<Datum>());
//     dbg!(std::mem::size_of::<My>());
// }


tests_utils::eq_variations_tests!(My, Rc<Datum>, identity, DatumAllocator::new);
