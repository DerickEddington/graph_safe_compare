#![cfg(all(
    any(rust_lib_feature = "test", rust_comp_feature = "unstable_features"),
    any(feature = "fastrand", feature = "oorandom", feature = "wyrng")
))]
#![cfg_attr(not(rust_lib_feature = "test"), feature(test))]

extern crate test;

use {
    core::num::NonZeroU16,
    graph_safe_compare::cycle_safe::modes::interleave::random::NumberGenerator as _,
    test::Bencher,
};


mod common;

const END: NonZeroU16 = common::defaults::FAST_LIMIT_MAX_RANGE_END;

#[cfg(feature = "fastrand")]
mod fastrand
{
    use {
        super::*,
        graph_safe_compare::cycle_safe::modes::interleave::random::fastrand,
    };

    #[bench]
    fn rand_upto(bencher: &mut Bencher)
    {
        let mut rng = fastrand::RandomNumberGenerator::default();
        bencher.iter(|| rng.rand_upto(END))
    }
}

#[cfg(feature = "oorandom")]
mod oorandom
{
    use {
        super::*,
        graph_safe_compare::cycle_safe::modes::interleave::random::oorandom,
    };

    #[bench]
    fn rand_upto(bencher: &mut Bencher)
    {
        let mut rng = oorandom::RandomNumberGenerator::default();
        bencher.iter(|| rng.rand_upto(END))
    }
}

#[cfg(feature = "wyrng")]
mod wyrng
{
    use {
        super::*,
        graph_safe_compare::cycle_safe::modes::interleave::random::wyrng,
    };

    #[bench]
    fn rand_upto(bencher: &mut Bencher)
    {
        let mut rng = wyrng::RandomNumberGenerator::default();
        bencher.iter(|| rng.rand_upto(END))
    }
}
