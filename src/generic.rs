pub use premade::*;

pub mod equiv_classes;


mod premade
{
    /// The primary variation of the algorithm, like that of the paper.
    pub mod precheck_interleave
    {
        #[allow(unreachable_pub)]
        mod sealed
        {
            use {
                super::{
                    super::super::equiv,
                    Params,
                },
                crate::{
                    basic::modes::limited::Limited,
                    cycle_safe::modes::interleave::Interleave,
                    Node,
                },
                core::marker::PhantomData,
            };

            pub struct PrecheckArgs<N, P>(PhantomData<(N, P)>);

            impl<N: Node, P: Params<N>> equiv::Params for PrecheckArgs<N, P>
            {
                type DescendMode = Limited<u16>;
                type Node = N;
                type RecurStack = P::PrecheckRecurStack;
            }

            pub struct InterleaveArgs<N, P>(PhantomData<(N, P)>);

            impl<N: Node, P: Params<N>> equiv::Params for InterleaveArgs<N, P>
            {
                type DescendMode = Interleave<P::InterleaveParams>;
                type Node = N;
                type RecurStack = P::InterleaveRecurStack;
            }
        }

        use {
            super::super::equiv::{
                Equiv,
                RecurStack,
            },
            crate::{
                basic::modes::limited::Limited,
                cycle_safe::modes::interleave,
                Node,
            },
            core::ops::ControlFlow,
            sealed::{
                InterleaveArgs,
                PrecheckArgs,
            },
        };


        /// Generic parameters of [`equiv`].
        pub trait Params<N: Node>: Sized
        {
            /// Type of recursion stack for the precheck.
            type PrecheckRecurStack: RecurStack<PrecheckArgs<N, Self>>
                + Into<Self::InterleaveRecurStack>;
            /// Type of recursion stack for the interleave.
            type InterleaveRecurStack: RecurStack<InterleaveArgs<N, Self>>;
            /// Type that `impl`s the arguments for the generic parameters for the interleave.
            type InterleaveParams: interleave::Params<Node = N>;
        }

        /// Equivalence predicate that can handle cyclic graphs, but first tries the precheck that
        /// is faster for small acyclic graphs, and that requires choosing specific type arguments
        /// that determine the implementations of internal dynamic data structures.  Safe for
        /// very-deep graphs only when the interleave recursion-stack type is.
        #[inline]
        pub fn equiv<N, P>(
            a: &N,
            b: &N,
        ) -> bool
        where
            N: Node,
            P: Params<N>,
        {
            use interleave::Params as _;

            let mut e =
                Equiv::<PrecheckArgs<N, P>>::new(Limited(P::InterleaveParams::PRECHECK_LIMIT));

            match e.precheck_equiv(a, b) {
                ControlFlow::Break(result) => result,
                ControlFlow::Continue(()) => {
                    let mut e: Equiv<InterleaveArgs<N, P>> = e.into();
                    e.is_equiv(a, b)
                },
            }
        }
    }
}


mod modes
{
    use super::equiv::Params;

    /// Controls if node edges are descended into.
    pub trait DescendMode<P: Params>
    {
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
            a: &P::Node,
            b: &P::Node,
        ) -> bool;

        /// Controls if individual descendent counterparts will be gotten and descended into via
        /// recursion.  Called after getting and comparing any previous sibling descendents.
        ///
        /// Returning `true` causes the next counterparts to be gotten and recurred on to compare
        /// them.
        ///
        /// Returning `false` causes the invocation of the algorithm to abort early and return
        /// `Err(`[`Aborted`](super::equiv::Aborted)`)`.
        fn do_recur(&mut self) -> bool;
    }
}


mod recursion
{
    use super::equiv::{
        Aborted,
        Equiv,
        Params,
    };

    /// Abstraction of recursion continuations.
    pub trait RecurStack<P: Params>: Default
    {
        /// Arrange for the given nodes to be recurred on, either immediately or later.
        ///
        /// The `it` parameter enables accessing the entire [`Equiv`] value.
        ///
        /// When recurred on immediately, the result must be that of comparing the given nodes
        /// (`Ok`) or attempting to (`Err`).  When saved for later, the result must be `Ok(true)`
        /// and [`Self::next`] must supply these nodes at some point, or the result must be
        /// `Err(Aborted)` if the implementor wants to impose limiting.
        ///
        /// Returning values other than `Ok(true)` causes the invocation of the algorithm to
        /// immediately return the `Ok(false)` or `Err(Aborted)` value.
        ///
        /// # Errors
        /// If aborted early, returns `Err(Aborted)`.
        fn recur(
            it: &mut Equiv<P>,
            a: P::Node,
            b: P::Node,
        ) -> Result<bool, Aborted>;

        /// Supply the next counterpart nodes for the algorithm to compare, if any were saved for
        /// later by [`Self::recur`].
        fn next(&mut self) -> Option<(P::Node, P::Node)>;

        /// Reset to be empty while preserving capacity, if relevant.
        ///
        /// An aborted precheck, that uses particular types of recursion-stacks, might leave some
        /// elements on such a stack, in which case it needs to be reset before doing the
        /// interleave using the same stack.
        ///
        /// Also, some things that take ownership of a `Self` might call this to ensure a stack is
        /// in a fresh state.
        ///
        /// When it is more efficient, the `self` value should be reset and then returned.  But a
        /// newly-created value may be returned if desired.
        fn reset(self) -> Self;
    }
}


/// The central parts of the algorithm.
pub mod equiv
{
    use {
        crate::Node,
        core::borrow::Borrow,
    };

