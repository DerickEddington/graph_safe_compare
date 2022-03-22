use {
    graph_safe_compare::{
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
        utils::RefId,
        Node,
        Step,
    },
    std::{
        cell::RefCell,
        convert::{
            identity,
            Infallible,
        },
        rc::Rc,
    },
    tests_utils::{
        node_types::diff_index::{
            Datum,
            DatumAllocator,
            Index::{
                One,
                Two,
                Zero,
            },
            Inner,
        },
        shapes::Leaf,
    },
};


#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Default)]
struct Index(tests_utils::node_types::diff_index::Index);

impl Step for Index
{
    fn increment(&self) -> Self
    {
        Self(self.0.increment())
    }
}


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
                type InterleaveRecurMode = CallStack;
                type PrecheckRecurMode = CallStack;
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

            #[allow(unstable_name_collisions)]
            precheck_interleave::equiv::<_, Args>(self.clone(), other.clone()).unwrap()
        };
        let robust = robust::precheck_equiv(self.clone(), other.clone());
        assert_eq!(callstack, robust);
        callstack
    }
}

impl Node for My
{
    type Cmp = bool;
    type Id = (Index, RefId<Rc<[RefCell<Inner>]>>);
    type Index = Index;

    fn id(&self) -> Self::Id
    {
        (Index(self.0.index), RefId(Rc::clone(&self.0.region)))
    }

    fn amount_edges(&self) -> Self::Index
    {
        Index(match *self.0.deref() {
            Inner::Leaf => Zero,
            Inner::Pair(_, _) => Two,
        })
    }

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self
    {
        match (idx.0, &*self.0.deref()) {
            (Zero, Inner::Pair(a, _)) => My(a.clone()),
            (One, Inner::Pair(_, b)) => My(b.clone()),
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
