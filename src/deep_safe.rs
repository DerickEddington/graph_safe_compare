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
            Node,
        },
        core::marker::PhantomData,
    };

    /// Equivalence predicate that can handle very-deep graphs but not cyclic graphs.
    #[inline]
    pub fn equiv<N: Node>(
        a: &N,
        b: &N,
    ) -> bool
    {
        struct Args<N>(PhantomData<N>);

        impl<N: Node> equiv::Params for Args<N>
        {
            type DescendMode = Unlimited;
            type Node = N;
            type RecurStack = VecStack<Self>;
        }

        impl<N: Node> vecstack::Params for Args<N>
        {
            type Node = N;
        }

        let mut e = Equiv::<Args<N>>::default();
        e.is_equiv(a, b)
    }
}


/// Extend the algorithm to be able to traverse very-deep graphs.
pub mod recursion
{
    pub mod vecstack
    {
        //! Use [`Vec`] for the recursion stack, instead of the call-stack.

        extern crate alloc;

        use {
            crate::{
                basic::recursion::callstack::CallStack,
                generic::equiv::{
                    self,
                    Aborted,
                    Equiv,
                    RecurStack,
                },
                Node,
            },
            alloc::vec::Vec,
        };

        /// Generic parameters of [`VecStack`] and its operations.
        pub trait Params
        {
            /// Amount of elements (i.e. recursion depth) that a stack can grow to contain
            /// initially before reallocating.
            ///
            /// The default value is a balance between being somewhat large to avoid excessive
            /// reallocations and not being too huge that it often consumes excessive memory.
            /// Typically, [`VecStack`] is used when it is likely that the input graphs will be
            /// deep but not extremely so.  When that is not the case, a custom `impl` of
            /// [`Params`] may be made with a more-appropriate value - either smaller or larger.
            /// Note that the default only affects the initial capacity of the underlying [`Vec`],
            /// and it will still grow as large as needed regardless by reallocating.
            const INITIAL_CAPACITY: usize = 2_usize.pow(20);
            /// Type of node that is saved on a stack.  Must be the same as used with the
            /// corresponding [`equiv::Params`].
            type Node: Node;
        }

        /// Stack of pairs of nodes that must next be compared pairwise.  The size, which
        /// corresponds to the maximum graph depth, is limited only by available memory.
        /// Specifies use of this.
        ///
        /// (If, instead, you want to limit how much a recursion-stack can grow, you must `impl`
        /// [`RecurStack`] for your own type that does that and use it with the
        /// [`generic`](crate::generic) API.)
        pub struct VecStack<P: Params>(Vec<(P::Node, P::Node)>);

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
        {
            #[inline]
            fn recur(
                it: &mut Equiv<E>,
                a: E::Node,
                b: E::Node,
            ) -> Result<bool, Aborted>
            {
                it.recur_stack.0.push((a, b));
                Ok(true)
            }

            #[inline]
            fn next(&mut self) -> Option<(E::Node, E::Node)>
            {
                self.0.pop()
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
