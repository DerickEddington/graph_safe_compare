pub use premade::*;

mod premade
{
    use {
        super::recursion::vecstack::{
            self,
            VecStack,
        },
        crate::{
            basic::modes::unlimited::Unlimited,
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
            type RecurStack = VecStack<Self>;
        }

        impl<N: Node> vecstack::Params for Args<N>
        {
            type Node = N;
        }

        let mut e = Equiv::<Args<N>>::default();
        e.equiv(a, b).into_ok()
    }
}


/// Extend the algorithm to be able to traverse very-deep graphs.
pub mod recursion
{
    pub mod vecstack
    {
        //! Use [`Vec`] for the recursion stack, instead of the call-stack.
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
                    RecurStack,
                },
                Cmp,
                Node,
            },
            alloc::vec::Vec,
            core::convert::Infallible,
        };

        /// Generic parameters of [`VecStack`] and its operations.
        pub trait Params
        {
            /// Amount of elements that a [`VecStack`] can contain initially before reallocating.
            ///
            /// The default value is a balance between being somewhat large to avoid excessive
            /// reallocations and not being too huge that it often consumes excessive memory.
            /// Typically, [`VecStack`] is used when it is likely that the input graphs will be
            /// deep but not extremely so.  When that is not the case, a custom `impl` of
            /// [`Params`] may be made with a more-appropriate value - either smaller or larger.
            /// Note that the default only affects the initial capacity of the underlying [`Vec`],
            /// and it will still grow as large as needed regardless by reallocating.
            ///
            /// The maximum amount of elements depends on the maximum recursion depth (which
            /// depends on the shape of an input value) and it depends on the order in which edges
            /// are given by the [`Node::get_edge`] implementation for an input type.  For some
            /// shapes, like lists, the order can be chosen to have a kind of "tail-call
            /// elimination" to achieve very few elements max on a stack even for very long
            /// shapes, by giving the deeper "tail" of a shape first before other shallower edges
            /// so that the shallower edges are descended first (because they are pushed after and
            /// so popped before) and then the deeper "tail" is descended last (because it is
            /// pushed first and so popped last), which limits the max to only the few elements
            /// needed to descend shallower edges.  This approach might also be doable for some
            /// shapes that have multiple "tails".
            const INITIAL_CAPACITY: usize = 2_usize.pow(17);
            /// Type of node that is saved on a stack.  Must be the same as used with the
            /// corresponding [`equiv::Params`].
            type Node: Node;
        }

        /// Stack of pairs of nodes that must next be compared pairwise.  The size, which
        /// corresponds to the maximum graph depth, is limited only by available memory.
        /// Specifies use of this.
        ///
        /// Does depth-first preorder traversals.
        ///
        /// (If, instead, you want to limit how much a recursion-stack can grow, you must `impl`
        /// [`RecurStack`] for your own type that does that and use it with the
        /// [`generic`](crate::generic) API.)
        pub struct VecStack<P: Params>(Vec<EdgesIter<P::Node>>);

        impl<P: Params> Default for VecStack<P>
        {
            /// Create a new instance with capacity
            /// [`P::INITIAL_CAPACITY`](Params::INITIAL_CAPACITY).
            #[inline]
            fn default() -> Self
            {
                Self(Vec::with_capacity(P::INITIAL_CAPACITY))
            }
        }

        /// Enables the call-stack to be used for the precheck and the vector-stack for the
        /// interleave, if desired.
        impl<P: Params> From<CallStack> for VecStack<P>
        {
            #[inline]
            fn from(_: CallStack) -> Self
            {
                Self::default()
            }
        }

        /// Enables [`VecStack`] to be used with the algorithm.
        impl<E, V> RecurStack<E> for VecStack<V>
        where
            E: equiv::Params<RecurStack = Self>,
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
                it.recur_stack.0.push(edges_iter);
                Ok(Cmp::new_equiv())
            }

            #[inline]
            fn next(&mut self) -> Option<(E::Node, E::Node)>
            {
                if let Some(edges_iter) = self.0.last_mut() {
                    let next = edges_iter.next();
                    if edges_iter.is_empty() {
                        // Prevent empty iterators from staying on the stack.
                        drop(self.0.pop());
                    }
                    debug_assert!(next.is_some());
                    next
                }
                else {
                    None
                }
            }

            /// An aborted precheck, that uses `VecStack`, might have left some elements, so we
            /// must reset before doing the interleave using the same `VecStack`.
            #[inline]
            fn reset(mut self) -> Self
            {
                self.0.clear();
                self
            }
        }
    }
}