    pub use super::{
        modes::DescendMode,
        recursion::RecurStack,
    };

    /// Generic parameters of [`Equiv`] and its operations.
    pub trait Params: Sized
    {
        /// Type of node to handle.
        type Node: Node;
        /// Type that controls descending node edges.
        type DescendMode: DescendMode<Self>;
        /// Type that provides recursion continuations.
        type RecurStack: RecurStack<Self>;
    }

    /// The state for an invocation of a variation of the algorithm.
    #[non_exhaustive]
    pub struct Equiv<P: Params>
    {
        /// Controls if node edges are descended into.
        pub(crate) descend_mode: P::DescendMode,
        /// Representation of recursion continuations.
        pub recur_stack:         P::RecurStack,
    }

    impl<P: Params> Equiv<P>
    {
        /// Create a new instance to use once for a single invocation of the algorithm.
        ///
        /// For use with [`DescendMode`] types that cannot implement `Default`.
        #[inline]
        pub fn new(descend_mode: P::DescendMode) -> Self
        {
            Self { descend_mode, recur_stack: P::RecurStack::default() }
        }
    }

    impl<P: Params> Default for Equiv<P>
    where P::DescendMode: Default
    {
        /// Create a new instance to use once for a single invocation of the algorithm.
        ///
        /// For use with [`DescendMode`] types that implement `Default`.
        #[inline]
        fn default() -> Self
        {
            Self {
                descend_mode: P::DescendMode::default(),
                recur_stack:  P::RecurStack::default(),
            }
        }
    }

    /// Enables the same recursion-stack value to be reused across the precheck and the
    /// interleave, which is more efficient for some types since this avoids dropping it and
    /// creating another.
    ///
    /// [`From`] or [`Into`] cannot be `impl`ed for this, because that would conflict with the
    /// blanket implementations provided by the `core` library.
    impl<PT: Params> Equiv<PT>
    where PT::DescendMode: Default
    {
        /// Like [`From::from`].
        #[inline]
        pub fn from<PF: Params>(e: Equiv<PF>) -> Self
        where PF::RecurStack: Into<PT::RecurStack>
        {
            Self {
                descend_mode: PT::DescendMode::default(),
                recur_stack:  e.recur_stack.reset().into(),
            }
        }
    }

    impl<PF: Params> Equiv<PF>
    {
        /// Like [`Into::into`].
        #[inline]
        pub fn into<PT: Params>(self) -> Equiv<PT>
        where
            PF::RecurStack: Into<PT::RecurStack>,
            PT::DescendMode: Default,
        {
            Equiv::from(self)
        }
    }


    /// Indicates that the algorithm aborted early without determining the result.
    ///
    /// This occurs when the [`DescendMode::do_recur`] returns `false` or when the
    /// [`RecurStack::recur`] returns `Err(Aborted)`.
    ///
    /// Used as the value in a `Result::Err`.
    #[non_exhaustive]
    pub struct Aborted;


    /// The primary logic of the algorithm.
    ///
    /// This generic design works with the [`Node`], [`DescendMode`], and [`RecurStack`] traits to
    /// enable variations.
    impl<P: Params> Equiv<P>
    {
        /// Convenience that calls [`Self::equiv`] and returns `true` if the given nodes are
        /// equivalent, `false` if not or if the algorithm aborted early (which can be impossible
        /// for some variations).
        #[inline]
        pub fn is_equiv<T: Borrow<P::Node>>(
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
        /// # Errors
        /// Returns `Err(Aborted)` if the [`DescendMode::do_recur`] or the [`RecurStack::recur`]
        /// indicates to abort early.
        #[inline]
        pub fn equiv<T: Borrow<P::Node>>(
            &mut self,
            ai: T,
            bi: T,
        ) -> Result<bool, Aborted>
        {
            let (mut ar, mut br) = (ai.borrow(), bi.borrow());
            let (mut ao, mut bo);

            // This loop, when used in conjunction with certain `RecurStack::recur` and
            // `RecurStack::next` implementations, is what prevents growing the call-stack, and so
            // prevents the possibility of stack overflow, when traversing descendents.  For other
            // implementations where the `RecurStack::recur` does grow the call-stack, the
            // `RecurStack::next` always returns `None` and so this loop should be optimized away.
            loop {
                match self.equiv_main(ar, br) {
                    Ok(true) => (),
                    result => return result,
                }

                if let Some((an, bn)) = self.recur_stack.next() {
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
        /// Must not be used as the initial entry-point, but may be called by
        /// [`RecurStack::recur`] implementations.
        ///
        /// Returns same as [`Self::equiv`].
        ///
        /// # Errors
        /// Same as [`Self::equiv`].
        #[inline]
        pub fn equiv_main(
            &mut self,
            a: &P::Node,
            b: &P::Node,
        ) -> Result<bool, Aborted>
        {
            // For trait method implementations that always return the same constant, dead
            // branches should be eliminated by the optimizer.  For the other methods, inlining
            // should be doable by the optimizer.

            if a.id() == b.id() {
            }
            else if let Some(amount_edges) = a.equiv_modulo_descendents_then_amount_edges(b) {
                let mut i = 0.into();
                if i < amount_edges && self.descend_mode.do_edges(a, b) {
                    while i < amount_edges {
                        if self.descend_mode.do_recur() {
                            let (ae, be) = (a.get_edge(&i), b.get_edge(&i));
                            match P::RecurStack::recur(self, ae, be) {
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
