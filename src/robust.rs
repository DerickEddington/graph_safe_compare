use crate::{
    cycle_safe::modes::interleave::Interleave,
    deep_safe::recursion::vecstack::VecStack,
    generic::{
        equiv::Equiv,
        equiv_classes::premade::HashMap,
        precheck_interleave_equiv,
    },
    Node,
};


/// Equivalence predicate that can handle cyclic graphs and very-deep graphs.
#[inline]
pub fn equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    let mut e = Equiv::<Interleave<HashMap<N>>, _>::new(VecStack::default());
    e.is_equiv(a, b)
}


/// Like [`equiv`] but first tries the precheck that is faster for small acyclic graphs.
#[inline]
pub fn precheck_equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    precheck_interleave_equiv::<N, HashMap<N>, VecStack<N>, VecStack<N>>(a, b)
}
