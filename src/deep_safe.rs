pub use premade::*;

mod premade
{
    use {
        super::recursion::vecqueue::{
            self,
            VecQueue,
        },
        crate::{
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
            utils::IntoOk as _,
            Node,
        },
        core::{
            convert::Infallible,
            marker::PhantomData,
        },
    };

    /// Equivalence predicate that can handle very-deep graphs but not cyclic graphs.
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
            type RecurMode = VecQueue<Self>;
        }

        impl<N: Node> vecqueue::Params for Args<N>
        {
            type Node = N;
        }

        let mut e = Equiv::<Args<N>>::default();
        #[allow(unstable_name_collisions)]
        e.equiv(a, b).into_ok()
    }

    /// Equivalence predicate that limits how many nodes are traversed, and that aborts early if
    /// the limit is reached.  Like [`equiv`](equiv()), this can handle very-deep graphs but not
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
            type RecurMode = VecQueue<Self>;
        }

        impl<N: Node, L: Ticker> vecqueue::Params for Args<N, L>
        {
            type Node = N;
        }

        let mut e = Equiv::<Args<N, L>>::new(Limited(limit));
        e.equiv(a, b)
    }
}


/// Extend the algorithm to be able to traverse very-deep graphs.
pub mod recursion
{
    pub mod vecqueue
    {
        //! Use [`VecDeque`] for the recursion continuations, instead of the call-stack.
        //!
        //! The performance is competitive with, and sometimes better than, the call-stack.

        extern crate alloc;

        use {
            crate::{
                basic::recursion::callstack::CallStack,
                generic::equiv::{
                    self,
                    EdgesIter,
                    Equiv,
                    RecurMode,
                },
                Cmp,
                Node,
            },
            alloc::collections::VecDeque,
            core::convert::Infallible,
        };

        /// Generic parameters of [`VecQueue`] and its operations.
        pub trait Params
        {
            /// Amount of elements that a [`VecQueue`] can contain initially before reallocating.
            ///
            /// An `impl` of [`Params`] may be made with a different value - either smaller or
            /// larger.  Note that the default only affects the initial capacity of the underlying
            /// [`VecDeque`], and it will still grow as large as needed regardless by
            /// reallocating.
            const INITIAL_CAPACITY: usize = 2_usize.pow(4);
            /// Type of node that is saved in a queue.  Must be the same as used with the
            /// corresponding [`equiv::Params`].
            type Node: Node;
        }

        /// Queue of pairs of nodes that must next be compared pairwise.  The size is limited only
        /// by available memory.  Specifies use of this.
        ///
        /// Does breadth-first traversals.  Typically used when it is likely that the input graphs
        /// will be deeper than they are wide.  Great depth can be handled with very little memory
        /// usage, but great width can cause excessive memory usage.
        ///
        /// (If, instead, you want to limit how much a recursion-queue can grow, you must `impl`
        /// [`RecurMode`] for your own type that does that and use it with the
        /// [`generic`](crate::generic) API.)
        pub struct VecQueue<P: Params>(VecDeque<EdgesIter<P::Node>>);

        impl<P: Params> Default for VecQueue<P>
        {
            /// Create a new instance with capacity
            /// [`P::INITIAL_CAPACITY`](Params::INITIAL_CAPACITY).
            #[inline]
            fn default() -> Self
            {
                Self(VecDeque::with_capacity(P::INITIAL_CAPACITY))
            }
        }

        /// Enables the call-stack to be used for the precheck and the vector-queue for the
        /// interleave, if desired.
        impl<P: Params> From<CallStack> for VecQueue<P>
        {
            #[inline]
            fn from(_: CallStack) -> Self
            {
                Self::default()
            }
        }

        /// Enables [`VecQueue`] to be used with the algorithm.
        impl<E, V> RecurMode<E> for VecQueue<V>
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
                debug_assert!(!edges_iter.is_empty());
                it.recur_mode.0.push_back(edges_iter);
                Ok(Cmp::new_equiv())
            }

            #[inline]
            fn next(&mut self) -> Option<(E::Node, E::Node)>
            {
                if let Some(edges_iter) = self.0.front_mut() {
                    let next = edges_iter.next();
                    if edges_iter.is_empty() {
                        // Prevent empty iterators from staying in the queue.
                        drop(self.0.pop_front());
                    }
                    debug_assert!(next.is_some());
                    next
                }
                else {
                    None
                }
            }

            /// An aborted precheck, that uses `VecQueue`, might have left some elements, so we
            /// must reset before doing the interleave using the same `VecQueue`.
            #[inline]
            fn reset(mut self) -> Self
            {
                self.0.clear();
                self
            }
        }
    }
}
