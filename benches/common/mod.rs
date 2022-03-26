/// Reuse this module from the integration tests.
#[cfg_attr(unix, path = "../../tests/common/borrow_pair.rs")]
#[cfg_attr(windows, path = "..\\..\\tests\\common\\borrow_pair.rs")]
pub mod borrow_pair;


/// The default values of the associated constants of [`graph_safe_compare`].
pub mod defaults
{
    // For the below `const` items, because the `super` module is not `pub` and this file is
    // shared by multiple "targets" that each do not use all the items.
    #![allow(dead_code)]

    use {
        super::borrow_pair::My,
        graph_safe_compare::{
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
            marker::PhantomData,
            num::NonZeroU16,
            ops::Deref,
        },
    };

    pub const PRECHECK_LIMIT: u32 = <Args as interleave::Params>::PRECHECK_LIMIT as _;
    pub const FAST_LIMIT_MAX: u32 = <Args as interleave::Params>::FAST_LIMIT_MAX as _;
    pub const SLOW_LIMIT: u32 = <Args as interleave::Params>::SLOW_LIMIT as _;

    pub const FAST_LIMIT_MAX_RANGE_END: NonZeroU16 = Interleave::<Args>::FAST_LIMIT_MAX_RANGE_END;
    pub const SLOW_LIMIT_NEG: i32 = Interleave::<Args>::SLOW_LIMIT_NEG;

    // The items defined below are not involved in the benchmarks, and they're only needed to be
    // able to use `Args` to define our constants above .

    #[derive(Clone, Default)]
    struct Args<'l>(PhantomData<&'l ()>);

    impl<'l> interleave::Params for Args<'l>
    {
        type Node = My<'l>;
        type RNG = Self;
        type Table = Self;
    }

    impl random::NumberGenerator for Args<'_>
    {
        fn rand_upto(
            &mut self,
            _exclusive_end: NonZeroU16,
        ) -> u16
        {
            unreachable!()
        }
    }

    impl Deref for Args<'_>
    {
        type Target = Cell<equiv_classes::Class<Self>>;

        fn deref(&self) -> &Self::Target
        {
            unreachable!()
        }
    }

    impl equiv_classes::Rc for Args<'_>
    {
        fn new(_value: Cell<equiv_classes::Class<Self>>) -> Self
        {
            unreachable!()
        }
    }

    impl<'l> equiv_classes::Table for Args<'l>
    {
        type Node = My<'l>;
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
}
