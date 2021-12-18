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
            generic::{
                equiv::{
                    Aborted,
                    Equiv,
                    Recur,
                },
                recursion::Reset,
            },
        },
    };

    #[derive(Default)]
    pub struct ListStack(LinkedList<(My, My)>);

    impl<M> Recur<ListStack> for Equiv<M, ListStack>
    {
        type Node = My;

        fn recur(
            &mut self,
            a: Self::Node,
            b: Self::Node,
        ) -> Result<bool, Aborted>
        {
            self.recur_stack.0.push_front((a, b));
            Ok(true)
        }

        fn next(&mut self) -> Option<(Self::Node, Self::Node)>
        {
            self.recur_stack.0.pop_front()
        }
    }

    impl Reset for ListStack
    {
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


/// Use our custom `Table`, `Rc`, and `Recur` types.
fn custom_equiv(
    a: &My,
    b: &My,
) -> bool
{
    use {
        custom_recur_stack::ListStack,
        cycle_deep_safe_compare::{
            basic::recursion::callstack::CallStack,
            generic,
        },
    };

    // Exercise the call-stack for the precheck since that is limited and will not overflow the
    // stack when the stack is already shallow, and use the list-stack for the interleave so great
    // depth is supported since an input could be very-deep.
    let precheck_on_callstack =
        generic::precheck_interleave_equiv::<_, custom_table::Map, CallStack, ListStack>(a, b);
    // Exercise our list-stack for the precheck.
    let precheck_on_liststack =
        generic::precheck_interleave_equiv::<_, custom_table::Map, ListStack, ListStack>(a, b);

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
