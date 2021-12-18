mod common
{
    pub mod rc_pair;
}
use {
    common::rc_pair::*,
    std::convert::identity,
};

tests_utils::eq_variations_tests!(My, Rc<Datum>, identity, DatumAllocator::new);
