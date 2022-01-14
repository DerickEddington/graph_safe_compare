use {
    cycle_deep_safe_compare::{
        basic::recursion::callstack::CallStack,
        cycle_safe::modes::interleave::{
            self,
            random,
        },
        generic::{
            equiv_classes::premade::hash_map,
            precheck_interleave,
        },
        robust,
        utils::IntoOk as _,
        Node,
    },
    std::{
        cell::RefCell,
        convert::{
            identity,
            Infallible,
        },
    },
    tests_utils::{
        node_types::diff_index::{
            Datum,
            DatumAllocator,
            Index,
            Inner,
        },
        shapes::Leaf,
    },
};


/// New type needed so we can impl the `Node` and `PartialEq` traits on it.
#[derive(Clone, Debug)]
struct My(Datum);

impl PartialEq for My
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        let callstack = {
            struct Args;

            impl precheck_interleave::Params<My> for Args
            {
                type Error = Infallible;
                type InterleaveParams = Self;
                type InterleaveRecurStack = CallStack;
                type PrecheckRecurStack = CallStack;
            }

            impl hash_map::Params for Args
            {
                type Node = My;
            }

            impl interleave::Params for Args
            {
                type Node = My;
                type RNG = random::default::RandomNumberGenerator;
                type Table = hash_map::Table<Self>;
            }

            precheck_interleave::equiv::<_, Args>(self, other).into_ok()
        };
        let robust = robust::precheck_equiv(self, other);
        assert_eq!(callstack, robust);
        callstack
    }
}

impl Node for My
{
    type Cmp = bool;
    type Id = (Index, *const [RefCell<Inner>]);
    type Index = Index;

    fn id(&self) -> Self::Id
    {
        (self.0.index, &*self.0.region)
    }

    fn amount_edges(&self) -> Self::Index
    {
        match *self.0.deref() {
            Inner::Leaf => Index::Zero,
            Inner::Pair(_, _) => Index::Two,
        }
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self
    {
        match (idx, &*self.0.deref()) {
            (Index::Zero, Inner::Pair(a, _)) => My(a.clone()),
            (Index::One, Inner::Pair(_, b)) => My(b.clone()),
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
    let leaf1 = Leaf::new_in(&DatumAllocator::new(1));
    let leaf2 = Leaf::new_in(&DatumAllocator::new(1));
    assert_eq!(My(leaf1), My(leaf2));
}


mod degenerate
{
    use super::*;

    // Must not cause amount allocated to exceed max `Index` discriminant.
    const DEPTH: u32 = 7;

    #[test]
    fn dag()
    {
        tests_utils::eq_case!(
            identity,
            DatumAllocator::new,
            DEPTH + 1,
            DEPTH,
            degenerate_dag,
            My
        );
    }

    #[test]
    fn cyclic_works()
    {
        tests_utils::eq_case!(
            identity,
            DatumAllocator::new,
            DEPTH + 1,
            DEPTH,
            degenerate_cyclic,
            My
        );
    }

    mod derived_eq
    {
        use super::*;

        #[test]
        fn dag()
        {
            tests_utils::eq_case!(
                identity,
                DatumAllocator::new,
                DEPTH + 1,
                DEPTH,
                degenerate_dag,
                identity
            );
        }

        #[test]
        #[ignore]
        fn cyclic_stack_overflow()
        {
            tests_utils::eq_case!(
                identity,
                DatumAllocator::new,
                DEPTH + 1,
                DEPTH,
                degenerate_cyclic,
                identity
            );
        }
    }
}
