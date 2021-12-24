use {
    crate::{
        cycle_safe::modes::interleave::{
            self,
            random::default,
            Interleave,
        },
        deep_safe::recursion::vecstack::{
            self,
            VecStack,
        },
        generic::{
            equiv::{
                self,
                Equiv,
            },
            equiv_classes::premade::hash_map,
            precheck_interleave,
        },
        Node,
    },
    core::marker::PhantomData,
};


struct Args<N>(PhantomData<N>);

impl<N: Node> interleave::Params for Args<N>
{
    type Node = N;
    type RNG = default::RandomNumberGenerator;
    type Table = hash_map::Table<Self>;
}

impl<N: Node> hash_map::Params for Args<N>
{
    type Node = N;
}

impl<N: Node> vecstack::Params for Args<N>
{
    type Node = N;
}


/// Equivalence predicate that can handle cyclic graphs and very-deep graphs.
#[inline]
pub fn equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    impl<N: Node> equiv::Params for Args<N>
    {
        type DescendMode = Interleave<Self>;
        type Node = N;
        type RecurStack = VecStack<Self>;
    }

    let mut e = Equiv::<Args<N>>::default();
    e.is_equiv(a, b)
}


/// Like [`equiv`](equiv()) but first tries the precheck that is faster for small acyclic graphs.
#[inline]
pub fn precheck_equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    impl<N: Node> precheck_interleave::Params<N> for Args<N>
    {
        type InterleaveParams = Self;
        type InterleaveRecurStack = VecStack<Self>;
        type PrecheckRecurStack = VecStack<Self>;
    }

    precheck_interleave::equiv::<N, Args<N>>(a, b)
}
