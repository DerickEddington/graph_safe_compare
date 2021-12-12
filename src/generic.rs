use {
    self::{
        equiv::{
            Descend,
            Equiv,
            Recur,
        },
        recursion::Reset,
    },
    crate::{
        basic::modes::limited::Limited,
        cycle_safe::modes::interleave::{
            Interleave,
            PRE_LIMIT,
        },
        Node,
    },
    core::ops::ControlFlow,
};


// TODO: Make generic on the "already seen" map type, once that is generic.
/// Equivalence predicate that can handle cyclic graphs, but first tries the precheck that is
/// faster for small acyclic graphs, and that requires choosing specific type arguments for the
/// recursion stack used for both the precheck and the interleave.  Safe for very-deep graphs only
/// when the recursion-stack type is.
#[inline]
pub fn precheck_interleave_equiv<N: Node, SP: Default + Reset + Into<SI>, SI>(
    a: &N,
    b: &N,
) -> bool
where
    Equiv<Limited<N>, SP>: Descend<Node = N> + Recur<Node = N>,
    Equiv<Interleave<N>, SI>: Descend<Node = N> + Recur<Node = N>,
{
    let mut e = Equiv::<Limited<N>, SP>::new(PRE_LIMIT.into(), SP::default());

    match e.precheck_equiv(a, b) {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => {
            let mut e: Equiv<Interleave<N>, SI> = e.into();
            e.is_equiv(a, b)
        },
    }
}


pub(super) mod recursion
{
    /// An aborted precheck, that uses particular types of recursion-stacks, might leave some
    /// elements on such a stack, in which case it needs to be reset before doing the interleave
    /// using the same stack.
    pub trait Reset
    {
        /// Reset a `Self` as appropriate.
        ///
        /// When it is more efficient, the same `self` value should be reset and then returned.
        /// But a newly-created value may be returned if desired.
        fn reset(self) -> Self;
    }
}


pub(super) mod equiv
{
    use {
        crate::Node,
        core::borrow::Borrow,
    };

    /// The state for an invocation of a variation of the algorithm.
    ///
    /// The `mode` type `M` must, and the `recur_stack` type `S` may, themselves be generic over a
    /// type parameter, conventionally named `N`, for the generic [`Node`] types, so that various
    /// `impl`s on `Equiv` can have the `N` parameter be constrained.
    pub struct Equiv<M, S>
    {
        /// Decremented for every node edge descended into.  May be arbitrarily initialized,
        /// reset, and interpreted.
        pub ticker:      i32,
        /// Controls if node edges are descended into.
        pub mode:        M,
        /// Representation of recursion continuations.
        pub recur_stack: S,
    }

    /// Indicates that the algorithm aborted early without determining the result.
    ///
    /// This occurs when the [`Descend::do_recur`] returns `false`.
    ///
    /// Used as the value in a `Result::Err`.
    pub struct Aborted;

    /// Controls if node edges are descended into.
    ///
    /// Implemented on [`Equiv`] of various types, instead of on the mode types, so that these
    /// methods have access to the entire `Equiv` values.
    pub trait Descend
    {
        /// Constrains `impl<N>` for a particular [`Equiv`] variation.
        type Node: Node;

        /// Controls if all the descendents of a pair of nodes being compared should be
        /// immediately skipped.  Called before getting any edges.
        ///
        /// Returning `true` causes all the descendents to begin to be compared, individually
        /// under the control of [`Self::do_recur`].
        ///
        /// Returning `false` causes all the descendents to be skipped, and assumes they are
        /// already known to satisfy equivalence with their counterparts, and causes the
        /// comparison traversal to immediately continue on to the next non-descendent
        /// (i.e. sibling or ancestor) nodes.
        fn do_edges(
            &mut self,
            _a: &Self::Node,
            _b: &Self::Node,
        ) -> bool;

        /// Controls if individual descendent counterparts will be gotten and descended into via
        /// recursion.  Called after getting and comparing any previous sibling descendents.
        ///
        /// Returning `true` causes the next counterparts to be gotten and recurred on to compare
        /// them.
        ///
        /// Returning `false` causes the invocation of the algorithm to abort early and return
        /// `Err(`[`Aborted`]`)`.
        fn do_recur(&mut self) -> bool;
    }

