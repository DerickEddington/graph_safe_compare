mod common
{
    pub mod rc_pair;
}
use {
    common::rc_pair::*,
    core::convert::identity,
};


/// Use `Arc` just because it's different than
/// `cycle_deep_safe_compare::generic::equiv_classes::premade::Rc`.  The multi-thread ability
/// provided by `Arc` is ignored.
mod custom_rc
{
    extern crate alloc;

    use {
        alloc::sync::Arc,
        core::{
            cell::Cell,
            ops::Deref,
        },
        cycle_deep_safe_compare::generic::equiv_classes::{
            self,
            Class,
        },
    };

    #[derive(Clone)]
    pub struct Rc(Arc<Cell<Class<Self>>>);

    impl Deref for Rc
    {
        type Target = Cell<Class<Self>>;

        fn deref(&self) -> &Self::Target
        {
            &*self.0
        }
    }

    impl equiv_classes::Rc for Rc
    {
        fn new(val: Cell<Class<Self>>) -> Self
        {
            Self(Arc::new(val))
        }
    }
}


/// Use `BTreeMap` just because it's different than
/// `cycle_deep_safe_compare::generic::equiv_classes::premade::HashMap`.  The ordering of keys
/// provided by `BTreeMap` is ignored.
mod custom_table
{
    extern crate alloc;

    use {
        super::{
            My,
            Node,
        },
        alloc::collections::BTreeMap,
        cycle_deep_safe_compare::generic::equiv_classes::Table,
    };

    #[derive(Default)]
    pub struct Map(BTreeMap<<My as Node>::Id, super::custom_rc::Rc>);

    impl Table for Map
    {
        type Node = My;
        type Rc = super::custom_rc::Rc;

        fn get(
            &self,
            k: &<Self::Node as Node>::Id,
        ) -> Option<&Self::Rc>
        {
            self.0.get(k)
        }

        fn insert(
            &mut self,
            k: <My as Node>::Id,
            v: Self::Rc,
        )
        {
            self.0.insert(k, v);
        }
    }
}


/// Use `LinkedList` just because it's different than
/// `cycle_deep_safe_compare::deep_safe::recursion::vecstack::VecStack`.
///
/// Also, enables this integration test to be used when the `cycle_deep_safe_compare` crate is
/// built without the "std" feature enabled, and enables running the test cases of very-deep
/// shapes.
mod custom_recur_stack
{
    extern crate alloc;

    use {
        super::My,
        alloc::collections::LinkedList,
        cycle_deep_safe_compare::{
            basic::recursion::callstack::CallStack,
            generic::equiv::{
                self,
                Aborted,
                Equiv,
                RecurStack,
            },
        },
    };

    #[derive(Default)]
    pub struct ListStack(LinkedList<(My, My)>);

    impl<P> RecurStack<P> for ListStack
    where P: equiv::Params<Node = My, RecurStack = Self>
    {
        fn recur(
            it: &mut Equiv<P>,
            a: P::Node,
            b: P::Node,
        ) -> Result<bool, Aborted>
        {
            it.recur_stack.0.push_front((a, b));
            Ok(true)
        }

        fn next(&mut self) -> Option<(P::Node, P::Node)>
        {
            self.0.pop_front()
        }

        fn reset(mut self) -> Self
        {
            self.0.clear();
            self
        }
    }

    impl From<CallStack> for ListStack
    {
        fn from(_: CallStack) -> Self
        {
            Self::default()
        }
    }
}


/// Use our own (dummy) PRNG to test not depending on any from the crate.
mod custom_rng
{
    use cycle_deep_safe_compare::cycle_safe::modes::interleave::random;

    #[derive(Default)]
    pub struct PseudoPseudoRNG(u128);

    impl random::NumberGenerator for PseudoPseudoRNG
    {
        fn rand_upto(
            &mut self,
            exclusive_end: std::num::NonZeroU16,
        ) -> u16
        {
            self.0 = self.0.wrapping_mul(42);
            self.0 = self.0.wrapping_add(987654321);
            self.0 as u16 % exclusive_end
        }
    }
}


/// Use our custom `Table`, `Rc`, `RecurStack`, and `NumberGenerator` types.
fn custom_equiv(
    a: &My,
    b: &My,
) -> bool
{
    use {
        custom_recur_stack::ListStack,
        custom_rng::PseudoPseudoRNG,
        cycle_deep_safe_compare::{
            basic::recursion::callstack::CallStack,
            cycle_safe::modes::interleave,
            generic::precheck_interleave,
        },
    };

    struct InterleaveArgs;

    impl interleave::Params for InterleaveArgs
    {
        type Node = My;
        type RNG = PseudoPseudoRNG;
        type Table = custom_table::Map;

        // Use custom values for these constants, not their defaults.
        const FAST_LIMIT_MAX: u16 = Self::PRECHECK_LIMIT / 4;
        const PRECHECK_LIMIT: u16 = 2000;
        const SLOW_LIMIT: u16 = Self::PRECHECK_LIMIT / 2;
    }

    // Exercise the call-stack for the precheck since that is limited and will not overflow the
    // stack when the stack is already shallow, and use the list-stack for the interleave so great
    // depth is supported since an input could be very-deep.
    let precheck_on_callstack = {
        struct Args;

        impl precheck_interleave::Params<My> for Args
        {
            type InterleaveParams = InterleaveArgs;
            type InterleaveRecurStack = ListStack;
            type PrecheckRecurStack = CallStack;
        }

        precheck_interleave::equiv::<_, Args>(a, b)
    };
    // Exercise our list-stack for the precheck.
    let precheck_on_liststack = {
        struct Args;

        impl precheck_interleave::Params<My> for Args
        {
            type InterleaveParams = InterleaveArgs;
            type InterleaveRecurStack = ListStack;
            type PrecheckRecurStack = ListStack;
        }

        precheck_interleave::equiv::<_, Args>(a, b)
    };

    assert_eq!(precheck_on_callstack, precheck_on_liststack);

    precheck_on_callstack
}

mod eq_variation
{
    use super::*;

    tests_utils::eq_variation_mod_body!(
        custom_equiv,
        My,
        Rc<Datum>,
        identity,
        DatumAllocator::new
    );
}

tests_utils::eq_shapes_tests!(identity, DatumAllocator::new, eq_variation::MyEq::new,
                              #[cfg(all())], #[cfg(all())]);
