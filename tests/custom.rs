mod common
{
    pub mod rc_pair;
}

use {
    common::rc_pair::*,
    core::convert::identity,
};


/// Use `Arc` just because it's different than
/// `graph_safe_compare::generic::equiv_classes::premade::Rc`.  The multi-thread ability
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
        graph_safe_compare::generic::equiv_classes::{
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
/// `graph_safe_compare::generic::equiv_classes::premade::HashMap`.  The ordering of keys
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
        graph_safe_compare::generic::equiv_classes::Table,
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
/// `graph_safe_compare::wide_safe::recursion::stack::RecurStack`.
///
/// Also, enables this integration test to be used when the `graph_safe_compare` crate is
/// built without the "std" feature enabled, and enables running the test cases of very-deep
/// shapes.
mod custom_recur_stack
{
    extern crate alloc;

    use {
        super::{
            other_recur_stack::OtherStack,
            My,
        },
        alloc::collections::LinkedList,
        graph_safe_compare::{
            basic::recursion::callstack::CallStack,
            generic::equiv::{
                self,
                CounterpartsResult,
                EdgesIter,
                Equiv,
                RecurMode,
            },
            Cmp,
            Node,
        },
    };

    #[derive(Default)]
    pub struct ListStack(LinkedList<EdgesIter<My>>);

    #[derive(Debug)]
    pub struct ExhaustedError;

    impl<P> RecurMode<P> for ListStack
    where
        P: equiv::Params<Node = My, RecurMode = Self>,
        ExhaustedError: Into<P::Error>,
    {
        type Error = ExhaustedError;

        fn recur(
            it: &mut Equiv<P>,
            edges_iter: EdgesIter<P::Node>,
        ) -> Result<<P::Node as Node>::Cmp, Self::Error>
        {
            if it.recur_mode.0.len() < 2_usize.pow(30) {
                it.recur_mode.0.push_front(edges_iter);
                Ok(Cmp::new_equiv())
            }
            else {
                Err(ExhaustedError)
            }
        }

        fn next(&mut self) -> Option<CounterpartsResult<P::Node>>
        {
            while let Some(edges_iter) = self.0.front_mut() {
                let next = edges_iter.next();
                if next.is_some() {
                    return next;
                }
                // Remove empty iterators from the stack, after returning `Some` - just to be
                // different and to have inefficient memory usage without "tail-call elimination".
                drop(self.0.pop_front());
            }
            None
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

    impl From<OtherStack> for ListStack
    {
        fn from(_: OtherStack) -> Self
        {
            Self::default()
        }
    }
}

/// A different type, to test combining two custom `RecurMode` types.
mod other_recur_stack
{
    use {
        super::My,
        graph_safe_compare::{
            generic::equiv::{
                self,
                CounterpartsResult,
                EdgesIter,
                Equiv,
                RecurMode,
            },
            Cmp,
            Node,
        },
    };

    #[derive(Default)]
    pub struct OtherStack;

    pub enum OtherStackError<R>
    {
        Novel,
        Recur(R),
    }

    /// Like `CallStack` but with custom error.
    impl<P> RecurMode<P> for OtherStack
    where
        P: equiv::Params<Node = My, RecurMode = Self>,
        OtherStackError<P::Error>: Into<P::Error>,
    {
        type Error = OtherStackError<P::Error>;

        fn recur(
            it: &mut Equiv<P>,
            edges_iter: EdgesIter<P::Node>,
        ) -> Result<<P::Node as Node>::Cmp, Self::Error>
        {
            if true {
                for next in edges_iter {
                    match next {
                        Ok([a, b]) => match it.equiv_main(a, b) {
                            Ok(cmp) if cmp.is_equiv() => (),
                            result => return result.map_err(OtherStackError::Recur),
                        },
                        Err(cmp_amount_edges) => return Ok(cmp_amount_edges),
                    }
                }
                Ok(Cmp::new_equiv())
            }
            else {
                Err(OtherStackError::Novel)
            }
        }

        fn next(&mut self) -> Option<CounterpartsResult<P::Node>>
        {
            None
        }

        fn reset(self) -> Self
        {
            self
        }
    }
}


/// Use our own (dummy) PRNG to test not depending on any from the crate.
mod custom_rng
{
    use graph_safe_compare::cycle_safe::modes::interleave::random;

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


/// Use our custom `Table`, `Rc`, `RecurMode`, and `NumberGenerator` types.
fn custom_equiv(
    a: My,
    b: My,
) -> bool
{
    use {
        custom_recur_stack::{
            ExhaustedError,
            ListStack,
        },
        custom_rng::PseudoPseudoRNG,
        graph_safe_compare::{
            cycle_safe::modes::interleave,
            generic::precheck_interleave::{
                self,
                InterleaveError,
                PrecheckError,
            },
        },
        other_recur_stack::{
            OtherStack,
            OtherStackError,
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
        #[derive(Debug)]
        enum ExhaustedOrOtherError
        {
            ExhaustedList,
            OtherNovel,
        }

        impl From<OtherStackError<Self>> for PrecheckError<ExhaustedOrOtherError>
        {
            fn from(e: OtherStackError<Self>) -> Self
            {
                match e {
                    OtherStackError::Recur(e) => e,
                    OtherStackError::Novel => Self::RecurError(ExhaustedOrOtherError::OtherNovel),
                }
            }
        }

        impl From<ExhaustedError> for InterleaveError<ExhaustedOrOtherError>
        {
            fn from(_: ExhaustedError) -> Self
            {
                InterleaveError(ExhaustedOrOtherError::ExhaustedList)
            }
        }

        struct Args;

        impl precheck_interleave::Params<My> for Args
        {
            type Error = ExhaustedOrOtherError;
            type InterleaveParams = InterleaveArgs;
            type InterleaveRecurMode = ListStack;
            type PrecheckRecurMode = OtherStack;
        }

        precheck_interleave::equiv::<_, Args>(a.clone(), b.clone()).unwrap()
    };

    // Exercise our list-stack for the precheck.
    let precheck_on_liststack = {
        impl From<ExhaustedError> for PrecheckError<ExhaustedError>
        {
            fn from(e: ExhaustedError) -> Self
            {
                PrecheckError::RecurError(e)
            }
        }

        impl From<ExhaustedError> for InterleaveError<ExhaustedError>
        {
            fn from(e: ExhaustedError) -> Self
            {
                InterleaveError(e)
            }
        }

        struct Args;

        impl precheck_interleave::Params<My> for Args
        {
            type Error = ExhaustedError;
            type InterleaveParams = InterleaveArgs;
            type InterleaveRecurMode = ListStack;
            type PrecheckRecurMode = ListStack;
        }

        precheck_interleave::equiv::<_, Args>(a, b).unwrap()
    };

    assert_eq!(precheck_on_callstack, precheck_on_liststack);

    precheck_on_callstack == Ordering::Equal
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