    /// Abstraction of recursion continuations.
    ///
    /// Implemented on [`Equiv`] of various types, instead of on the stack types, so that these
    /// methods have access to the entire `Equiv` values.
    pub trait Recur
    {
        /// Constrains `impl<N>` for a particular [`Equiv`] variation.
        type Node: Node;

        /// Arrange for the given nodes to be recurred on, either immediately or later.
        ///
        /// When recurred on immediately, the result must be that of comparing the given nodes
        /// (`Ok`) or attempting to (`Err`).  When saved for later, the result must be `Ok(true)`
        /// and [`Self::next`] must supply these nodes at some point.
        ///
        /// Returning values other than `Ok(true)` causes the invocation of the algorithm to
        /// immediately return the `Ok(false)` or `Err(Aborted)` value.
        fn recur(
            &mut self,
            a: Self::Node,
            b: Self::Node,
        ) -> Result<bool, Aborted>;

        /// Supply the next counterpart nodes for the algorithm to compare, if any were saved for
        /// later by [`Self::recur`].
        fn next(&mut self) -> Option<(Self::Node, Self::Node)>;
    }


    /// The primary logic of the algorithm.
    ///
    /// This generic design works with the [`Node`], [`Descend`], and [`Recur`] traits to enable
    /// variations.
    impl<N: Node, M, S> Equiv<M, S>
    where Self: Descend<Node = N> + Recur<Node = N>
    {
        /// Convenience that calls [`Self::equiv`] and returns `true` if the given nodes are
        /// equivalent, `false` if not or if the algorithm aborted early (which can be impossible
        /// for some variations).
        #[inline]
        pub fn is_equiv<T: Borrow<N>>(
            &mut self,
            ai: T,
            bi: T,
        ) -> bool
        {
            matches!(self.equiv(ai, bi), Ok(true))
        }

        /// The entry-point of the algorithm.
        ///
        /// Returns `Ok(true)` if the given nodes are equivalent, according to the trait
        /// implementations that define the variation of the logic.
        ///
        /// Returns `Ok(false)` if the given nodes are unequivalent.
        ///
        /// Returns `Err(Aborted)` if the [`Descend::do_recur`] indicates to abort early.
        #[inline]
        pub fn equiv<T: Borrow<N>>(
            &mut self,
            ai: T,
            bi: T,
        ) -> Result<bool, Aborted>
        {
            let (mut ar, mut br) = (ai.borrow(), bi.borrow());
            let (mut ao, mut bo): (N, N);

            // This loop, when used in conjunction with certain `self.recur` and `self.next`
            // implementations, is what prevents growing the call-stack, and so prevents the
            // possibility of stack overflow, when traversing descendents.  For other
            // implementations where the `self.recur` does grow the call-stack, the `self.next`
            // always returns `None` and so this loop should be optimized away.
            loop {
                match self.equiv_main(ar, br) {
                    Ok(true) => (),
                    result => return result,
                }

                if let Some((an, bn)) = self.next() {
                    ao = an;
                    bo = bn;
                    ar = &ao;
                    br = &bo;
                }
                else {
                    break Ok(true);
                }
            }
        }

        /// The main logic of the algorithm.
        ///
        /// Must not be used as the initial entry-point, but may be called by [`Recur::recur`]
        /// implementations.
        pub fn equiv_main(
            &mut self,
            a: &N,
            b: &N,
        ) -> Result<bool, Aborted>
        {
            // For trait method implementations that always return the same constant, dead
            // branches should be eliminated by the optimizer.  For the other methods, inlining
            // should be doable by the optimizer.

            if a.id() == b.id() {
            }
            else if let Some(amount_edges) = a.equiv_modulo_descendents_then_amount_edges(b) {
                let mut i = 0.into();
                if i < amount_edges && self.do_edges(a, b) {
                    while i < amount_edges {
                        self.ticker = self.ticker.saturating_sub(1);
                        if self.do_recur() {
                            let (ae, be) = (a.get_edge(&i), b.get_edge(&i));
                            match self.recur(ae, be) {
                                Ok(true) => (),
                                result => return result,
                            }
                        }
                        else {
                            return Err(Aborted);
                        }
                        i += 1.into();
                    }
                }
            }
            else {
                return Ok(false);
            }
            Ok(true)
        }
    }
}


// TODO: In the future, when these aspects are made generic
// pub mod equiv_classes;
