pub use premade::*;

mod premade
{
    use {
        super::recursion::{
            self,
            stack::RecurStack,
        },
        crate::{
            anticipated_or_like::Infallible,
            basic::modes::{
                limited::{
                    LimitReached,
                    Limited,
                    Ticker,
                },
                unlimited::Unlimited,
            },
            generic::equiv::{
                self,
                Equiv,
            },
            Node,
        },
        core::marker::PhantomData,
    };

    #[cfg(not(feature = "anticipate"))]
    use crate::like_anticipated::IntoOk as _;

    /// Equivalence predicate that can handle very-wide graphs but not cyclic graphs.
    #[inline]
    pub fn equiv<N: Node>(
        a: N,
        b: N,
    ) -> N::Cmp
    {
        struct Args<N>(PhantomData<N>);

        impl<N: Node> equiv::Params for Args<N>
        {
            type DescendMode = Unlimited;
            type Error = Infallible;
            type Node = N;
            type RecurMode = RecurStack<Self>;
        }

        impl<N: Node> recursion::stack::Params for Args<N>
        {
            type Node = N;
        }

        let mut e = Equiv::<Args<N>>::default();
        #[allow(unstable_name_collisions)]
        e.equiv(a, b).into_ok()
    }

    /// Equivalence predicate that limits how many nodes are traversed, and that aborts early if
    /// the limit is reached.  Like [`equiv`](equiv()), this can handle very-wide graphs but not
    /// cyclic graphs.
    ///
    /// # Errors
    /// If the limit is reached before completing, return `Err(LimitReached)`.
    #[inline]
    pub fn limited_equiv<N: Node, L: Ticker>(
        limit: L,
        a: N,
        b: N,
    ) -> Result<N::Cmp, LimitReached>
    {
        struct Args<N, L>(PhantomData<(N, L)>);

        impl<N: Node, L: Ticker> equiv::Params for Args<N, L>
        {
            type DescendMode = Limited<L>;
            type Error = LimitReached;
            type Node = N;
            type RecurMode = RecurStack<Self>;
        }

        impl<N: Node, L: Ticker> recursion::stack::Params for Args<N, L>
        {
            type Node = N;
        }

        let mut e = Equiv::<Args<N, L>>::new(Limited(limit));
        e.equiv(a, b)
    }
}


/// Extend the algorithm to be able to traverse very-wide graphs.
pub mod recursion
{
    pub mod stack
    {
        //! Use `LazyVecStack` for the recursion stack, instead of the call-stack.
        //!
        //! The performance is competitive with, and sometimes better than, the call-stack.

        use crate::{
            anticipated_or_like::Infallible,
            basic::recursion::callstack::CallStack,
            generic::equiv::{
                self,
                CounterpartsResult,
                EdgesIter,
                Equiv,
                RecurMode,
            },
            utils::{
                LazierIterator as _,
                LazyVecStack,
            },
            Cmp,
            Node,
        };

        /// Generic parameters of [`RecurStack`] and its operations.
        pub trait Params
        {
            /// Amount of elements that a [`RecurStack`] can contain initially before
            /// reallocating.
            ///
            /// An `impl` of [`Params`] may be made with a different value - either smaller or
            /// larger.  Note that the default only affects the initial capacity of the underlying
            /// `Vec`, and it will still grow as large as needed regardless by reallocating.
            ///
            /// The maximum amount of elements depends on the order in which edges are given by
            /// the [`Node::get_edge`] implementation for an input type.  (See also the
            /// documentation of [`RecurStack`].)
            const INITIAL_CAPACITY: usize = 2_usize.pow(4);
            /// Type of node that is saved on a stack.  Must be the same as used with the
            /// corresponding [`equiv::Params`].
            type Node: Node;
        }

        /// Stack of lazily-generated pairs of nodes that must next be compared pairwise.  The
        /// size is limited only by available memory.  Specifies use of this.
        ///
        /// Does depth-first preorder traversals.  Typically used when it is likely that the input
        /// graphs will be wider than they are deep.  Great width can be handled with very little
        /// memory usage, but great depth can cause excessive memory usage (when "tail-call
        /// elimination" cannot be achieved).
        ///
        /// The memory usage depends on the order in which edges are given by the
        /// [`Node::get_edge`] implementation for an input type.  For some shapes, e.g. lists, the
        /// order can be chosen to give the analogue of "tail-call elimination" to achieve limited
        /// elements maximum on a stack even for very long shapes, by giving the deeper "tail"
        /// edge of a node after its other shallower edges so that the shallower edges are
        /// descended before the deeper "tail", which limits the max to only the elements needed
        /// to descend shallower edges.  This approach might also be doable for some shapes that
        /// have multiple "tails", by giving edges in different orders based on the different
        /// position of each node in a shape.
        ///
        /// (If, instead, you want to limit how much a recursion-stack can grow, you must `impl`
        /// [`RecurMode`] for your own type that does that and use it with the
        /// [`generic`](crate::generic) API.)
        #[allow(clippy::module_name_repetitions)]
        pub struct RecurStack<P: Params>(LazyVecStack<EdgesIter<P::Node>>);

        impl<P: Params> Default for RecurStack<P>
        {
            /// Create a new instance with capacity
            /// [`P::INITIAL_CAPACITY`](Params::INITIAL_CAPACITY).
            #[inline]
            fn default() -> Self
            {
                Self(LazyVecStack::with_capacity(P::INITIAL_CAPACITY))
            }
        }

        /// Enables the call-stack to be used for the precheck and the vector-stack for the
        /// interleave, if desired.
        impl<P: Params> From<CallStack> for RecurStack<P>
        {
            #[inline]
            fn from(_: CallStack) -> Self
            {
                Self::default()
            }
        }

        /// Enables [`RecurStack`] to be used with the algorithm.
        impl<E, V> RecurMode<E> for RecurStack<V>
        where
            E: equiv::Params<RecurMode = Self>,
            V: Params<Node = E::Node>,
            Infallible: Into<E::Error>,
        {
            type Error = Infallible;

            #[inline]
            fn recur(
                it: &mut Equiv<E>,
                edges_iter: EdgesIter<E::Node>,
            ) -> Result<<E::Node as Node>::Cmp, Self::Error>
            {
                it.recur_mode.0.extend(edges_iter);
                Ok(Cmp::new_equiv())
            }

            #[inline]
            fn next(&mut self) -> Option<CounterpartsResult<E::Node>>
            {
                self.0.next()
            }

            /// An aborted precheck, that uses `RecurStack`, might have left some elements, so we
            /// must reset before doing the interleave using the same `RecurStack`.
            #[inline]
            fn reset(mut self) -> Self
            {
                self.0.clear();
                self
            }
        }
    }
}
