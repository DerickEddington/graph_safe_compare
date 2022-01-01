/// Reuse this module from the integration tests.
#[path = "../../tests/common/rc_pair.rs"]
pub mod rc_pair;


/// The default values of the associated constants of [`cycle_deep_safe_compare`].
pub mod defaults
{
    // For the below `const` items, because the `super` module is not `pub` and this file is
    // shared by multiple "targets" that each do not use all the items.
    #![allow(dead_code)]

    use {
        super::rc_pair::My,
        cycle_deep_safe_compare::{
            cycle_safe::modes::interleave::{
                self,
                random,
                Interleave,
            },
            generic::equiv_classes,
            Node,
        },
        std::{
            cell::Cell,
            num::NonZeroU16,
            ops::Deref,
        },
    };

    pub const PRECHECK_LIMIT: u32 = <Args as interleave::Params>::PRECHECK_LIMIT as _;
    pub const FAST_LIMIT_MAX: u32 = <Args as interleave::Params>::FAST_LIMIT_MAX as _;
    pub const SLOW_LIMIT: u32 = <Args as interleave::Params>::SLOW_LIMIT as _;

    pub const FAST_LIMIT_MAX_RANGE_END: NonZeroU16 = Interleave::<Args>::FAST_LIMIT_MAX_RANGE_END;
    pub const SLOW_LIMIT_NEG: i32 = Interleave::<Args>::SLOW_LIMIT_NEG;

    #[cfg(feature = "alloc")]
    pub use deep_safe::VECSTACK_INITIAL_CAPACITY;

    // The items defined below are not involved in the benchmarks, and they're only needed to be
    // able to `impl` the `interleave::Params` and `vecstack::Params` traits.

    #[derive(Clone, Default)]
    struct Args;

    impl interleave::Params for Args
    {
        type Node = My;
        type RNG = Self;
        type Table = Self;
    }

    impl random::NumberGenerator for Args
    {
        fn rand_upto(
            &mut self,
            _exclusive_end: NonZeroU16,
        ) -> u16
        {
            unreachable!()
        }
    }

    impl Deref for Args
    {
        type Target = Cell<equiv_classes::Class<Self>>;

        fn deref(&self) -> &Self::Target
        {
            unreachable!()
        }
    }

    impl equiv_classes::Rc for Args
    {
        fn new(_value: Cell<equiv_classes::Class<Self>>) -> Self
        {
            unreachable!()
        }
    }

    impl equiv_classes::Table for Args
    {
        type Node = My;
        type Rc = Self;

        fn get(
            &self,
            _k: &<Self::Node as Node>::Id,
        ) -> Option<&Self::Rc>
        {
            unreachable!()
        }

        fn insert(
            &mut self,
            _k: <Self::Node as Node>::Id,
            _v: Self::Rc,
        )
        {
            unreachable!()
        }
    }

    #[cfg(feature = "alloc")]
    mod deep_safe
    {
        use {
            super::*,
            cycle_deep_safe_compare::deep_safe::recursion::vecstack,
        };

        pub const VECSTACK_INITIAL_CAPACITY: u32 =
            <Args as vecstack::Params>::INITIAL_CAPACITY as _;

        impl vecstack::Params for Args
        {
            type Node = My;
        }
    }
}
