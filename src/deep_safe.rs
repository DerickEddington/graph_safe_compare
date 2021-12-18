use {
    self::recursion::vecstack::VecStack,
    crate::{
        basic::modes::unlimited::Unlimited,
        generic::equiv::Equiv,
        Node,
    },
};


/// Equivalence predicate that can handle very-deep graphs but not cyclic graphs.
#[inline]
pub fn equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    let mut e = Equiv::<Unlimited<N>, _>::new(VecStack::default());
    e.is_equiv(a, b)
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
                generic::{
                    equiv::{
                        Aborted,
                        Equiv,
                        Recur,
                    },
                    recursion::Reset,
                },
                Node,
            },
            alloc::vec::Vec,
        };

        // TODO: Maybe this should be much larger, since VecStack will be used when large depth is
        // expected.  Maybe the user should be able to pass a value thru the API for this instead.
        const DEFAULT_CAPACITY: usize = 32;

        /// Stack of pairs of nodes that must next be compared pairwise.  The size, which
        /// corresponds to the maximum graph depth, is limited only by available memory.
        /// Specifies use of this.
        pub struct VecStack<N>(Vec<(N, N)>);

        impl<N> Default for VecStack<N>
        {
            #[inline]
            fn default() -> Self
            {
                Self(Vec::with_capacity(DEFAULT_CAPACITY))
            }
        }

        /// Enables the call-stack to be used for the precheck and the vector-stack for the
        /// interleave, if desired.
        impl<N> From<CallStack> for VecStack<N>
        {
            #[inline]
            fn from(_: CallStack) -> Self
            {
                Self::default()
            }
        }

        /// An aborted precheck, that uses `VecStack`, might have left some elements, so we must
        /// reset before doing the interleave using the same `VecStack`.
        impl<N> Reset for VecStack<N>
        {
            #[inline]
            fn reset(mut self) -> Self
            {
                self.0.clear();
                self
            }
        }

        /// Enables [`VecStack`] to be used with the algorithm.
        impl<N: Node, M> Recur<VecStack<N>> for Equiv<M, VecStack<N>>
        {
            type Node = N;

            #[inline]
            fn recur(
                &mut self,
                a: Self::Node,
                b: Self::Node,
            ) -> Result<bool, Aborted>
            {
                self.recur_stack.0.push((a, b));
                Ok(true)
            }

            #[inline]
            fn next(&mut self) -> Option<(Self::Node, Self::Node)>
            {
                self.recur_stack.0.pop()
            }
        }
    }
}
